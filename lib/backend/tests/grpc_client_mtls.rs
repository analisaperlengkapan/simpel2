//! Does the client helper complete a REAL mutual-TLS handshake — and does a
//! server that demands client authentication actually refuse a client that
//! doesn't have it?
//!
//! `grpc::client`'s unit tests prove the configuration rules. They cannot prove
//! that the `ClientTlsConfig` it builds is accepted by a server, which is the
//! only thing that matters. So these tests stand up a real tonic server with
//! `client_ca_root` (client authentication REQUIRED) on an ephemeral port and
//! dial it through `endpoint()`.
//!
//! `Endpoint::connect()` is NOT proof, in either direction, and an earlier draft
//! of this file that trusted it passed two tests it should have failed: under
//! TLS 1.3 the client considers the handshake finished as soon as it has sent
//! its own Finished message, so a server that is about to reject the client
//! certificate still lets `connect()` return `Ok` (the rejection arrives as an
//! alert afterwards); and HTTP/2 prior-knowledge "connects" to a TLS port simply
//! by writing a preface nobody reads. Only a real RPC distinguishes them, so
//! each test makes one. The service answers `UNIMPLEMENTED`, which means "the
//! call reached the handler"; any transport failure means it did not.

#![cfg(feature = "grpc")]

use std::convert::Infallible;
use std::future::{Ready, ready};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::task::{Context, Poll};
use std::time::Duration;

use lib_backend::grpc::client::{ClientTls, endpoint};
use rcgen::{BasicConstraints, CertificateParams, DistinguishedName, DnType, IsCa, KeyPair};
use tempfile::TempDir;
use tonic::codegen::{Service, http};
use tonic::server::NamedService;
use tonic::transport::{Certificate, Identity, Server, ServerTlsConfig};

/// A gRPC service that does nothing: the tests are about the handshake.
#[derive(Clone)]
struct Nop;

impl NamedService for Nop {
    const NAME: &'static str = "test.Nop";
}

impl Service<http::Request<tonic::body::Body>> for Nop {
    type Response = http::Response<tonic::body::Body>;
    type Error = Infallible;
    type Future = Ready<Result<Self::Response, Infallible>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Infallible>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _: http::Request<tonic::body::Body>) -> Self::Future {
        ready(Ok(tonic::Status::unimplemented("nop").into_http()))
    }
}

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
        params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        let key = KeyPair::generate().expect("CA key");
        let cert = params.self_signed(&key).expect("self-sign CA");
        Self {
            cert_pem: cert.pem(),
            params,
            key,
        }
    }

    fn issue(&self, common_name: &str, sans: Vec<String>) -> Leaf {
        let mut dn = DistinguishedName::new();
        dn.push(DnType::CommonName, common_name);
        let mut params = CertificateParams::new(sans).expect("SANs");
        params.distinguished_name = dn;
        params.is_ca = IsCa::NoCa;
        let issuer = rcgen::Issuer::from_params(&self.params, &self.key);
        let key = KeyPair::generate().expect("leaf key");
        let cert = params.signed_by(&key, &issuer).expect("sign leaf");
        Leaf {
            cert_pem: cert.pem(),
            key_pem: key.serialize_pem(),
        }
    }
}

fn write(dir: &TempDir, name: &str, body: &str) -> PathBuf {
    let p = dir.path().join(name);
    std::fs::write(&p, body).expect("write PEM");
    p
}

/// A server that REQUIRES a client certificate signed by `ca`.
async fn serve(ca: &TestCa, server: &Leaf) -> SocketAddr {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("addr");
    // What a service's startup does. The dev-dependency `rustls/ring` makes this graph
    // ambiguous, and the server task races the client's endpoint() for the first TLS config.
    lib_backend::grpc::ensure_crypto_provider();
    let tls = ServerTlsConfig::new()
        .identity(Identity::from_pem(&server.cert_pem, &server.key_pem))
        .client_ca_root(Certificate::from_pem(&ca.cert_pem));
    tokio::spawn(async move {
        let _ = Server::builder()
            .tls_config(tls)
            .expect("server TLS")
            .add_service(Nop)
            .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener))
            .await;
    });
    addr
}

