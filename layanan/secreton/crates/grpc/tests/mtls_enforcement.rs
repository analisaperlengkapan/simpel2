//! Does the gRPC boundary actually refuse an unauthenticated caller?
//!
//! These tests run a real `SecretonGrpcService` over a real TLS listener and
//! connect with a real tonic client, because that is the only way to answer
//! the question. A unit test cannot: `TlsConnectInfo` — the extension carrying
//! the peer certificate — has private fields and no constructor, so a
//! fabricated request can never prove what a handshake does.
//!
//! The certificates are minted here at runtime rather than committed as
//! fixtures: a checked-in cert expires one day and turns a security test into
//! a flake that someone deletes.
//!
//! Reference point for every assertion below: a call that gets *through* the
//! layer reaches `get_secret`, which answers `NOT_FOUND` for a path that was
//! never written. So `NOT_FOUND` means "authenticated and dispatched", and
//! anything else means the boundary stopped it.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use rcgen::{CertificateParams, DistinguishedName, DnType, IsCa, KeyPair};
use secreton_crypto::transit::TransitEngine;
use secreton_grpc::generated::secreton::v1::GetSecretRequest;
use secreton_grpc::generated::secreton::v1::secreton_service_client::SecretonServiceClient;
use secreton_grpc::{
    AllowedCommonNames, AnyTrustedPeer, GrpcAuthorization, GrpcTlsConfig, PeerAuthorizer,
    SecretonGrpcService,
};
use secreton_storage::MemoryBackend;
use tempfile::TempDir;
use tonic::Code;
use tonic::transport::{Certificate, Channel, ClientTlsConfig, Identity};

/// A minimal certificate authority, plus the leaves it signs.
struct TestCa {
    cert_pem: String,
    params: CertificateParams,
    key: KeyPair,
}

struct Leaf {
    cert_pem: String,
    key_pem: String,
}

impl TestCa {
    fn new(name: &str) -> Self {
        let mut dn = DistinguishedName::new();
        dn.push(DnType::CommonName, name);

        let mut params = CertificateParams::default();
        params.distinguished_name = dn;
        params.is_ca = IsCa::Ca(rcgen::BasicConstraints::Unconstrained);

        let key = KeyPair::generate().expect("generate CA key");
        let cert = params.self_signed(&key).expect("self-sign CA");

        Self {
            cert_pem: cert.pem(),
            params,
            key,
        }
    }

    /// Issue a leaf certificate with `common_name`, valid for `sans`.
    fn issue(&self, common_name: &str, sans: Vec<String>) -> Leaf {
        let mut dn = DistinguishedName::new();
        dn.push(DnType::CommonName, common_name);

        let mut params = CertificateParams::new(sans).expect("valid SANs");
        params.distinguished_name = dn;
        params.is_ca = IsCa::NoCa;

        let issuer = rcgen::Issuer::from_params(&self.params, &self.key);
        let key = KeyPair::generate().expect("generate leaf key");
        let cert = params.signed_by(&key, &issuer).expect("sign leaf");

        Leaf {
            cert_pem: cert.pem(),
            key_pem: key.serialize_pem(),
        }
    }
}

fn write(dir: &TempDir, name: &str, contents: &str) -> std::path::PathBuf {
    let path = dir.path().join(name);
    std::fs::write(&path, contents).expect("write PEM");
    path
}

