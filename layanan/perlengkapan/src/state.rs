use axum::extract::FromRef;
use std::sync::Arc;

use lib_perlengkapan::contracts::{
    AuditSink, DocumentGenerator, DocumentStorage, NotificationSender,
};

use crate::shared::cache::CacheManager;
use crate::shared::grpc::clients::{AuthencClient, IntegrasiClient};
use crate::shared::rate_limit::RateLimiter;
use crate::{
    analisis::AnalisisService, dashboard, dashboard::services::DashboardService,
    kebutuhan_bmn::KebutuhanBmnService, pakaian_dinas::PakaianDinasService,
    pemakaian_bmn::PemakaianBmnService, penghapusan_bmn::PenghapusanBmnService,
    roadmap_sarpras::RoadmapService, services::PerlengkapanService,
};

#[derive(Clone)]
pub struct AppState {
    pub service: PerlengkapanService,
    pub analisis_service: AnalisisService,
    pub authenc: AuthencClient,
    pub pakaian_dinas_service: PakaianDinasService,
    pub kebutuhan_bmn_service: KebutuhanBmnService,
    pub pemakaian_bmn_service: PemakaianBmnService,
    pub penghapusan_bmn_service: Arc<PenghapusanBmnService>,
    pub roadmap_service: RoadmapService,
    pub dashboard_service: DashboardService,
    pub dashboard_updates: tokio::sync::broadcast::Sender<dashboard::DashboardUpdate>,
    pub db_pool: deadpool_postgres::Pool,
    pub cache_manager: Arc<CacheManager>,
    pub rate_limiter: Arc<RateLimiter>,
    /// Document generator (port). Replaces the dropped `dokumen` gRPC client.
    pub docs: Arc<dyn DocumentGenerator>,
    /// Notification sender (port). Replaces the dropped `notifikasi` gRPC
    /// client.
    pub notifier: Arc<dyn NotificationSender>,
    /// Cross-module audit sink. Concrete impl writes to
    /// `perlengkapan.audit_log` (see `shared::audit::PgAuditSink`).
    pub audit_sink: Arc<dyn AuditSink>,
    /// Document storage adapter. Filesystem-backed today
    /// (`dokumen::FilesystemStorage`); swappable for an S3 adapter in a
    /// follow-up without touching call sites.
    pub document_storage: Arc<dyn DocumentStorage>,
    /// Wall-clock instant the service finished bootstrapping; surfaced by
    /// the health endpoints as `uptime_seconds`. Cheap to clone (`Instant`
    /// is `Copy`).
    pub boot_time: std::time::Instant,
    /// Optional gRPC client to `layanan-integrasi`. Holds the same
    /// `Arc`-shared circuit breakers as the clones injected into the
    /// pakaian-dinas & kebutuhan-bmn services, so the health endpoint can
    /// observe live breaker state (SIMAN/MySIMKARI/MonSAKTI) via
    /// [`IntegrasiClient::circuit_states`]. `None` when integrasi is not
    /// configured. (Fase 2.2 lanjutan / #36.)
    pub integrasi_client: Option<IntegrasiClient>,
}

impl FromRef<AppState> for PerlengkapanService {
    fn from_ref(state: &AppState) -> Self {
        state.service.clone()
    }
}

impl FromRef<AppState> for AnalisisService {
    fn from_ref(state: &AppState) -> Self {
        state.analisis_service.clone()
    }
}

impl FromRef<AppState> for AuthencClient {
    fn from_ref(state: &AppState) -> Self {
        state.authenc.clone()
    }
}

impl FromRef<AppState> for PakaianDinasService {
    fn from_ref(state: &AppState) -> Self {
        state.pakaian_dinas_service.clone()
    }
}

impl FromRef<AppState> for KebutuhanBmnService {
    fn from_ref(state: &AppState) -> Self {
        state.kebutuhan_bmn_service.clone()
    }
}

impl FromRef<AppState> for DashboardService {
    fn from_ref(state: &AppState) -> Self {
        state.dashboard_service.clone()
    }
}

impl FromRef<AppState> for RoadmapService {
    fn from_ref(state: &AppState) -> Self {
        state.roadmap_service.clone()
    }
}

impl FromRef<AppState> for PemakaianBmnService {
    fn from_ref(state: &AppState) -> Self {
        state.pemakaian_bmn_service.clone()
    }
}

impl FromRef<AppState> for Arc<PenghapusanBmnService> {
    fn from_ref(state: &AppState) -> Self {
        state.penghapusan_bmn_service.clone()
    }
}

impl FromRef<AppState> for deadpool_postgres::Pool {
    fn from_ref(state: &AppState) -> Self {
        state.db_pool.clone()
    }
}

impl FromRef<AppState> for Arc<CacheManager> {
    fn from_ref(state: &AppState) -> Self {
        state.cache_manager.clone()
    }
}

impl FromRef<AppState> for Arc<RateLimiter> {
    fn from_ref(state: &AppState) -> Self {
        state.rate_limiter.clone()
    }
}

impl FromRef<AppState> for Arc<dyn DocumentGenerator> {
    fn from_ref(state: &AppState) -> Self {
        state.docs.clone()
    }
}

impl FromRef<AppState> for Arc<dyn NotificationSender> {
    fn from_ref(state: &AppState) -> Self {
        state.notifier.clone()
    }
}

impl FromRef<AppState> for Arc<dyn AuditSink> {
    fn from_ref(state: &AppState) -> Self {
        state.audit_sink.clone()
    }
}

impl FromRef<AppState> for Arc<dyn DocumentStorage> {
    fn from_ref(state: &AppState) -> Self {
        state.document_storage.clone()
    }
}

impl FromRef<AppState> for Option<IntegrasiClient> {
    fn from_ref(state: &AppState) -> Self {
        state.integrasi_client.clone()
    }
}
