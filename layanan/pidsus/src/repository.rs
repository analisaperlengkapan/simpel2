use deadpool_postgres::Pool;
use std::error::Error;
use uuid::Uuid;
use crate::models::{CreateCaseRequest, SpecialCase, DashboardStats, SpecialCrimeType, SpecialCaseStatus, SpecialCasePriority, ClassificationLevel};
use chrono::Utc;

#[derive(Clone)]
pub struct Repository {
    pool: Pool,
}

impl Repository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    pub async fn get_cases(&self) -> Result<Vec<SpecialCase>, Box<dyn Error + Send + Sync>> {
        let client = self.pool.get().await?;
        let stmt = "
            SELECT
                id, case_number, crime_type, title, description, status, priority, classification,
                created_date, updated_date, lead_investigator, location, estimated_loss,
                suspects_count, evidence_count, witnesses_count, team_members, related_agencies
            FROM pidsus.cases
            ORDER BY created_date DESC
        ";

        let rows = client.query(stmt, &[]).await?;

        let cases = rows.iter().map(|row| {
            SpecialCase {
                id: row.get("id"),
                case_number: row.get("case_number"),
                crime_type: row.get("crime_type"),
                title: row.get("title"),
                description: row.get("description"),
                status: row.get("status"),
                priority: row.get("priority"),
                classification: row.get("classification"),
                created_date: row.get("created_date"),
                updated_date: row.get("updated_date"),
                lead_investigator: row.get("lead_investigator"),
                location: row.get("location"),
                estimated_loss: row.get("estimated_loss"),
                suspects_count: row.get("suspects_count"),
                evidence_count: row.get("evidence_count"),
                witnesses_count: row.get("witnesses_count"),
                team_members: row.get("team_members"),
                related_agencies: row.get("related_agencies"),
            }
        }).collect();

        Ok(cases)
    }

    pub async fn get_case(&self, id: Uuid) -> Result<Option<SpecialCase>, Box<dyn Error + Send + Sync>> {
        let client = self.pool.get().await?;
        let stmt = "
            SELECT
                id, case_number, crime_type, title, description, status, priority, classification,
                created_date, updated_date, lead_investigator, location, estimated_loss,
                suspects_count, evidence_count, witnesses_count, team_members, related_agencies
            FROM pidsus.cases
            WHERE id = $1
        ";

        let row_opt = client.query_opt(stmt, &[&id]).await?;

        if let Some(row) = row_opt {
            Ok(Some(SpecialCase {
                id: row.get("id"),
                case_number: row.get("case_number"),
                crime_type: row.get("crime_type"),
                title: row.get("title"),
                description: row.get("description"),
                status: row.get("status"),
                priority: row.get("priority"),
                classification: row.get("classification"),
                created_date: row.get("created_date"),
                updated_date: row.get("updated_date"),
                lead_investigator: row.get("lead_investigator"),
                location: row.get("location"),
                estimated_loss: row.get("estimated_loss"),
                suspects_count: row.get("suspects_count"),
                evidence_count: row.get("evidence_count"),
                witnesses_count: row.get("witnesses_count"),
                team_members: row.get("team_members"),
                related_agencies: row.get("related_agencies"),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn create_case(&self, req: CreateCaseRequest) -> Result<SpecialCase, Box<dyn Error + Send + Sync>> {
        let client = self.pool.get().await?;
        let stmt = "
            INSERT INTO pidsus.cases (
                case_number, crime_type, title, description, status, priority, classification,
                lead_investigator, location, estimated_loss, team_members, related_agencies
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING
                id, case_number, crime_type, title, description, status, priority, classification,
                created_date, updated_date, lead_investigator, location, estimated_loss,
                suspects_count, evidence_count, witnesses_count, team_members, related_agencies
        ";

        let row = client.query_one(stmt, &[
            &req.case_number,
            &req.crime_type,
            &req.title,
            &req.description,
            &req.status,
            &req.priority,
            &req.classification,
            &req.lead_investigator,
            &req.location,
            &req.estimated_loss,
            &req.team_members,
            &req.related_agencies,
        ]).await?;

        Ok(SpecialCase {
            id: row.get("id"),
            case_number: row.get("case_number"),
            crime_type: row.get("crime_type"),
            title: row.get("title"),
            description: row.get("description"),
            status: row.get("status"),
            priority: row.get("priority"),
            classification: row.get("classification"),
            created_date: row.get("created_date"),
            updated_date: row.get("updated_date"),
            lead_investigator: row.get("lead_investigator"),
            location: row.get("location"),
            estimated_loss: row.get("estimated_loss"),
            suspects_count: row.get("suspects_count"),
            evidence_count: row.get("evidence_count"),
            witnesses_count: row.get("witnesses_count"),
            team_members: row.get("team_members"),
            related_agencies: row.get("related_agencies"),
        })
    }

    pub async fn get_stats(&self) -> Result<DashboardStats, Box<dyn Error + Send + Sync>> {
        let client = self.pool.get().await?;

        // Simple aggregate queries
        let total_cases: i64 = client.query_one("SELECT COUNT(*) FROM pidsus.cases", &[]).await?.get(0);

        let active_cases: i64 = client.query_one("SELECT COUNT(*) FROM pidsus.cases WHERE status != 'Closed'", &[]).await?.get(0);

        let closed_cases: i64 = client.query_one("SELECT COUNT(*) FROM pidsus.cases WHERE status = 'Closed'", &[]).await?.get(0);

        let total_suspects: i64 = client.query_one("SELECT COALESCE(SUM(suspects_count), 0)::bigint FROM pidsus.cases", &[]).await?.get(0);

        let total_evidence: i64 = client.query_one("SELECT COALESCE(SUM(evidence_count), 0)::bigint FROM pidsus.cases", &[]).await?.get(0);

        let total_recovered_assets: f64 = 0.0; // Placeholder until we have a real transactions/asset table

        let conviction_rate = if total_cases > 0 {
             (closed_cases as f64 / total_cases as f64) * 100.0
        } else {
            0.0
        };

        Ok(DashboardStats {
            total_cases,
            active_cases,
            closed_cases,
            total_suspects,
            total_evidence,
            total_recovered_assets,
            conviction_rate,
        })
    }
}
