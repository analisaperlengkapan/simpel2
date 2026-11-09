//! A binary whose dependency graph carries BOTH rustls providers must still be able
//! to build a client TLS config.
//!
//! This is its own test binary on purpose: rustls' default provider is process-wide,
//! so any other test that installed one first would make this one pass for the wrong
//! reason. It starts from a process in which nothing has installed a provider and
//! asserts exactly that before acting.
//!
//! The dev-dependency `rustls = { features = ["ring"] }` in Cargo.toml recreates the
//! shipped binaries' graph (aws_lc_rs from reqwest/tonic, ring from lettre). In that graph
//! `ClientConfig::builder()` panics unless a provider is installed first — which is what
//! crashed `authenc` and `layanan-perlengkapan` in staging the first time mTLS was on.

#![cfg(feature = "grpc")]

use lib_backend::grpc::client::{ClientTls, endpoint};
use rcgen::{BasicConstraints, CertificateParams, DistinguishedName, DnType, IsCa, KeyPair};

#[test]
fn client_tls_endpoint_builds_when_two_rustls_providers_are_linked() {
    assert!(
        rustls::crypto::CryptoProvider::get_default().is_none(),
        "something installed a provider before this test, so it would prove nothing"
    );

    let mut dn = DistinguishedName::new();
    dn.push(DnType::CommonName, "test-ca");
    let mut ca_params = CertificateParams::default();
    ca_params.distinguished_name = dn;
    ca_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    let ca_key = KeyPair::generate().expect("CA key");
    let ca = ca_params.self_signed(&ca_key).expect("self-sign CA");

    let leaf_key = KeyPair::generate().expect("leaf key");
    let leaf_params = CertificateParams::new(vec!["client.test".into()]).expect("SANs");
    let issuer = rcgen::Issuer::from_params(&ca_params, &ca_key);
    let leaf = leaf_params
        .signed_by(&leaf_key, &issuer)
        .expect("sign leaf");

    let dir = tempfile::tempdir().expect("tempdir");
    let write = |name: &str, body: String| {
        let p = dir.path().join(name);
        std::fs::write(&p, body).expect("write PEM");
        p
    };
    let tls = ClientTls::Enabled {
        ca: write("ca.pem", ca.pem()),
        cert: write("tls.crt", leaf.pem()),
        key: write("tls.key", leaf_key.serialize_pem()),
    };

    // The panic in staging happened inside this call (tonic builds the rustls config here).
    endpoint("https://peer.test:50051", &tls).expect("TLS endpoint must build");

    assert!(
        rustls::crypto::CryptoProvider::get_default().is_some(),
        "endpoint() must leave a process-wide provider installed"
    );
}
