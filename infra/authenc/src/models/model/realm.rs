use serde::{Serialize, Deserialize};

/// Mewakili struktur data `Realm`.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Realm {
    pub id: String,
    pub name: String,
    pub enabled: bool,
}
