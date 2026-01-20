use crate::models::{CreateCaseRequest, CreateSuspectRequest, MilitaryCase, MilitarySuspect};
use anyhow::Result;
use deadpool_postgres::Pool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PostgresRepository {
    pool: Pool,
}

impl PostgresRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    pub async fn create_case(&self, req: CreateCaseRequest) -> Result<MilitaryCase> {
        let client = self.pool.get().await?;
        let stmt = client
            .prepare(
                "INSERT INTO military_cases
                (case_number, case_type, title, description, status, priority,
                 assigned_investigator, unit_involved, location, suspects_count, evidence_count)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 0, 0)
                RETURNING *",
            )
            .await?;

        // Generate case number format: PIDMIL-YYYY-XXXX (Random for demo)
        let case_number = format!("PIDMIL-{}-{}", chrono::Utc::now().format("%Y"), Uuid::new_v4().to_string()[..4].to_uppercase());

        let row = client
            .query_one(
                &stmt,
                &[
                    &case_number,
                    &req.case_type,
                    &req.title,
                    &req.description,
                    &"Reported", // Default status
                    &req.priority,
                    &req.assigned_investigator,
                    &req.unit_involved,
                    &req.location,
                ],
            )
            .await?;

        Ok(MilitaryCase::from(row))
    }

    pub async fn get_all_cases(&self) -> Result<Vec<MilitaryCase>> {
        let client = self.pool.get().await?;
        let rows = client
            .query("SELECT * FROM military_cases ORDER BY created_date DESC", &[])
            .await?;

        Ok(rows.into_iter().map(MilitaryCase::from).collect())
    }

    pub async fn get_case_by_id(&self, id: Uuid) -> Result<Option<MilitaryCase>> {
        let client = self.pool.get().await?;
        let rows = client
            .query("SELECT * FROM military_cases WHERE id = $1", &[&id])
            .await?;

        if rows.is_empty() {
            Ok(None)
        } else {
            Ok(Some(MilitaryCase::from(rows[0].clone())))
        }
    }

    pub async fn create_suspect(&self, case_id: Uuid, req: CreateSuspectRequest) -> Result<MilitarySuspect> {
        let client = self.pool.get().await?;
        let stmt = client.prepare(
            "INSERT INTO military_suspects (nrp, name, rank, unit, position, case_id, status, detention_status, charges)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
             RETURNING *"
        ).await?;

        let row = client.query_one(&stmt, &[
            &req.nrp,
            &req.name,
            &req.rank,
            &req.unit,
            &req.position,
            &case_id,
            &req.status,
            &req.detention_status,
            &req.charges
        ]).await?;

        // Update suspect count on case
        client.execute(
            "UPDATE military_cases SET suspects_count = suspects_count + 1 WHERE id = $1",
            &[&case_id]
        ).await?;

        Ok(MilitarySuspect::from(row))
    }

    pub async fn get_suspects_by_case(&self, case_id: Uuid) -> Result<Vec<MilitarySuspect>> {
        let client = self.pool.get().await?;
        let rows = client
            .query("SELECT * FROM military_suspects WHERE case_id = $1", &[&case_id])
            .await?;

        Ok(rows.into_iter().map(MilitarySuspect::from).collect())
    }
}
