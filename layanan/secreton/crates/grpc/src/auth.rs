//! Client authentication for the secreton gRPC boundary.
//!
//! # Why this module exists
//!
//! Until this landed, both serving paths called `Server::builder()` with no
//! interceptor and no layer. Anything that could open a TCP connection to the
//! gRPC port could read, write and delete every secret in the engine — the
//! whole point of the service. The thing that *looked* like the control was
//! `interceptor::auth_interceptor`: it had zero callers, and on the one path
//! that mattered (no `authorization` header at all) it returned `Ok(req)`.
//!
//! # What authenticates a caller
//!
//! The client certificate presented during the TLS handshake. By the time a
//! request reaches this layer rustls has **already** verified that the
//! certificate chains to the CA configured via [`crate::GrpcTlsConfig`] and is
//! inside its validity window — that is what `client_ca_root` does. So this
//! layer does not re-verify the chain (a second, hand-rolled verification is
//! how you end up with a *weaker* check than the one you already had). It
//! does exactly two things:
//!
//! 1. **Refuse** any request that arrived without a verified peer certificate.
//!    That is the default-DENY: no certificate, no call, no exceptions.
//! 2. Turn the leaf certificate into a [`PeerIdentity`] and hand it to a
//!    [`PeerAuthorizer`], then attach it to the request extensions so handlers
//!    can see who called them.
//!
//! # What this does NOT do
//!
//! mTLS authenticates a **workload**, it does not authorize a **request**. A
//! valid `gateway` certificate proves the caller is the gateway; it says
//! nothing about whether the gateway may read `kv/prod/db-password`. Per-path
//! scoping needs the policy engine to actually be consulted on the secret
//! read/write path, which is a separate piece of work (task #129). Do not read
//! "mTLS is on" as "secrets are access-controlled".

use std::collections::BTreeSet;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use async_trait::async_trait;
use tonic::Status;
use tonic::transport::server::{TcpConnectInfo, TlsConnectInfo};
use tower::{Layer, Service};
use tracing::{debug, warn};

/// Environment variable holding a comma-separated allowlist of client
/// certificate common names. Unset means "any peer the CA vouched for".
pub const ALLOWED_COMMON_NAMES_ENV: &str = "GRPC_ALLOWED_CLIENT_CNS";

/// The identity a caller proved by presenting a client certificate.
///
/// Attached to request extensions by [`ClientAuth`], so a handler can read it
/// with `request.extensions().get::<PeerIdentity>()`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PeerIdentity {
    /// Subject Common Name of the leaf certificate. This is the workload name
    /// the mesh issues certificates under (`gateway`, `layanan-perlengkapan`, …).
    pub common_name: String,
    /// Lowercase hex serial of the leaf certificate — the stable handle for a
    /// revocation list, since common names get reissued.
    pub serial: String,
}

impl PeerIdentity {
    /// Parse the identity out of a DER-encoded leaf certificate.
    ///
    /// Chain validity is *not* checked here; rustls did that during the
    /// handshake. See the module docs.
    pub fn from_der(der: &[u8]) -> Result<Self, Status> {
        let (_, cert) = x509_parser::parse_x509_certificate(der).map_err(|e| {
            warn!(error = %e, "client certificate could not be parsed");
            Status::unauthenticated("client certificate is unreadable")
        })?;

        let common_name = cert
            .subject()
            .iter_common_name()
            .next()
            .and_then(|cn| cn.as_str().ok())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                warn!(subject = %cert.subject(), "client certificate has no common name");
                Status::unauthenticated("client certificate has no common name")
            })?;

        Ok(Self {
            common_name,
            serial: hex::encode(cert.raw_serial()),
        })
    }
}

/// Decides whether an authenticated peer may make a given call.
///
/// Async on purpose: the implementations that matter later (policy lookup,
/// revocation list) do I/O. A synchronous `tonic` interceptor cannot await, so
/// enforcement lives in a tower layer instead — see [`ClientAuthLayer`].
#[async_trait]
pub trait PeerAuthorizer: Send + Sync + 'static {
    /// Return `Err(reason)` to deny. `method` is the full gRPC path, e.g.
    /// `/secreton.v1.SecretonService/GetSecret`.
    ///
    /// The reason is logged server-side; the caller only ever sees a generic
    /// denial, so it is safe to be specific here.
    async fn authorize(&self, peer: &PeerIdentity, method: &str) -> Result<(), String>;
}

/// Accepts every peer whose certificate the configured CA vouched for.
///
/// Correct when the CA issues certificates *only* to mesh workloads, so
/// holding a chain-valid certificate is itself the authorization decision.
/// Wrong the moment that CA also signs anything else.
pub struct AnyTrustedPeer;

#[async_trait]
impl PeerAuthorizer for AnyTrustedPeer {
    async fn authorize(&self, peer: &PeerIdentity, method: &str) -> Result<(), String> {
        debug!(peer = %peer.common_name, %method, "peer accepted on CA trust alone");
        Ok(())
    }
}

/// Accepts only the listed common names.
///
/// This is the mapping the Helm `secretonAuth.policies` block already
/// describes (service → policy); here it is enforced as service → allowed.
pub struct AllowedCommonNames {
    allowed: BTreeSet<String>,
}

