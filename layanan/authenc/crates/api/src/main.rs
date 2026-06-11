//! Authenc identity provider - main entry point
//!
//! Wires up all sub-crates and starts the HTTP server.

use std::net::SocketAddr;
use std::sync::Arc;

use authenc_core::services::{
    AuthenticationServiceImpl, BruteForceProtectorImpl, OAuth2ServiceImpl,
    ProductionClientRegistrationService, RealmManagementServiceImpl, UserManagementServiceImpl,
};
use authenc_crypto::{Argon2PasswordHasher, JwtService};
use authenc_storage::{
    Database, PostgresClientStore, PostgresCredentialStore, PostgresRealmStore,
    PostgresRevocationStore, PostgresSessionStore, PostgresUserStore,
};
use authenc_types::{
    AuthorizationCode, RefreshToken, Result, UserId,
    traits::{AuthorizationCodeStore, RefreshTokenStore, TokenGenerator},
};
use authenc_webauthn::{WebAuthnConfig, WebAuthnService};

use async_trait::async_trait;
use chrono::Utc;
use dashmap::DashMap;
use tracing::{info, warn};
use url::Url;

use authenc_api::{
    AppConfig, AxumApp, CorsConfig, CsrfConfig, Environment,
    session_store::SessionStore as WebAuthnSessionStore, state::ApiState,
};
use authenc_iam_api::{create_iam_router, state::IamApiState};

// ============================================================================
// In-memory OAuth2 stores
// ============================================================================

struct InMemoryAuthorizationCodeStore {
    codes: DashMap<String, AuthorizationCode>,
}

impl InMemoryAuthorizationCodeStore {
    fn new() -> Self {
        Self {
            codes: DashMap::new(),
        }
    }
}

#[async_trait]
impl AuthorizationCodeStore for InMemoryAuthorizationCodeStore {
    async fn store_code(&self, code: AuthorizationCode) -> Result<()> {
        self.codes.insert(code.code.clone(), code);
        Ok(())
    }

    async fn get_code(&self, code: &str) -> Result<Option<AuthorizationCode>> {
        Ok(self.codes.get(code).map(|c| c.clone()))
    }

    async fn mark_code_used(&self, code: &str) -> Result<()> {
        if let Some(mut c) = self.codes.get_mut(code) {
            c.used = true;
        }
        Ok(())
    }

    async fn cleanup_expired_codes(&self) -> Result<usize> {
        let now = Utc::now();
        let before = self.codes.len();
        self.codes.retain(|_, c| c.expires_at > now);
        Ok(before - self.codes.len())
    }
}

struct InMemoryRefreshTokenStore {
    tokens: DashMap<String, RefreshToken>,
}

impl InMemoryRefreshTokenStore {
    fn new() -> Self {
        Self {
            tokens: DashMap::new(),
        }
    }
}

#[async_trait]
impl RefreshTokenStore for InMemoryRefreshTokenStore {
    async fn store_token(&self, token: RefreshToken) -> Result<()> {
        self.tokens.insert(token.token.clone(), token);
        Ok(())
    }

    async fn get_token(&self, token: &str) -> Result<Option<RefreshToken>> {
        Ok(self.tokens.get(token).map(|t| t.clone()))
    }

    async fn revoke_token(&self, token: &str) -> Result<()> {
        if let Some(mut t) = self.tokens.get_mut(token) {
            t.revoked = true;
        }
        Ok(())
    }

    async fn revoke_user_tokens(&self, user_id: UserId) -> Result<()> {
        self.tokens.iter_mut().for_each(|mut entry| {
            if entry.user_id == user_id {
                entry.revoked = true;
            }
        });
        Ok(())
    }

    async fn cleanup_expired_tokens(&self) -> Result<usize> {
        let now = Utc::now();
        let before = self.tokens.len();
        self.tokens.retain(|_, t| t.expires_at > now && !t.revoked);
        Ok(before - self.tokens.len())
    }
}

/// JWT-backed implementation of the TokenGenerator trait
struct JwtTokenGenerator {
    jwt_service: Arc<JwtService>,
}

impl JwtTokenGenerator {
    fn new(jwt_service: Arc<JwtService>) -> Self {
        Self { jwt_service }
    }
}

impl TokenGenerator for JwtTokenGenerator {
    fn generate_access_token(&self, user_id: UserId, scope: &str) -> Result<String> {
        let uid = user_id.as_uuid().to_string();
        self.jwt_service
            .generate_access_token(&uid, None, Some(scope.to_string()), None)
    }

    fn generate_refresh_token(&self, user_id: UserId) -> Result<String> {
        let uid = user_id.as_uuid().to_string();
        let session_id = uuid::Uuid::new_v4().to_string();
        self.jwt_service.generate_refresh_token(&uid, &session_id)
    }

