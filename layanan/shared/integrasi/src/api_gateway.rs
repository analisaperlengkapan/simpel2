use axum::Router;

pub struct ApiGateway;

impl ApiGateway {
    pub fn new() -> Self {
        Self
    }

    pub fn create_router(&self) -> Router {
        Router::new()
        // TODO: Implement API gateway routes
    }
}
