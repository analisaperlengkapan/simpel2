use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
/// Mewakili pub `AppRole`.
pub struct AppRole {
    pub role_id: String,
    pub secret_id: String,
    pub policies: Vec<String>,
}
