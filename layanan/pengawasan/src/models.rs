use chrono::{DateTime, Utc};
use garde::Validate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SupervisionSchedule {
    #[garde(skip)]
    pub id: Uuid,
    #[garde(length(min = 1))]
    pub title: String,
    #[garde(skip)]
    pub scheduled_at: DateTime<Utc>,
    #[garde(length(min = 1))]
    pub location: String,
    #[garde(skip)]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateScheduleRequest {
    #[garde(length(min = 1))]
    pub title: String,
    #[garde(skip)]
    pub scheduled_at: DateTime<Utc>,
    #[garde(length(min = 1))]
    pub location: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupervisionReport {
    pub id: Uuid,
    pub schedule_id: Uuid,
    pub content: String,
    pub findings: Vec<Finding>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub id: Uuid,
    pub description: String,
    pub severity: FindingSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FindingSeverity {
    Low,
    Medium,
    High,
    Critical,
}
