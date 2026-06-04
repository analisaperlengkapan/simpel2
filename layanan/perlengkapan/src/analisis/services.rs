//! Business logic for Analisis Kebutuhan.

use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use super::models::{AnalisisKebutuhan, CreateAnalisisRequest};
use super::repository::AnalisisRepository;
use crate::shared::error::AppResult;

#[derive(Clone)]
pub struct AnalisisService {
    repo: Arc<dyn AnalisisRepository>,
}

impl AnalisisService {
    pub fn new(repo: Arc<dyn AnalisisRepository>) -> Self {
        Self { repo }
    }

    pub async fn get_all_analisis(
        &self,
        page: i32,
        per_page: i32,
    ) -> AppResult<(Vec<AnalisisKebutuhan>, i64)> {
        self.repo.get_all_analisis(page, per_page).await
    }

    pub async fn get_analisis_by_id(&self, id: Uuid) -> AppResult<AnalisisKebutuhan> {
        self.repo.get_analisis_by_id(id).await
    }

    pub async fn create_analisis(
        &self,
        request: CreateAnalisisRequest,
        user_id: Option<Uuid>,
    ) -> AppResult<AnalisisKebutuhan> {
        request.validate()?;
        self.repo.create_analisis(request, user_id).await
    }
}
