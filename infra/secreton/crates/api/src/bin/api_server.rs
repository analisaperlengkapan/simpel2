use axum::serve;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{error, info, warn};

use secreton_api::{create_api_router, ApiState, KVApiState, TransitApiState, config::ApiConfig, KVEngine};
use secreton_crypto::transit::TransitEngine;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    print_startup_banner();

    info!("Creating transit engine...");
    // Create transit engine
    let transit_engine = Arc::new(TransitEngine::new());

    info!("Creating KV engine...");
    // Create KV engine
    let kv_engine = Arc::new(KVEngine::new());

    info!("Creating API state...");
    // Create API state
    let api_state = ApiState {
        transit: TransitApiState {
            engine: transit_engine,
        },
        kv: KVApiState { engine: kv_engine },
    };

    info!("Creating router...");
    // Create router
    let app = create_api_router(api_state);

    info!("Loading configuration...");
    // Load configuration
    let config = ApiConfig::default();
    let addr: SocketAddr = config.http.bind_address;

    // Check if TLS is configured
    if let Some(tls_config) = &config.tls {
        info!(
            "🔐 Starting Secreton API server with TLS on https://{}",
            addr
        );
        serve_with_tls(addr, app, tls_config).await?;
    } else {
        info!("Starting Secreton API server on http://{}", addr);
        let listener = TcpListener::bind(addr).await?;
        serve(listener, app).await.map_err(|e| {
            error!("Server error: {}", e);
            e
        })?;
    }

    Ok(())
}

async fn serve_with_tls(
    addr: SocketAddr,
    app: axum::Router,
    tls_config: &secreton_api::config::TlsConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    use axum_server::tls_rustls::RustlsConfig;
    use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
    use rustls_pemfile::{certs, pkcs8_private_keys};
    use std::fs;
    use std::io::BufReader;

    info!("Loading TLS certificates...");

    // Load server certificate
    let cert_file = fs::File::open(&tls_config.cert_file)?;
    let mut cert_reader = BufReader::new(cert_file);
    let cert_chain: Vec<CertificateDer<'static>> = certs(&mut cert_reader)
        .collect::<Result<Vec<_>, _>>()?;

    if cert_chain.is_empty() {
        return Err("No certificates found in cert file".into());
    }

    // Load private key
    let key_file = fs::File::open(&tls_config.key_file)?;
    let mut key_reader = BufReader::new(key_file);
    let mut keys: Vec<rustls::pki_types::PrivatePkcs8KeyDer> = rustls_pemfile::pkcs8_private_keys(&mut key_reader)
        .collect::<Result<Vec<_>, _>>()?;

    if keys.is_empty() {
        return Err("No private keys found in key file".into());
    }

    // Create TLS configuration
    let mut server_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert_chain, rustls::pki_types::PrivateKeyDer::Pkcs8(keys.remove(0)))?;

    info!("Binding to address...");
    let listener = std::net::TcpListener::bind(addr)?;
    listener.set_nonblocking(true)?;

    info!("🚀 Secreton API server starting on https://{}", addr);
    info!("📋 Health check: https://{}/health", addr);
    info!("📋 Version info: https://{}/version", addr);
    info!("🔐 Transit API: https://{}/v1/transit", addr);
    info!("🗄️  KV Secrets API: https://{}/v1/secrets", addr);

    // Start server with TLS
    let rustls_config = RustlsConfig::from_config(Arc::new(server_config));
    axum_server::from_tcp_rustls(listener, rustls_config)
        .serve(app.into_make_service())
        .await
        .map_err(|e| {
            error!("Server error: {}", e);
            e
        })?;

    Ok(())
}

fn print_startup_banner() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                      🔐 Secreton API Server                     ║");
    println!("║                 Custom Rust Vault Implementation               ║");
    println!("║                                                              ║");
    println!("║  Features:                                                   ║");
    println!("║  • Transit Engine (Encryption/Decryption)                   ║");
    println!("║  • KV Secrets Engine (Key-Value Storage)                    ║");
    println!("║  • TLS 1.3 Support                                          ║");
    println!("║  • High-Performance Async Operations                        ║");
    println!("║                                                              ║");
    println!("║  Security: Zero-Trust, MFA, RBAC, Audit Trails              ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
}
