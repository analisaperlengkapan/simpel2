use deadpool_postgres::{Pool, Client};
use tokio_postgres::NoTls;
use anyhow::Result;
use uuid::Uuid;
use crate::model::{Case, Asset, CreateCaseRequest, CreateAssetRequest};

#[derive(Clone)]
pub struct DB {
    pool: Pool,
}

impl DB {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    async fn get_client(&self) -> Result<Client> {
        self.pool.get().await.map_err(|e| anyhow::anyhow!(e))
    }

    pub async fn list_cases(&self) -> Result<Vec<Case>> {
        let client = self.get_client().await?;
        let rows = client.query("SELECT id, title, description, status, created_at, updated_at FROM cases ORDER BY created_at DESC", &[]).await?;

        let mut cases = Vec::new();
        for row in rows {
            cases.push(Case {
                id: row.get("id"),
                title: row.get("title"),
                description: row.get("description"),
                status: row.get("status"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }
        Ok(cases)
    }

    pub async fn create_case(&self, req: CreateCaseRequest) -> Result<Case> {
        let client = self.get_client().await?;
        let id = Uuid::new_v4();
        let row = client.query_one(
            "INSERT INTO cases (id, title, description, status) VALUES ($1, $2, $3, 'Draft') RETURNING id, title, description, status, created_at, updated_at",
            &[&id, &req.title, &req.description]
        ).await?;

        Ok(Case {
            id: row.get("id"),
            title: row.get("title"),
            description: row.get("description"),
            status: row.get("status"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    pub async fn list_assets(&self, case_id: Uuid) -> Result<Vec<Asset>> {
        let client = self.get_client().await?;
        let rows = client.query(
            "SELECT id, case_id, name, asset_type, CAST(estimated_value AS DOUBLE PRECISION), status, location, created_at, updated_at FROM assets WHERE case_id = $1 ORDER BY created_at DESC",
            &[&case_id]
        ).await?;

        let mut assets = Vec::new();
        for row in rows {
            assets.push(Asset {
                id: row.get(0),
                case_id: row.get(1),
                name: row.get(2),
                asset_type: row.get(3),
                estimated_value: row.get(4),
                status: row.get(5),
                location: row.get(6),
                created_at: row.get(7),
                updated_at: row.get(8),
            });
        }
        Ok(assets)
    }

    pub async fn create_asset(&self, req: CreateAssetRequest) -> Result<Asset> {
        let client = self.get_client().await?;
        let id = Uuid::new_v4();
        let row = client.query_one(
            "INSERT INTO assets (id, case_id, name, asset_type, estimated_value, status, location) VALUES ($1, $2, $3, $4, $5, 'Identified', $6) RETURNING id, case_id, name, asset_type, CAST(estimated_value AS DOUBLE PRECISION), status, location, created_at, updated_at",
            &[&id, &req.case_id, &req.name, &req.asset_type, &req.estimated_value, &req.location]
        ).await?;

        Ok(Asset {
            id: row.get(0),
            case_id: row.get(1),
            name: row.get(2),
            asset_type: row.get(3),
            estimated_value: row.get(4),
            status: row.get(5),
            location: row.get(6),
            created_at: row.get(7),
            updated_at: row.get(8),
        })
    }
}
