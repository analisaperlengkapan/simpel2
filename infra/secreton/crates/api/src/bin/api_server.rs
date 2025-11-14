use axum::serve;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{error, info, warn};

use secreton_api::grpc::server::SecretonGrpcService;
use secreton_api::grpc::tls::GrpcTlsConfig;
use secreton_api::services::ServiceContainer;
use secreton_api::{
    ApiState, KVApiState, KVEngine, PkiApiState, TransitApiState, config::ApiConfig,
    create_api_router,
};
use secreton_crypto::transit::TransitEngine;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    print_startup_banner();

    info!("Loading configuration...");
    // Load configuration with hierarchy: default.toml → production.toml → env vars
    // This provides better security and auditability for Secret Management service
    let config = ApiConfig::load().map_err(|e| {
        error!("Failed to load configuration: {}", e);
        std::io::Error::new(std::io::ErrorKind::InvalidInput, e)
    })?;

    info!("Initializing service container...");
    // Create service container (includes SealService, storage, crypto, etc.)
    let services = Arc::new(ServiceContainer::new(&config).await?);

    // CRITICAL SECURITY: Check seal status at startup
    if services.seal.is_sealed().await {
        warn!("🔒 Vault is SEALED at startup");
        warn!("   All secret operations will be blocked until vault is unsealed");
        warn!("   Use /v1/sys/unseal endpoint with threshold shares to unseal");
    } else {
        info!("🔓 Vault is UNSEALED at startup");
        info!("   Secret operations are allowed");
    }

    info!("Creating transit engine...");
    // Create transit engine (shared between REST and gRPC)
    let transit_engine = Arc::new(TransitEngine::new());

    info!("Creating KV engine...");
    // Create KV engine (shared between REST and gRPC)
    let kv_engine = Arc::new(KVEngine::new());

    info!("Creating API state...");
    // Create API state for REST server (includes ServiceContainer)
    let api_state = ApiState {
        transit: TransitApiState {
            engine: Arc::clone(&transit_engine),
        },
        kv: KVApiState { engine: kv_engine },
        pki: PkiApiState::default(),
        services: Arc::clone(&services),
    };

    info!("Creating router...");
    // Create REST router
    let app = create_api_router(api_state);

    let http_addr: SocketAddr = config.http.bind_address;
    let grpc_addr: SocketAddr = config.grpc.bind_address;
    let tls_config_opt = config.tls.clone();
    let grpc_enabled = config.grpc.enabled;

    // Create gRPC service (shared state with REST)
    let grpc_service =
        SecretonGrpcService::new(services.storage.clone(), Arc::clone(&transit_engine));

    // Start both servers concurrently
    info!("🚀 Starting Secreton servers...");
    info!("   REST API: {}", http_addr);
    info!("   gRPC API: {}", grpc_addr);

    let rest_handle = if let Some(tls_cfg) = tls_config_opt.clone() {
        info!("🔐 REST server with TLS on https://{}", http_addr);
        tokio::spawn(async move {
            if let Err(e) = serve_rest_with_tls(http_addr, app, tls_cfg).await {
                error!("REST server error: {}", e);
            }
        })
    } else {
        info!("⚠️  REST server without TLS on http://{}", http_addr);
        tokio::spawn(async move {
            if let Err(e) = serve_rest(http_addr, app).await {
                error!("REST server error: {}", e);
            }
        })
    };

    let grpc_handle = if grpc_enabled {
        if let Some(tls_cfg) = tls_config_opt {
            info!("🔐 gRPC server with mTLS on https://{}", grpc_addr);
            let mut grpc_tls_config =
                GrpcTlsConfig::new(tls_cfg.cert_file.clone(), tls_cfg.key_file.clone());

            if let Some(ca_file) = &tls_cfg.ca_file {
                grpc_tls_config = grpc_tls_config
                    .with_ca_cert(ca_file.clone())
                    .with_client_auth();
            }

            tokio::spawn(async move {
                if let Err(e) = grpc_service
                    .serve_with_tls(grpc_addr, grpc_tls_config)
                    .await
                {
                    error!("gRPC server error: {}", e);
                }
            })
        } else {
            warn!("⚠️  gRPC server without TLS on http://{}", grpc_addr);
            tokio::spawn(async move {
                if let Err(e) = grpc_service.serve(grpc_addr).await {
                    error!("gRPC server error: {}", e);
                }
            })
        }
    } else {
        info!("gRPC server disabled in configuration");
        tokio::spawn(async {
            // No-op task when gRPC is disabled
            tokio::time::sleep(tokio::time::Duration::from_secs(u64::MAX)).await;
        })
    };

    // Wait for both servers (if one fails, shutdown both)
    tokio::select! {
        rest_result = rest_handle => {
            match rest_result {
                Ok(()) => info!("REST server shutdown gracefully"),
                Err(e) => error!("REST server task panicked: {}", e),
            }
        }
        grpc_result = grpc_handle => {
            match grpc_result {
                Ok(()) => info!("gRPC server shutdown gracefully"),
                Err(e) => error!("gRPC server task panicked: {}", e),
            }
        }
    }

    info!("Secreton servers stopped");
    Ok(())
}

