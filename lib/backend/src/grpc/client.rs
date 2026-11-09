//! mTLS for gRPC **clients**.
//!
//! The server half already existed (`secreton-grpc` and `authenc-grpc` load
//! `GRPC_TLS_*` and can require a client certificate). The client half did not:
//! gateway, layanan-perlengkapan and authenc's federation client all dialled
//! `http://…` with no credentials. So the moment a server was switched to mTLS
//! — which production requires, authenc and secreton refuse to start without
//! it — every caller of it would have been shut out. This module is the missing
//! half, shared so the three callers cannot disagree about the rules.
//!
//! # Configuration (environment)
//!
//! | variable                | meaning                                              |
//! |-------------------------|------------------------------------------------------|
//! | `GRPC_CLIENT_CA_PATH`   | CA that signed the SERVERS this process dials (PEM)  |
//! | `GRPC_CLIENT_CERT_PATH` | this workload's client certificate (PEM)             |
//! | `GRPC_CLIENT_KEY_PATH`  | its private key (PEM)                                |
//!
//! # Rules (all fail CLOSED)
//!
//! * none of the three set  → plaintext; only `http://` URLs are accepted;
//! * all three set          → mTLS; only `https://` URLs are accepted;
//! * ANY OTHER COMBINATION  → error. A half-configured client that quietly falls
//!   back to plaintext is the worst outcome: it looks secured and is not.
//! * `http://` with TLS material, or `https://` without it → error. Neither is
//!   silently "fixed": a URL and a credential set that disagree mean the
//!   deployment is wrong, and the operator needs to hear that at startup.
//!
//! Server certificates are verified against the URL's host, so a server's
//! certificate must carry that exact DNS name in its SAN list.
//!
//! The files are read once, when the channel is built; rotate by restarting.

use std::fmt;
use std::path::PathBuf;

use tonic::transport::{Certificate, ClientTlsConfig, Endpoint, Identity};

/// Environment variable naming the CA that signed the servers.
pub const CA_ENV: &str = "GRPC_CLIENT_CA_PATH";
/// Environment variable naming this workload's client certificate.
pub const CERT_ENV: &str = "GRPC_CLIENT_CERT_PATH";
/// Environment variable naming this workload's private key.
pub const KEY_ENV: &str = "GRPC_CLIENT_KEY_PATH";

/// Which transport a client was configured for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientTls {
    /// No TLS material: plaintext, `http://` only.
    Disabled,
    /// Mutual TLS, `https://` only.
    Enabled {
        ca: PathBuf,
        cert: PathBuf,
        key: PathBuf,
    },
}

/// Why a client channel could not be configured.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientTlsError {
    /// Some, but not all, of the three variables are set.
    Partial {
        set: Vec<&'static str>,
        missing: Vec<&'static str>,
    },
    /// TLS material is configured but the URL is `http://`.
    PlaintextUrlWithTls(String),
    /// The URL is `https://` but no TLS material is configured.
    TlsUrlWithoutMaterial(String),
    /// The URL is neither `http://` nor `https://`.
    UnsupportedScheme(String),
    /// A PEM file could not be read.
    Unreadable { path: PathBuf, reason: String },
    /// tonic rejected the endpoint or the TLS configuration.
    Transport(String),
}

impl fmt::Display for ClientTlsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Partial { set, missing } => write!(
                f,
                "gRPC client mTLS is half-configured: {} set but {} missing — set all three \
                 or none (a partial setting must not silently fall back to plaintext)",
                set.join(", "),
                missing.join(", ")
            ),
            Self::PlaintextUrlWithTls(url) => write!(
                f,
                "gRPC client TLS material is configured but `{url}` is http:// — refusing to \
                 downgrade; use an https:// URL"
            ),
            Self::TlsUrlWithoutMaterial(url) => write!(
                f,
                "`{url}` is https:// but {CA_ENV}/{CERT_ENV}/{KEY_ENV} are not set — this \
                 process has no credentials to present"
            ),
            Self::UnsupportedScheme(url) => {
                write!(f, "`{url}` must start with http:// or https://")
            }
            Self::Unreadable { path, reason } => {
                write!(f, "cannot read {}: {reason}", path.display())
            }
            Self::Transport(e) => write!(f, "invalid gRPC endpoint or TLS configuration: {e}"),
        }
    }
}

impl std::error::Error for ClientTlsError {}

/// Resolve the three variables through `get`. Taking a lookup function (instead
/// of reading the process environment directly) keeps the rules unit-testable
/// without mutating global state.
pub fn client_tls_from(get: impl Fn(&str) -> Option<String>) -> Result<ClientTls, ClientTlsError> {
    let read = |name: &'static str| get(name).filter(|v| !v.trim().is_empty());
    let (ca, cert, key) = (read(CA_ENV), read(CERT_ENV), read(KEY_ENV));
    match (ca, cert, key) {
        (None, None, None) => Ok(ClientTls::Disabled),
        (Some(ca), Some(cert), Some(key)) => Ok(ClientTls::Enabled {
            ca: ca.into(),
            cert: cert.into(),
            key: key.into(),
        }),
        (ca, cert, key) => {
            let mut set = Vec::new();
            let mut missing = Vec::new();
            for (name, v) in [(CA_ENV, &ca), (CERT_ENV, &cert), (KEY_ENV, &key)] {
                if v.is_some() {
                    set.push(name)
                } else {
                    missing.push(name)
                }
            }
            Err(ClientTlsError::Partial { set, missing })
        }
    }
}

