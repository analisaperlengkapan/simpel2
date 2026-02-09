use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MilitaryCase {
    pub id: Uuid,
    pub case_number: String,
    pub case_type: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub priority: String,
    pub created_date: DateTime<Utc>,
    pub updated_date: DateTime<Utc>,
    pub assigned_investigator: String,
    pub unit_involved: String,
    pub location: String,
    pub suspects_count: i32,
    pub evidence_count: i32,
}

impl From<Row> for MilitaryCase {
    fn from(row: Row) -> Self {
        Self {
            id: row.get("id"),
            case_number: row.get("case_number"),
            case_type: row.get("case_type"),
            title: row.get("title"),
            description: row.get("description"),
            status: row.get("status"),
            priority: row.get("priority"),
            created_date: row.get("created_date"),
            updated_date: row.get("updated_date"),
            assigned_investigator: row.get("assigned_investigator"),
            unit_involved: row.get("unit_involved"),
            location: row.get("location"),
            suspects_count: row.get("suspects_count"),
            evidence_count: row.get("evidence_count"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, garde::Validate)]
pub struct CreateCaseRequest {
    #[garde(length(min = 1))]
    pub title: String,
    #[garde(length(min = 1))]
    pub case_type: String,
    #[garde(length(min = 1))]
    pub description: String,
    #[garde(length(min = 1))]
    pub priority: String,
    #[garde(length(min = 1))]
    pub assigned_investigator: String,
    #[garde(length(min = 1))]
    pub unit_involved: String,
    #[garde(length(min = 1))]
    pub location: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MilitarySuspect {
    pub id: Uuid,
    pub nrp: String,
    pub name: String,
    pub rank: String,
    pub unit: String,
    pub position: String,
    pub case_id: Uuid,
    pub status: String,
    pub arrest_date: Option<DateTime<Utc>>,
    pub detention_status: String,
    pub charges: Vec<String>,
}

impl From<Row> for MilitarySuspect {
    fn from(row: Row) -> Self {
        Self {
            id: row.get("id"),
            nrp: row.get("nrp"),
            name: row.get("name"),
            rank: row.get("rank"),
            unit: row.get("unit"),
            position: row.get("position"),
            case_id: row.get("case_id"),
            status: row.get("status"),
            arrest_date: row.get("arrest_date"),
            detention_status: row.get("detention_status"),
            charges: row.get("charges"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, garde::Validate)]
pub struct CreateSuspectRequest {
    #[garde(length(min = 1))]
    pub nrp: String,
    #[garde(length(min = 1))]
    pub name: String,
    #[garde(length(min = 1))]
    pub rank: String,
    #[garde(length(min = 1))]
    pub unit: String,
    #[garde(skip)]
    pub position: String,
    #[garde(skip)]
    pub status: String,
    #[garde(skip)]
    pub detention_status: String,
    #[garde(skip)]
    pub charges: Vec<String>,
}