    fn validate_token(&self, token: &str) -> Result<authenc_types::traits::TokenClaims> {
        let claims = self.jwt_service.verify_token(token)?;
        Ok(authenc_types::traits::TokenClaims {
            sub: claims.sub,
            iss: claims.iss,
            aud: claims.aud,
            exp: claims.exp,
            iat: claims.iat,
            scope: claims.scope.unwrap_or_default(),
        })
    }
}

/// Load gRPC mTLS material from Secreton-provisioned file paths, if configured.
///
/// Requires `GRPC_TLS_CERT_PATH` + `GRPC_TLS_KEY_PATH`; when `GRPC_TLS_CA_PATH`
/// is also set the server enforces mutual auth (clients must present a cert
/// signed by the CA). Returns `None` when not configured, so callers can decide
/// whether to refuse startup (production) or run plaintext (dev).
fn load_grpc_tls() -> Option<authenc_grpc::TlsConfig> {
    let cert = std::env::var("GRPC_TLS_CERT_PATH").ok()?;
    let key = std::env::var("GRPC_TLS_KEY_PATH").ok()?;
    let ca = std::env::var("GRPC_TLS_CA_PATH").ok();
    let mutual = ca.is_some();
    match authenc_grpc::TlsConfig::from_files(&cert, &key, ca) {
        Ok(cfg) => {
            info!("gRPC mTLS enabled (mutual_auth={})", mutual);
            Some(cfg)
        }
        Err(e) => {
            warn!(
                "Failed to load gRPC TLS material from configured paths: {}",
                e
            );
            None
        }
    }
}

