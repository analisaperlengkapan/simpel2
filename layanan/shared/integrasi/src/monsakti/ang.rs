use crate::client::MonsaktiClient;
use crate::error::MonsaktiError;

/// Endpoint: /API/ANG/dataAng/KLxxx/KDSATKER/KODE_STS_HISTORY
pub async fn data_ang(client: &mut MonsaktiClient, kode_kl: &str, kdsatker: &str, kode_sts_history: &str) -> Result<serde_json::Value, MonsaktiError> {
    let kl_formatted = format!("KL{}", kode_kl);
    let response = client.fetch("ANG", "dataAng", vec![kl_formatted, kdsatker.to_string(), kode_sts_history.to_string()]).await?;
    response.data.ok_or_else(|| MonsaktiError::ApiError("No data".to_string()))
}

/// Endpoint: /API/ANG/refSts/KLxxx/KDSATKER (opsional)
pub async fn ref_sts(client: &mut MonsaktiClient, kode_kl: &str, kdsatker: &str) -> Result<serde_json::Value, MonsaktiError> {
    let kl_formatted = format!("KL{}", kode_kl);
    let response = client.fetch("ANG", "refSts", vec![kl_formatted, kdsatker.to_string()]).await?;
    response.data.ok_or_else(|| MonsaktiError::ApiError("No data".to_string()))
}

/// Endpoint: /API/ANG/pendapatan/KLxxx/KDSATKER/KODE_STS_HISTORY (opsional)
pub async fn pendapatan(client: &mut MonsaktiClient, kode_kl: &str, kdsatker: &str, kode_sts_history: &str) -> Result<serde_json::Value, MonsaktiError> {
    let kl_formatted = format!("KL{}", kode_kl);
    let response = client.fetch("ANG", "pendapatan", vec![kl_formatted, kdsatker.to_string(), kode_sts_history.to_string()]).await?;
    response.data.ok_or_else(|| MonsaktiError::ApiError("No data".to_string()))
}
