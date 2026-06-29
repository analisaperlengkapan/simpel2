//! simpelv1 REST→gRPC gateway sidecar entrypoint.

use layanan_gateway::{AppState, GatewayConfig, build_router};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let cfg = GatewayConfig::from_env();
    tracing::info!(
        port = cfg.listen_port,
        authenc = %cfg.authenc_grpc_url,
        secreton = %cfg.secreton_grpc_url,
        integrasi = %cfg.integrasi_grpc_url,
        "starting gateway sidecar"
    );

    let state = AppState::connect_lazy(&cfg)?;
    let app = build_router(state);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], cfg.listen_port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(%addr, "gateway listening");
    axum::serve(listener, app).await?;
    Ok(())
}
