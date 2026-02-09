use crate::client::MonsaktiClient;
use crate::error::MonsaktiError;

/// Endpoint: <https://mysimkari.kejaksaan.go.id/api/anbut/get-satker>
/// Mengambil data satker dari MySIMKARI
pub async fn get_satker(client: &mut MonsaktiClient) -> Result<serde_json::Value, MonsaktiError> {
    let response = client.fetch_mysimkari("get-satker", vec![]).await?;
    response
        .data
        .ok_or_else(|| MonsaktiError::ApiError("No data".to_string()))
}

/// Endpoint: <https://mysimkari.kejaksaan.go.id/api/anbut/pegawai-satker/{id}>
/// Mengambil data pegawai untuk satker tertentu
pub async fn pegawai_satker(
    client: &mut MonsaktiClient,
    satker_id: &str,
) -> Result<serde_json::Value, MonsaktiError> {
    let response = client
        .fetch_mysimkari("pegawai-satker", vec![satker_id.to_string()])
        .await?;
    response
        .data
        .ok_or_else(|| MonsaktiError::ApiError("No data".to_string()))
}
