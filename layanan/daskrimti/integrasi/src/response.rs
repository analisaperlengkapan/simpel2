use serde::{Deserialize, Serialize};

/// Struktur respons dari API MonSAKTI
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MonsaktiResponse {
    /// Token baru jika ada
    #[serde(rename = "newToken")]
    pub new_token: Option<String>,
    /// Data respons
    pub data: Option<serde_json::Value>,
    /// Pesan error jika ada
    pub error: Option<String>,
}