// ============================================================================
// Main
// ============================================================================

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "authenc=info,tower_http=info".to_string()),
        )
        .json()
        .init();

    info!("Starting Authenc identity provider");

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://authenc:authenc@localhost:5432/authenc".to_string());
    let jwt_issuer =
        std::env::var("JWT_ISSUER").unwrap_or_else(|_| "http://10.1.7.121/api/v1/auth".to_string());
    let jwt_secret_hex = std::env::var("JWT_SECRET").unwrap_or_default();
    let webauthn_rp_id =
        std::env::var("WEBAUTHN_RP_ID").unwrap_or_else(|_| "localhost".to_string());
    let webauthn_rp_origin =
        std::env::var("WEBAUTHN_RP_ORIGIN").unwrap_or_else(|_| "http://localhost:8088".to_string());
    let webauthn_rp_name =
        std::env::var("WEBAUTHN_RP_NAME").unwrap_or_else(|_| "SIMPEL Authenc".to_string());
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "8088".to_string())
        .parse()
        .unwrap_or(8088);
    let registration_base =
        std::env::var("REGISTRATION_BASE").unwrap_or_else(|_| jwt_issuer.clone());

    // Database
    info!("Connecting to database...");
    let db = Arc::new(
        Database::new(&database_url, 20)
            .await
            .map_err(|e| format!("Failed to connect to database: {}", e))?,
    );
    info!("Database connected");

    // Stores
    let user_store = Arc::new(PostgresUserStore::new(db.clone()));
    let session_store = Arc::new(PostgresSessionStore::new(db.clone()));
    let realm_store = Arc::new(PostgresRealmStore::new(db.clone()));
    let client_store = Arc::new(PostgresClientStore::new(db.clone()));
    let credential_store = Arc::new(PostgresCredentialStore::new(db.clone()));
    // Token revocation list (F2H) — shared by REST validate/introspect and the
    // gRPC validate_token service so both reject revoked tokens before `exp`.
    let revocation_store = Arc::new(PostgresRevocationStore::new(db.clone()));

    // JWT service
    let signing_key_bytes: [u8; 32] = if !jwt_secret_hex.is_empty() {
        let bytes =
            hex::decode(&jwt_secret_hex).map_err(|e| format!("Invalid JWT_SECRET hex: {}", e))?;
        if bytes.len() != 32 {
            return Err("JWT_SECRET must be 32 bytes (64 hex chars)".into());
        }
        let mut key = [0u8; 32];
        key.copy_from_slice(&bytes);
        key
    } else {
        info!("JWT_SECRET not set - generating ephemeral key (not for production)");
        JwtService::generate_signing_key()
    };

    let jwt_service = Arc::new(JwtService::new(
        &signing_key_bytes,
        jwt_issuer.clone(),
        chrono::Duration::minutes(15),
        chrono::Duration::days(7),
    )?);

    let password_hasher = Arc::new(Argon2PasswordHasher::new());
    let brute_force_protector = Arc::new(BruteForceProtectorImpl::new());
    // Same instance shared into ApiState so the login handler can enforce CAPTCHA
    // once a username crosses the failure threshold (#49).
    let brute_force_protector_for_state = brute_force_protector.clone();

    // OAuth2 stores
    let code_store = Arc::new(InMemoryAuthorizationCodeStore::new());
    let refresh_token_store = Arc::new(InMemoryRefreshTokenStore::new());
    let token_generator = Arc::new(JwtTokenGenerator::new(jwt_service.clone()));

    // Services
    let auth_service = Arc::new(AuthenticationServiceImpl::new(
        user_store.clone(),
        session_store.clone(),
        password_hasher.clone(),
        brute_force_protector,
    ));
    let user_service = Arc::new(UserManagementServiceImpl::new(
        user_store.clone(),
        password_hasher,
    ));
    let oauth2_service = Arc::new(OAuth2ServiceImpl::new(
        client_store.clone(),
        code_store,
        refresh_token_store,
        token_generator,
    ));
    let realm_service = Arc::new(RealmManagementServiceImpl::new(realm_store));
    let client_service = Arc::new(ProductionClientRegistrationService::new(
        db.clone(),
        None,
        registration_base,
    ));

    // WebAuthn
    let webauthn_config = WebAuthnConfig {
        rp_id: webauthn_rp_id,
        rp_origin: Url::parse(&webauthn_rp_origin)
            .map_err(|e| format!("Invalid WEBAUTHN_RP_ORIGIN: {}", e))?,
        rp_name: webauthn_rp_name,
    };
    let webauthn_service = Arc::new(WebAuthnService::new(webauthn_config, credential_store)?);

    // WebAuthn session store (in-memory)
    let webauthn_session_store = WebAuthnSessionStore::new();

    // Clone services shared between ApiState and IamApiState
    let iam_user_service = user_service.clone();
    let iam_realm_service = realm_service.clone();
    let iam_oauth2_service = oauth2_service.clone();
    let iam_jwt_service = jwt_service.clone();

    // CAPTCHA service
    let captcha_service = Arc::new(authenc_core::services::CaptchaService::new((*db).clone()));

    // MFA adapter — Postgres-backed TOTP + backup codes wired into the
    // REST `MfaApiService` trait. Without this the handlers in
    // `handlers::mfa` always returned `mfa_not_configured` and the Portal
    // /mfa-setup page bounced off a 503. Phase 1.3 of the stabilization plan.
    let mfa_service: Arc<dyn authenc_api::handlers::mfa::MfaApiService> = {
        let totp_store = Arc::new(authenc_api::services::PgTotpStore::new(db.clone()));
        let backup_store = Arc::new(authenc_api::services::PgBackupCodesStore::new(db.clone()));
        Arc::new(authenc_api::services::LocalMfaApi::with_defaults(
            totp_store,
            backup_store,
        ))
    };

    // Clones for the gRPC service (service<->service channel). The Arcs below
    // are moved into ApiState::new, so capture clones first.
    let grpc_auth_service = auth_service.clone();
    let grpc_user_service = user_service.clone();
    let grpc_oauth2_service = oauth2_service.clone();
    let grpc_realm_service = realm_service.clone();
    let grpc_jwt_service = jwt_service.clone();

    // API state
    let mut state = ApiState::new(
        jwt_service,
        auth_service,
        user_service,
        oauth2_service,
        realm_service,
        client_service,
        webauthn_service,
        Some(mfa_service),
        webauthn_session_store,
        db.clone(),
        revocation_store.clone(),
        captcha_service,
    )
    .with_brute_force_protector(brute_force_protector_for_state);

    // Background sweep: purge revocation rows past their expires_at so the table
    // doesn't grow unbounded. Hourly is plenty — rows only matter until `exp`.
    {
        let revocation_store = revocation_store.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(3600));
            loop {
                interval.tick().await;
                match revocation_store.cleanup_expired().await {
                    Ok(n) if n > 0 => info!("Purged {} expired token revocation(s)", n),
                    Ok(_) => {}
                    Err(e) => warn!("Token revocation cleanup failed: {}", e),
                }
            }
        });
    }

    // gRPC server (service<->service channel: authenc <-> perlengkapan /
    // integrasi / simpelv1). Enforces the same token revocation as the REST
    // surface and is secured with mTLS — transport-level zero-trust — in
    // production. Runs alongside the HTTP server.
    {
        let grpc_port: u16 = std::env::var("GRPC_PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(9088);
        let grpc_addr = SocketAddr::from(([0, 0, 0, 0], grpc_port));

        // Services the gRPC facade needs beyond those held by ApiState.
        let role_service = Arc::new(authenc_core::services::RoleManagementServiceImpl::new(
            db.clone(),
        ));
        let federation_service = Arc::new(authenc_federation::service::FederationService::new());
        let audit_service = Arc::new(authenc_core::services::AuditService::new(db.clone()));

        let grpc_service = authenc_grpc::AuthencGrpcService::new(authenc_grpc::AuthencGrpcDeps {
            auth_service: grpc_auth_service,
            user_service: grpc_user_service,
            oauth2_service: grpc_oauth2_service,
            realm_service: grpc_realm_service,
            role_service,
            jwt_service: grpc_jwt_service,
            federation_service,
            audit_service,
            revocation_store: revocation_store.clone(),
        });

        // mTLS material (Secreton-provisioned). When a CA cert is supplied the
        // server requires clients to present a cert signed by it — mutual auth,
        // the zero-trust transport for the mesh. Fail closed in production.
        let tls_config = load_grpc_tls();
        let is_production = std::env::var("APP_ENVIRONMENT")
            .map(|v| v.eq_ignore_ascii_case("production"))
            .unwrap_or(false);
        let allow_insecure = std::env::var("GRPC_ALLOW_INSECURE").unwrap_or_default() == "true";
        if tls_config.is_none() && is_production && !allow_insecure {
            return Err("gRPC mTLS required in production: set GRPC_TLS_CERT_PATH, \
                 GRPC_TLS_KEY_PATH and GRPC_TLS_CA_PATH (Secreton PKI), or set \
                 GRPC_ALLOW_INSECURE=true for transitional deploys"
                .into());
        }

        let grpc_config = authenc_grpc::GrpcServerConfig {
            bind_address: grpc_addr,
            tls_config,
            enable_logging: true,
            enable_auth: true,
        };

        info!("Starting gRPC server on {}", grpc_addr);
        tokio::spawn(async move {
            if let Err(e) = authenc_grpc::GrpcServerBuilder::new(grpc_config)
                .with_service(grpc_service)
                .serve()
                .await
            {
                tracing::error!("gRPC server terminated: {}", e);
            }
        });
    }

    // Integrasi gRPC client (optional, enabled via INTEGRASI_GRPC_URL env var)
    if let Ok(integrasi_url) = std::env::var("INTEGRASI_GRPC_URL") {
        let integrasi_config = authenc_core::config::IntegrasiConfig {
            grpc_url: integrasi_url,
            sync_interval_minutes: 60,
            sync_on_startup: false,
            connection_timeout_secs: std::env::var("INTEGRASI_CONNECT_TIMEOUT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
            request_timeout_secs: std::env::var("INTEGRASI_REQUEST_TIMEOUT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(30),
        };
        match authenc_federation::IntegrasiGrpcClient::new_lazy(&integrasi_config) {
            Ok(client) => {
                info!("Integrasi gRPC client configured");
                state = state.with_integrasi_client(Arc::new(client));
            }
            Err(e) => {
                warn!("Failed to create integrasi gRPC client: {}", e);
            }
        }
    }

    // Satker identity = integrasi/MySIMKARI (SoT, #42). Build the read-model once
    // and feed BOTH the read view and the RBAC hierarchy from it; fall back to the
    // local DB cache only if integrasi is unreachable at startup (resilience, not
    // a second master).
    let all_satkers = match &state.integrasi_client {
        Some(client) => match client.get_satker_readmodel().await {
            Ok(s) if !s.is_empty() => {
                info!(
                    "Loaded {} satkers from integrasi read-model (identity SoT)",
                    s.len()
                );
                s
            }
            Ok(_) => {
                warn!("integrasi returned 0 satkers; falling back to local DB cache");
                db.get_all_satkers().await.unwrap_or_default()
            }
            Err(e) => {
                warn!("integrasi satker read-model failed ({e}); falling back to local DB cache");
                db.get_all_satkers().await.unwrap_or_default()
            }
        },
        None => {
            warn!("No integrasi client configured; using local DB satker cache");
            db.get_all_satkers().await.unwrap_or_default()
        }
    };
    let satker_service = Arc::new(authenc_core::services::SatkerManagementService::new(
        all_satkers.clone(),
    ));
    let satker_auth_service = Arc::new(authenc_core::services::SatkerAuthorizationService::new(
        all_satkers,
    ));

    // IAM API - create state and router for admin endpoints
    let iam_state = IamApiState::new(
        iam_user_service,
        iam_realm_service,
        iam_oauth2_service,
        iam_jwt_service,
        satker_service,
        satker_auth_service,
    );
    let iam_router = create_iam_router(Arc::new(iam_state));

    // App config - development CORS + disabled CSRF for API service
    let config = AppConfig {
        cors: CorsConfig::new(Environment::Development),
        csrf: CsrfConfig {
            enabled: false,
            ..Default::default()
        },
        ..Default::default()
    };

    let app = AxumApp::new(state, config);
    // Merge IAM admin routes into the main router
    let router = app.into_router().merge(iam_router);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("Server listening on {}", addr);
    axum::serve(
        listener,
        router.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(async {
        tokio::signal::ctrl_c().await.ok();
        info!("Graceful shutdown initiated");
    })
    .await?;

    Ok(())
}
