use axum::extract::FromRef;
use std::sync::Arc;

use crate::{
    cache_strategy::CacheManager,
    dashboard,
    dashboard::services::DashboardService,
    grpc_clients::{AuthencClient, SecretonClient},
    kebutuhan_bmn::KebutuhanBmnService,
    pakaian_dinas::PakaianDinasService,
    pemakaian_bmn::PemakaianBmnService,
    penghapusan_bmn::PenghapusanBmnService,
    rate_limiting::RateLimiter,
    roadmap_sarpras::RoadmapService,
    services::PerlengkapanService,
};

#[derive(Clone)]
pub struct AppState {
    pub service: PerlengkapanService,
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
}

impl FromRef<AppState> for PerlengkapanService {
    fn from_ref(state: &AppState) -> Self {
        state.service.clone()
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