async fn serve_rest(addr: SocketAddr, app: axum::Router) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(addr).await?;
    serve(listener, app).await.map_err(|e| {
        error!("REST server error: {}", e);
        e
    })?;
    Ok(())
}

async fn serve_rest_with_tls(
    addr: SocketAddr,
    app: axum::Router,
    tls_config: secreton_api::config::TlsConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    use axum_server::tls_rustls::RustlsConfig;
    use rustls::pki_types::CertificateDer;
    use rustls_pemfile::certs;
    use std::fs;
    use std::io::BufReader;

    info!("Loading TLS certificates for REST server...");

    // Load server certificate
    let cert_file = fs::File::open(&tls_config.cert_file)?;
    let mut cert_reader = BufReader::new(cert_file);
    let cert_chain: Vec<CertificateDer<'static>> =
        certs(&mut cert_reader).collect::<Result<Vec<_>, _>>()?;

    if cert_chain.is_empty() {
        return Err("No certificates found in cert file".into());
    }

    // Load private key
    let key_file = fs::File::open(&tls_config.key_file)?;
    let mut key_reader = BufReader::new(key_file);
    let mut keys: Vec<rustls::pki_types::PrivatePkcs8KeyDer> =
        rustls_pemfile::pkcs8_private_keys(&mut key_reader).collect::<Result<Vec<_>, _>>()?;

    if keys.is_empty() {
        return Err("No private keys found in key file".into());
    }

    // Create TLS configuration
    let server_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(
            cert_chain,
            rustls::pki_types::PrivateKeyDer::Pkcs8(keys.remove(0)),
        )?;

    info!("Binding REST server to address...");
    let listener = std::net::TcpListener::bind(addr)?;
    listener.set_nonblocking(true)?;

    info!("✅ REST API server ready on https://{}", addr);
    info!("   📋 Health check: https://{}/health", addr);
    info!("   📋 Version info: https://{}/version", addr);
    info!("   🔐 Transit API: https://{}/v1/transit", addr);
    info!("   🗄️  KV Secrets API: https://{}/v1/secret", addr);

    // Start server with TLS
    let rustls_config = RustlsConfig::from_config(Arc::new(server_config));
    axum_server::from_tcp_rustls(listener, rustls_config)
        .serve(app.into_make_service())
        .await
        .map_err(|e| {
            error!("REST server error: {}", e);
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
    println!("║  • REST API (HTTP/HTTPS)                                    ║");
    println!("║  • gRPC API (with mTLS support)                             ║");
    println!("║  • TLS 1.3 Support                                          ║");
    println!("║  • High-Performance Async Operations                        ║");
    println!("║                                                              ║");
    println!("║  Security: Zero-Trust, MFA, RBAC, Audit Trails              ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
}
