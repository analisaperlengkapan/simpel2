use axum::{
    routing::get,
    Router,
    response::IntoResponse,
};
use crate::handlers::{AppState, list_schedules, create_schedule, get_schedule};
use deadpool_postgres::Pool;

async fn health_check() -> impl IntoResponse {
    "pengawasan Service OK"
}

pub fn create_router(pool: Pool) -> Router {
    let state = AppState::new(pool);

    let api_routes = Router::new()
        .route("/schedules", get(list_schedules).post(create_schedule))
        .route("/schedules/:id", get(get_schedule))
        .with_state(state);

    Router::new()
        .route("/", get(health_check))
        .route("/api/v1/pengawasan/health", get(health_check))
        .nest("/api/v1/pengawasan", api_routes)
}