/// Start the real service behind mTLS on an ephemeral port.
async fn serve_with_mtls(
    ca: &TestCa,
    server: &Leaf,
    authorizer: Arc<dyn PeerAuthorizer>,
    dir: &TempDir,
) -> SocketAddr {
    let cert = write(dir, "server.crt", &server.cert_pem);
    let key = write(dir, "server.key", &server.key_pem);
    let ca_path = write(dir, "ca.crt", &ca.cert_pem);

    // Bind first so the test knows the port, then hand the address to tonic.
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
    let addr = listener.local_addr().expect("read bound address");
    drop(listener);

    // Authorization is deliberately opted out here, because this suite is about
    // the *transport*: which peers complete a handshake and reach a handler at
    // all. Enforcing policy on top would mean every assertion below could be
    // satisfied by a policy denial instead of a TLS rejection, and the two are
    // exactly what this file exists to tell apart. Per-path authorization has
    // its own suite in `grpc_authorization.rs`.
    let service = SecretonGrpcService::new(
        Arc::new(MemoryBackend::new()),
        Arc::new(TransitEngine::new()),
        None,
        GrpcAuthorization::AnonymousOptOut {
            reason: "transport-layer test: policy enforcement covered separately".to_string(),
        },
    );

    let tls = GrpcTlsConfig::new(cert, key)
        .with_ca_cert(ca_path)
        .with_client_auth();

    tokio::spawn(async move {
        // Errors here surface as connection failures in the test body.
        let _ = service.serve_with_mtls(addr, tls, authorizer).await;
    });

    // Wait for the listener to accept connections rather than sleeping blind.
    for _ in 0..100 {
        if std::net::TcpStream::connect(addr).is_ok() {
            return addr;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("gRPC server never started listening on {addr}");
}

/// Connect a client, optionally presenting a client certificate.
async fn connect(
    addr: SocketAddr,
    ca_pem: &str,
    client: Option<&Leaf>,
) -> Result<Channel, tonic::transport::Error> {
    let mut tls = ClientTlsConfig::new()
        .ca_certificate(Certificate::from_pem(ca_pem))
        .domain_name("localhost");

    if let Some(leaf) = client {
        tls = tls.identity(Identity::from_pem(&leaf.cert_pem, &leaf.key_pem));
    }

    Channel::from_shared(format!("https://{addr}"))
        .expect("valid endpoint")
        .tls_config(tls)?
        .connect()
        .await
}

async fn call_get_secret(channel: Channel) -> Result<(), tonic::Status> {
    SecretonServiceClient::new(channel)
        .get_secret(GetSecretRequest {
            path: "kv/never/written".to_string(),
            version: None,
        })
        .await
        .map(|_| ())
}

#[tokio::test]
async fn a_client_with_a_ca_signed_certificate_reaches_the_handler() {
    let dir = TempDir::new().unwrap();
    let ca = TestCa::new("simpel-test-ca");
    let server = ca.issue("secreton", vec!["localhost".into()]);
    let client = ca.issue("gateway", vec!["gateway".into()]);

    let addr = serve_with_mtls(&ca, &server, Arc::new(AnyTrustedPeer), &dir).await;
    let channel = connect(addr, &ca.cert_pem, Some(&client))
        .await
        .expect("a CA-signed client must complete the handshake");

    let status = call_get_secret(channel)
        .await
        .expect_err("the secret was never written, so the handler answers NOT_FOUND");

    assert_eq!(
        status.code(),
        Code::NotFound,
        "an authenticated call must be dispatched to the handler, got: {status:?}"
    );
}

#[tokio::test]
async fn a_client_with_no_certificate_never_gets_in() {
    let dir = TempDir::new().unwrap();
    let ca = TestCa::new("simpel-test-ca");
    let server = ca.issue("secreton", vec!["localhost".into()]);

    let addr = serve_with_mtls(&ca, &server, Arc::new(AnyTrustedPeer), &dir).await;

    // Trusts the server, presents nothing of its own — the shape of every
    // caller that could read the whole engine before this change.
    let outcome = match connect(addr, &ca.cert_pem, None).await {
        Err(_) => None,                                      // refused at the handshake
        Ok(channel) => Some(call_get_secret(channel).await), // or at the layer
    };

    match outcome {
        None => {}
        Some(Err(status)) => assert_ne!(
            status.code(),
            Code::NotFound,
            "an anonymous caller reached the handler — the boundary is open"
        ),
        Some(Ok(())) => panic!("an anonymous caller read from the engine"),
    }
}

#[tokio::test]
async fn a_certificate_from_another_ca_is_not_a_certificate() {
    let dir = TempDir::new().unwrap();
    let ca = TestCa::new("simpel-test-ca");
    let server = ca.issue("secreton", vec!["localhost".into()]);

    // Same common name as a legitimate workload, signed by a CA we never
    // trusted. Name equality must not be identity.
    let rogue_ca = TestCa::new("rogue-ca");
    let rogue = rogue_ca.issue("gateway", vec!["gateway".into()]);

    let addr = serve_with_mtls(&ca, &server, Arc::new(AnyTrustedPeer), &dir).await;

    let outcome = match connect(addr, &ca.cert_pem, Some(&rogue)).await {
        Err(_) => None,
        Ok(channel) => Some(call_get_secret(channel).await),
    };

    match outcome {
        None => {}
        Some(Err(status)) => assert_ne!(
            status.code(),
            Code::NotFound,
            "a certificate from an untrusted CA reached the handler"
        ),
        Some(Ok(())) => panic!("a certificate from an untrusted CA read from the engine"),
    }
}

#[tokio::test]
async fn an_authenticated_peer_outside_the_allowlist_is_denied() {
    let dir = TempDir::new().unwrap();
    let ca = TestCa::new("simpel-test-ca");
    let server = ca.issue("secreton", vec!["localhost".into()]);
    // Chain-valid, so the handshake succeeds; the layer is what must refuse it.
    let stranger = ca.issue("some-other-workload", vec!["stranger".into()]);

    let authorizer = Arc::new(AllowedCommonNames::new(["gateway"]));
    let addr = serve_with_mtls(&ca, &server, authorizer, &dir).await;

    let channel = connect(addr, &ca.cert_pem, Some(&stranger))
        .await
        .expect("the certificate chains to the trusted CA, so TLS succeeds");

    let status = call_get_secret(channel)
        .await
        .expect_err("an unlisted workload must be refused");

    assert_eq!(
        status.code(),
        Code::PermissionDenied,
        "expected the authorizer to deny, got: {status:?}"
    );
}
