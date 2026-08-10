use axum::serve;
use metrics_exporter_prometheus::PrometheusBuilder;
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

    info!("Loading bootstrap configuration...");
    // Load bootstrap config (infrastructure only, no secrets)
    let bootstrap_config_path =
        std::env::var("SECRETON_CONFIG").unwrap_or_else(|_| "secreton.toml".to_string());

    let bootstrap = secreton_core::config::BootstrapConfig::from_file(std::path::Path::new(
        &bootstrap_config_path,
    ))
    .map_err(|e| {
        error!(
            "Failed to load bootstrap config from {}: {}",
            bootstrap_config_path, e
        );
        std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string())
    })?;

    info!("Bootstrap config loaded successfully");
    info!("  Storage backend: {:?}", bootstrap.storage.backend);
    info!("  HTTP listener: {}", bootstrap.http.address);
    info!("  gRPC listener: {}", bootstrap.grpc.address);

    info!("Initializing service container with bootstrap config...");
    // Create service container (includes SealService, storage, crypto, etc.)
    let services = Arc::new(ServiceContainer::new_from_bootstrap(&bootstrap).await?);

    // CRITICAL SECURITY: Check seal status at startup
    if services.seal.is_sealed().await {
        warn!("🔒 Engine is SEALED at startup");
        warn!("   All secret operations will be blocked until engine is unsealed");
        warn!("   Use /v1/sys/unseal endpoint with threshold shares to unseal");
        warn!("   Application config will be loaded after unsealing");
    } else {
        info!("🔓 Engine is UNSEALED at startup");
        info!("   Loading application config from encrypted storage...");
    }

    // Load application config (encrypted in storage, only accessible when unsealed)
    let app_config = if !services.seal.is_sealed().await {
        match secreton_core::config::ApplicationConfig::load_from_storage(
            &*services.storage,
            &services.seal,
        )
        .await
        {
            Ok(cfg) => {
                info!("✅ Application config loaded successfully");
                cfg
            }
            Err(e) => {
                warn!("⚠️  Failed to load application config: {}", e);
                warn!("   Using default application config");
                warn!("   Run 'secreton config init' to initialize application config");
                secreton_core::config::ApplicationConfig::default()
            }
        }
    } else {
        info!("Using default application config (engine is sealed)");
        secreton_core::config::ApplicationConfig::default()
    };

    info!("Creating API configuration...");
    // Merge bootstrap + application config into ApiConfig
    let config =
        ApiConfig::from_bootstrap_and_application(&bootstrap, &app_config).map_err(|e| {
            error!("Failed to create API config: {}", e);
            std::io::Error::new(std::io::ErrorKind::InvalidInput, e)
        })?;

    // Apply environment variable overrides
    let mut config = config;
    config.apply_env_overrides().map_err(|e| {
        error!("Failed to apply environment overrides: {}", e);
        std::io::Error::new(std::io::ErrorKind::InvalidInput, e)
    })?;

    info!("Creating transit engine...");
    // Create transit engine (shared between REST and gRPC)
    let transit_engine = Arc::new(TransitEngine::new());

    info!("Creating KV engine...");
    // Create KV engine (shared between REST and gRPC)
    let kv_engine = Arc::new(KVEngine::new());

    info!("Initializing metrics recorder...");
    // Initialize Prometheus recorder
    let builder = PrometheusBuilder::new();
    let prometheus_handle = builder
        .install_recorder()
        .map_err(|e| {
            error!("Failed to install metrics recorder: {}", e);
            e
        })
        .ok();

    if prometheus_handle.is_some() {
        info!("Metrics recorder installed successfully");
    } else {
        warn!("Metrics recorder could not be installed");
    }

    info!("Creating Global Metrics...");
    let metrics = Arc::new(secreton_api::metrics::GlobalMetrics::new());

    info!("Creating API state...");
    // Create API state for REST server (includes ServiceContainer)
    let api_state = ApiState {
        transit: TransitApiState {
            engine: Arc::clone(&transit_engine),
            config: config.auth.mtls.clone().map(Arc::new),
            metrics: Arc::clone(&metrics),
        },
        kv: KVApiState {
            engine: kv_engine,
            metrics: Arc::clone(&metrics),
        },
        pki: PkiApiState::default(),
        services: Arc::clone(&services),
        prometheus_handle,
        metrics: Arc::clone(&metrics),
    };

    info!("Creating router...");
    // Create REST router
    let app = create_api_router(api_state);

    let http_addr: SocketAddr = config.http.bind_address;
    let grpc_addr: SocketAddr = config.grpc.bind_address;
    let tls_config_opt = config.tls.clone();
    let grpc_enabled = config.grpc.enabled;

    // Create gRPC service (shared state with REST).
    // It gets `secret_storage`, not `storage`: gRPC is the path the gateway
    // sidecar and simpelv1 use, so it must encrypt at rest like REST does.
    let grpc_service = SecretonGrpcService::new(
        services.secret_storage.clone(),
        Arc::clone(&transit_engine),
        Some(Arc::clone(&metrics.grpc_requests_total)),
    );

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

    // Initialize session cache
    secreton_api::tls_optimization::init_session_cache(1000, 3600);

    // Create optimized TLS configuration
    let server_config = secreton_api::tls_optimization::create_optimized_tls_config(
        cert_chain,
        rustls::pki_types::PrivateKeyDer::Pkcs8(keys.remove(0)),
        &tls_config.min_version,
        &tls_config.cipher_suites,
        &tls_config.alpn_protocols,
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
    axum_server::from_tcp_rustls(listener, rustls_config)?
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
    println!("║                 Custom Rust Engine Implementation               ║");
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