/// [`client_tls_from`] over the real process environment.
pub fn client_tls_from_env() -> Result<ClientTls, ClientTlsError> {
    client_tls_from(|k| std::env::var(k).ok())
}

fn read_pem(path: &PathBuf) -> Result<Vec<u8>, ClientTlsError> {
    std::fs::read(path).map_err(|e| ClientTlsError::Unreadable {
        path: path.clone(),
        reason: e.to_string(),
    })
}

/// Build a tonic [`Endpoint`] for `url` under the given transport settings.
///
/// Call `.connect_lazy()` or `.connect().await` on the result.
pub fn endpoint(url: &str, tls: &ClientTls) -> Result<Endpoint, ClientTlsError> {
    let is_https = url.starts_with("https://");
    if !is_https && !url.starts_with("http://") {
        return Err(ClientTlsError::UnsupportedScheme(url.to_string()));
    }
    let ep = Endpoint::from_shared(url.to_string())
        .map_err(|e| ClientTlsError::Transport(e.to_string()))?;
    match (tls, is_https) {
        (ClientTls::Disabled, false) => Ok(ep),
        (ClientTls::Disabled, true) => Err(ClientTlsError::TlsUrlWithoutMaterial(url.to_string())),
        (ClientTls::Enabled { .. }, false) => {
            Err(ClientTlsError::PlaintextUrlWithTls(url.to_string()))
        }
        (ClientTls::Enabled { ca, cert, key }, true) => {
            // Before the first rustls config in the process — see the function's docs.
            super::ensure_crypto_provider();
            let cfg = ClientTlsConfig::new()
                .ca_certificate(Certificate::from_pem(read_pem(ca)?))
                .identity(Identity::from_pem(read_pem(cert)?, read_pem(key)?));
            ep.tls_config(cfg)
                .map_err(|e| ClientTlsError::Transport(e.to_string()))
        }
    }
}

/// [`endpoint`] using the settings in the process environment.
pub fn endpoint_from_env(url: &str) -> Result<Endpoint, ClientTlsError> {
    endpoint(url, &client_tls_from_env()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn env(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let m: HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        move |k| m.get(k).cloned()
    }

    #[test]
    fn nothing_set_means_plaintext() {
        assert_eq!(client_tls_from(env(&[])).unwrap(), ClientTls::Disabled);
    }

    #[test]
    fn blank_values_count_as_unset() {
        let t = client_tls_from(env(&[(CA_ENV, "  "), (CERT_ENV, ""), (KEY_ENV, "")])).unwrap();
        assert_eq!(t, ClientTls::Disabled);
    }

    #[test]
    fn all_three_set_means_mtls() {
        let t =
            client_tls_from(env(&[(CA_ENV, "/ca"), (CERT_ENV, "/c"), (KEY_ENV, "/k")])).unwrap();
        assert!(matches!(t, ClientTls::Enabled { .. }));
    }

    #[test]
    fn every_partial_combination_is_an_error() {
        let all = [(CA_ENV, "/ca"), (CERT_ENV, "/c"), (KEY_ENV, "/k")];
        for mask in 1u8..7 {
            let pairs: Vec<_> = all
                .iter()
                .enumerate()
                .filter(|(i, _)| mask & (1 << i) != 0)
                .map(|(_, p)| *p)
                .collect();
            let err = client_tls_from(env(&pairs)).expect_err(&format!("mask {mask:03b}"));
            assert!(matches!(err, ClientTlsError::Partial { .. }), "{err}");
        }
    }

    #[test]
    fn partial_error_names_what_is_missing() {
        let err = client_tls_from(env(&[(CA_ENV, "/ca")]))
            .unwrap_err()
            .to_string();
        assert!(err.contains(CERT_ENV) && err.contains(KEY_ENV), "{err}");
    }

    #[test]
    fn plaintext_url_with_tls_material_is_refused() {
        let tls = ClientTls::Enabled {
            ca: "/a".into(),
            cert: "/b".into(),
            key: "/c".into(),
        };
        assert_eq!(
            endpoint("http://authenc:9088", &tls).unwrap_err(),
            ClientTlsError::PlaintextUrlWithTls("http://authenc:9088".into())
        );
    }

    #[test]
    fn tls_url_without_material_is_refused() {
        assert_eq!(
            endpoint("https://authenc:9088", &ClientTls::Disabled).unwrap_err(),
            ClientTlsError::TlsUrlWithoutMaterial("https://authenc:9088".into())
        );
    }

    #[test]
    fn unknown_scheme_is_refused() {
        assert!(matches!(
            endpoint("authenc:9088", &ClientTls::Disabled).unwrap_err(),
            ClientTlsError::UnsupportedScheme(_)
        ));
    }

    #[test]
    fn unreadable_pem_names_the_path() {
        let tls = ClientTls::Enabled {
            ca: "/nonexistent/ca.pem".into(),
            cert: "/nonexistent/c.pem".into(),
            key: "/nonexistent/k.pem".into(),
        };
        let err = endpoint("https://authenc:9088", &tls)
            .unwrap_err()
            .to_string();
        assert!(err.contains("/nonexistent/ca.pem"), "{err}");
    }

    #[test]
    fn plaintext_endpoint_builds() {
        assert!(endpoint("http://authenc:9088", &ClientTls::Disabled).is_ok());
    }
}
