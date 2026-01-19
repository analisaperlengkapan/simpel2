use serde::{Serialize, Deserialize};

/// Mewakili struktur data `Permission`.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Permission {
    pub id: String,
    pub name: String,
    pub realm: String,
    pub description: Option<String>,
}