impl AllowedCommonNames {
    pub fn new<I, S>(names: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            allowed: names.into_iter().map(Into::into).collect(),
        }
    }

    /// Build from [`ALLOWED_COMMON_NAMES_ENV`]. Returns `None` when the
    /// variable is unset or lists no names, so the caller can fall back to
    /// [`AnyTrustedPeer`] deliberately rather than to an empty allowlist that
    /// would deny everything.
    pub fn from_env() -> Option<Self> {
        let raw = std::env::var(ALLOWED_COMMON_NAMES_ENV).ok()?;
        let names: BTreeSet<String> = raw
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect();
        if names.is_empty() {
            None
        } else {
            Some(Self { allowed: names })
        }
    }
}

#[async_trait]
impl PeerAuthorizer for AllowedCommonNames {
    async fn authorize(&self, peer: &PeerIdentity, _method: &str) -> Result<(), String> {
        if self.allowed.contains(&peer.common_name) {
            Ok(())
        } else {
            Err(format!(
                "common name {:?} is not in the client allowlist",
                peer.common_name
            ))
        }
    }
}

/// Tower layer that enforces client authentication on every gRPC method.
///
/// Applied once via `Server::builder().layer(..)` rather than per-method: with
/// 30-odd RPCs on the service, a per-method check is one forgotten line away
/// from an unauthenticated hole.
#[derive(Clone)]
pub struct ClientAuthLayer {
    authorizer: Arc<dyn PeerAuthorizer>,
}

impl ClientAuthLayer {
    pub fn new(authorizer: Arc<dyn PeerAuthorizer>) -> Self {
        Self { authorizer }
    }
}

impl<S> Layer<S> for ClientAuthLayer {
    type Service = ClientAuth<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ClientAuth {
            inner,
            authorizer: Arc::clone(&self.authorizer),
        }
    }
}

/// The service produced by [`ClientAuthLayer`].
#[derive(Clone)]
pub struct ClientAuth<S> {
    inner: S,
    authorizer: Arc<dyn PeerAuthorizer>,
}

impl<S, ReqBody, ResBody> Service<http::Request<ReqBody>> for ClientAuth<S>
where
    S: Service<http::Request<ReqBody>, Response = http::Response<ResBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    ReqBody: Send + 'static,
    ResBody: Default,
{
    type Response = http::Response<ResBody>;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: http::Request<ReqBody>) -> Self::Future {
        // Clone-and-swap: the clone we hold may not be the one `poll_ready`
        // reserved capacity on, so hand the *ready* service to the future.
        let ready = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, ready);
        let authorizer = Arc::clone(&self.authorizer);

        Box::pin(async move {
            let method = req.uri().path().to_string();

            let peer = match peer_identity(&req) {
                Ok(peer) => peer,
                Err(status) => return Ok(status.into_http()),
            };

            if let Err(reason) = authorizer.authorize(&peer, &method).await {
                warn!(
                    peer = %peer.common_name,
                    serial = %peer.serial,
                    %method,
                    %reason,
                    "gRPC call denied"
                );
                // Deliberately generic: the caller learns it was refused, not
                // what the allowlist contains.
                return Ok(Status::permission_denied("caller is not authorized").into_http());
            }

            debug!(peer = %peer.common_name, %method, "gRPC call authenticated");
            req.extensions_mut().insert(peer);
            inner.call(req).await
        })
    }
}

/// Extract the caller's identity, or the `Status` that refuses the call.
fn peer_identity<B>(req: &http::Request<B>) -> Result<PeerIdentity, Status> {
    let certs = req
        .extensions()
        .get::<TlsConnectInfo<TcpConnectInfo>>()
        .and_then(|info| info.peer_certs())
        .ok_or_else(|| {
            // Either the listener is not doing TLS at all, or the handshake
            // completed without a client certificate. Both are the same answer.
            warn!("gRPC call rejected: connection carries no verified client certificate");
            Status::unauthenticated("client certificate required")
        })?;

    let leaf = certs.first().ok_or_else(|| {
        warn!("gRPC call rejected: peer certificate chain is empty");
        Status::unauthenticated("client certificate required")
    })?;

    PeerIdentity::from_der(leaf.as_ref())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn peer(cn: &str) -> PeerIdentity {
        PeerIdentity {
            common_name: cn.to_string(),
            serial: "00".to_string(),
        }
    }

    #[tokio::test]
    async fn an_allowlisted_common_name_is_accepted() {
        let authorizer = AllowedCommonNames::new(["gateway", "layanan-perlengkapan"]);
        assert!(
            authorizer
                .authorize(&peer("gateway"), "/secreton.v1.SecretonService/GetSecret")
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn an_unlisted_common_name_is_denied() {
        let authorizer = AllowedCommonNames::new(["gateway"]);
        let denied = authorizer
            .authorize(
                &peer("someone-else"),
                "/secreton.v1.SecretonService/GetSecret",
            )
            .await;
        assert!(denied.is_err(), "an unlisted CN must not be authorized");
    }

    #[tokio::test]
    async fn the_allowlist_is_case_sensitive() {
        // Common names come from a CA we control, so exact match is the
        // correct rule; a case-insensitive one would let `Gateway` in.
        let authorizer = AllowedCommonNames::new(["gateway"]);
        assert!(authorizer.authorize(&peer("Gateway"), "/m").await.is_err());
    }

    #[test]
    fn an_unparseable_certificate_is_not_an_identity() {
        let err = PeerIdentity::from_der(b"not a certificate").unwrap_err();
        assert_eq!(err.code(), tonic::Code::Unauthenticated);
    }
}