fn tls_for(dir: &TempDir, ca: &TestCa, client: &Leaf) -> ClientTls {
    ClientTls::Enabled {
        ca: write(dir, "ca.crt", &ca.cert_pem),
        cert: write(dir, "client.crt", &client.cert_pem),
        key: write(dir, "client.key", &client.key_pem),
    }
}

/// True only if a real RPC reached the server's handler (it answers UNIMPLEMENTED).
async fn can_connect(url: &str, tls: &ClientTls) -> bool {
    let ep = endpoint(url, tls)
        .expect("endpoint builds")
        .connect_timeout(Duration::from_secs(3));
    let Ok(channel) = ep.connect().await else {
        return false;
    };
    let mut grpc = tonic::client::Grpc::new(channel);
    if grpc.ready().await.is_err() {
        return false;
    }
    let call = grpc.unary::<(), (), _>(
        tonic::Request::new(()),
        http::uri::PathAndQuery::from_static("/test.Nop/Ping"),
        tonic_prost::ProstCodec::<(), ()>::default(),
    );
    match tokio::time::timeout(Duration::from_secs(3), call).await {
        Ok(Err(status)) => status.code() == tonic::Code::Unimplemented,
        _ => false,
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn a_client_with_a_certificate_from_the_servers_ca_is_accepted() {
    let (dir, ca) = (TempDir::new().unwrap(), TestCa::new("simpel-grpc-ca"));
    let server = ca.issue("secreton", vec!["localhost".into()]);
    let client = ca.issue("gateway", vec!["gateway".into()]);
    let addr = serve(&ca, &server).await;

    let url = format!("https://localhost:{}", addr.port());
    assert!(can_connect(&url, &tls_for(&dir, &ca, &client)).await);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_client_certificate_from_a_different_ca_is_refused() {
    let (dir, ca) = (TempDir::new().unwrap(), TestCa::new("simpel-grpc-ca"));
    let server = ca.issue("secreton", vec!["localhost".into()]);
    let addr = serve(&ca, &server).await;

    // Trusts the server's CA (so server verification passes) but presents a
    // certificate an attacker's CA signed.
    let rogue = TestCa::new("rogue-ca");
    let forged = rogue.issue("gateway", vec!["gateway".into()]);
    let tls = ClientTls::Enabled {
        ca: write(&dir, "ca.crt", &ca.cert_pem),
        cert: write(&dir, "rogue.crt", &forged.cert_pem),
        key: write(&dir, "rogue.key", &forged.key_pem),
    };

    let url = format!("https://localhost:{}", addr.port());
    assert!(!can_connect(&url, &tls).await);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_server_the_client_does_not_trust_is_refused() {
    let (dir, ca) = (TempDir::new().unwrap(), TestCa::new("simpel-grpc-ca"));
    let server = ca.issue("secreton", vec!["localhost".into()]);
    let client = ca.issue("gateway", vec!["gateway".into()]);
    let addr = serve(&ca, &server).await;

    // The client trusts a DIFFERENT CA, so it must not accept this server.
    let other = TestCa::new("other-ca");
    let tls = ClientTls::Enabled {
        ca: write(&dir, "other-ca.crt", &other.cert_pem),
        cert: write(&dir, "client.crt", &client.cert_pem),
        key: write(&dir, "client.key", &client.key_pem),
    };

    let url = format!("https://localhost:{}", addr.port());
    assert!(!can_connect(&url, &tls).await);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_server_certificate_for_another_name_is_refused() {
    let (dir, ca) = (TempDir::new().unwrap(), TestCa::new("simpel-grpc-ca"));
    // Valid chain, wrong identity: the cert says it is `secreton.example`.
    let server = ca.issue("secreton", vec!["secreton.example".into()]);
    let client = ca.issue("gateway", vec!["gateway".into()]);
    let addr = serve(&ca, &server).await;

    let url = format!("https://localhost:{}", addr.port());
    assert!(!can_connect(&url, &tls_for(&dir, &ca, &client)).await);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_plaintext_client_cannot_talk_to_an_mtls_server() {
    let (_dir, ca) = (TempDir::new().unwrap(), TestCa::new("simpel-grpc-ca"));
    let server = ca.issue("secreton", vec!["localhost".into()]);
    let addr = serve(&ca, &server).await;

    let url = format!("http://localhost:{}", addr.port());
    assert!(!can_connect(&url, &ClientTls::Disabled).await);
}
