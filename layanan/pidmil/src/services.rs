use crate::models::{CreateCaseRequest, CreateSuspectRequest, MilitaryCase, MilitarySuspect};
use crate::repository::PostgresRepository;
use anyhow::Result;
use uuid::Uuid;

#[derive(Clone)]
pub struct PidmilService {
    repo: PostgresRepository,
}

impl PidmilService {
    pub fn new(repo: PostgresRepository) -> Self {
        Self { repo }
    }

    pub async fn create_case(&self, req: CreateCaseRequest) -> Result<MilitaryCase> {
        self.repo.create_case(req).await
    }

    pub async fn get_all_cases(&self) -> Result<Vec<MilitaryCase>> {
        self.repo.get_all_cases().await
    }

    pub async fn get_case_by_id(&self, id: Uuid) -> Result<Option<MilitaryCase>> {
        self.repo.get_case_by_id(id).await
    }

    pub async fn create_suspect(&self, case_id: Uuid, req: CreateSuspectRequest) -> Result<MilitarySuspect> {
        if let Some(_) = self.repo.get_case_by_id(case_id).await? {
             self.repo.create_suspect(case_id, req).await
        } else {
             Err(anyhow::anyhow!("Case not found"))
        }
    }

    pub async fn get_suspects_by_case(&self, case_id: Uuid) -> Result<Vec<MilitarySuspect>> {
        self.repo.get_suspects_by_case(case_id).await
    }
}
