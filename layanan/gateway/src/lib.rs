//! SIMPel gateway sidecar — a thin REST → gRPC proxy.
//!
//! simpelv1 (Laravel/php-fpm) cannot afford to open mTLS gRPC channels per
//! request, so per `monolith/simpelv1/AGENTS.md` it talks to the core Rust
//! services through a **local sidecar** that keeps long-lived upstream gRPC
//! channels and exposes a small REST surface on localhost. This crate is that
//! sidecar. The REST contract below is derived from the existing PHP clients
//! (`monolith/simpelv1/app/Services/Grpc/{Authenc,Secrethon,Integrasi}GrpcClient.php`),
//! NOT invented:
//!
//! | REST (PHP calls)                       | upstream gRPC                          |
//! |----------------------------------------|----------------------------------------|
//! | `POST /v1/tokens/verify {token}`       | authenc `ValidateToken`                |
//! | `GET  /v1/users/{id}`                  | authenc `GetUser`                      |
//! | `GET/PUT/DELETE /v1/secrets/{*path}`   | secreton `GetSecret/Store/Delete`      |
//! | `GET  /v1/database-credentials/{db}`   | secreton `GenerateDatabaseCredentials` |
//! | `GET  /v1/api-keys/{name}`             | secreton `GetSecret` (api-keys/…)      |
//! | `GET  /v1/mysimkari/employees/{nip}`   | integrasi `GetMysimkariPegawai`        |
//! | `GET  /v1/siman/inventory/{id}`        | integrasi `GetSimanAssets` (+filter)   |
//! | `GET  /v1/monsakti/*`                  | 501 — MonSAKTI dormant (→ MyIntress)   |
//! | `GET  /healthz`                        | gateway liveness                       |

use tonic::transport::Channel;

/// Generated gRPC client stubs for the upstreams we proxy. Same proto set as
/// `layanan/perlengkapan` (see `build.rs`) so the wire types stay in lockstep.
pub mod proto {
    pub mod authenc {
        pub mod v1 {
            tonic::include_proto!("authenc.v1");
        }
    }
    pub mod secreton {
        pub mod v1 {
            tonic::include_proto!("secreton.v1");
        }
    }
    pub mod common {
        pub mod v1 {
            tonic::include_proto!("common.v1");
        }
    }
    pub mod integrasi {
        pub mod v1 {
            tonic::include_proto!("integrasi.v1");
        }
    }
}

pub mod handlers;

use proto::authenc::v1::authenc_service_client::AuthencServiceClient;
use proto::integrasi::v1::integrasi_service_client::IntegrasiServiceClient;
use proto::secreton::v1::secreton_service_client::SecretonServiceClient;

/// Upstream gRPC endpoints, resolved from env at startup.
#[derive(Clone, Debug)]
pub struct GatewayConfig {
    pub listen_port: u16,
    pub authenc_grpc_url: String,
    pub secreton_grpc_url: String,
    pub integrasi_grpc_url: String,
}

impl GatewayConfig {
    /// Read config from the environment, applying compose/K8s-friendly defaults
    /// (in-cluster the upstreams are reachable by service name).
    pub fn from_env() -> Self {
        fn var(key: &str, default: &str) -> String {
            std::env::var(key).unwrap_or_else(|_| default.to_string())
        }
        Self {
            listen_port: var("GATEWAY_PORT", "8090").parse().unwrap_or(8090),
            authenc_grpc_url: var("AUTHENC_GRPC_URL", "http://authenc:50051"),
            secreton_grpc_url: var("SECRETON_GRPC_URL", "http://secreton:50053"),
            integrasi_grpc_url: var("INTEGRASI_GRPC_URL", "http://layanan-integrasi:50052"),
        }
    }
}

/// Shared state: long-lived (lazy) gRPC channels to each upstream.
///
/// `connect_lazy()` never fails at startup and reconnects transparently, so the
/// sidecar boots even if an upstream is briefly unavailable (it surfaces a 502
/// per-request instead, which the PHP clients treat as "indeterminate").
#[derive(Clone)]
pub struct AppState {
    pub authenc: AuthencServiceClient<Channel>,
    pub secreton: SecretonServiceClient<Channel>,
    pub integrasi: IntegrasiServiceClient<Channel>,
}

impl AppState {
    pub fn connect_lazy(cfg: &GatewayConfig) -> anyhow::Result<Self> {
        let authenc_ch = Channel::from_shared(cfg.authenc_grpc_url.clone())?.connect_lazy();
        let secreton_ch = Channel::from_shared(cfg.secreton_grpc_url.clone())?.connect_lazy();
        let integrasi_ch = Channel::from_shared(cfg.integrasi_grpc_url.clone())?.connect_lazy();
        Ok(Self {
            authenc: AuthencServiceClient::new(authenc_ch),
            secreton: SecretonServiceClient::new(secreton_ch),
            integrasi: IntegrasiServiceClient::new(integrasi_ch),
        })
    }
}

/// Build the REST router. Kept separate from `main` so it can be exercised in
/// tests and reused by other harnesses.
pub fn build_router(state: AppState) -> axum::Router {
    use axum::routing::{delete, get, post, put};
    use tower_http::trace::TraceLayer;

    axum::Router::new()
        .route("/healthz", get(handlers::healthz))
        .route("/health", get(handlers::healthz))
        // authenc
        .route("/v1/tokens/verify", post(handlers::verify_token))
        .route("/v1/users/{id}", get(handlers::get_user))
        // secreton
        .route("/v1/secrets/{*path}", get(handlers::get_secret))
        .route("/v1/secrets/{*path}", put(handlers::put_secret))
        .route("/v1/secrets/{*path}", delete(handlers::delete_secret))
        .route(
            "/v1/database-credentials/{db}",
            get(handlers::get_database_credentials),
        )
        .route("/v1/api-keys/{name}", get(handlers::get_api_key))
        // integrasi
        .route(
            "/v1/mysimkari/employees/{nip}",
            get(handlers::get_mysimkari_employee),
        )
        .route(
            "/v1/siman/inventory/{id}",
            get(handlers::get_siman_inventory),
        )
        .route("/v1/monsakti/{*rest}", get(handlers::monsakti_unavailable))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
