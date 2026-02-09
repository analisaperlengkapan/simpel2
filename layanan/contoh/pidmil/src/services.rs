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

    pub async fn get_all_cases(&self, page: u32, limit: u32) -> Result<Vec<MilitaryCase>> {
        let (limit, offset) = calculate_pagination(page, limit);
        self.repo.get_all_cases(limit, offset).await
    }

    pub async fn get_case_by_id(&self, id: Uuid) -> Result<Option<MilitaryCase>> {
        self.repo.get_case_by_id(id).await
    }

    pub async fn create_suspect(
        &self,
        case_id: Uuid,
        req: CreateSuspectRequest,
    ) -> Result<MilitarySuspect> {
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

fn calculate_pagination(page: u32, limit: u32) -> (i64, i64) {
    let limit = limit as i64;
    let offset = ((page.max(1) - 1) as i64) * limit;
    (limit, offset)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_calculation() {
        assert_eq!(calculate_pagination(1, 10), (10, 0));
        assert_eq!(calculate_pagination(2, 10), (10, 10));
        assert_eq!(calculate_pagination(1, 50), (50, 0));
        assert_eq!(calculate_pagination(0, 10), (10, 0)); // Should handle 0 as 1
        assert_eq!(calculate_pagination(3, 20), (20, 40));
    }
}
