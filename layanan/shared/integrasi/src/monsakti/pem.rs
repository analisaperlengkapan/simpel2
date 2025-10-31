use crate::client::MonsaktiClient;
use crate::error::MonsaktiError;

/// Endpoint: /API/PEM/realisasi/KLxxx/KDSATKER (opsional)/KD_JNS_SPP (opsional)/NO_SPP (opsional)
pub async fn realisasi(client: &mut MonsaktiClient, kode_kl: &str, kdsatker: &str, kd_jns_spp: &str, no_spp: &str) -> Result<serde_json::Value, MonsaktiError> {
    let kl_formatted = format!("KL{}", kode_kl);
    let response = client.fetch("PEM", "realisasi", vec![kl_formatted, kdsatker.to_string(), kd_jns_spp.to_string(), no_spp.to_string()]).await?;
    response.data.ok_or_else(|| MonsaktiError::ApiError("No data".to_string()))
}

/// Endpoint: /API/PEM/sppHeader/KLxxx/KDSATKER (opsional)/KD_JNS_SPP (opsional)/NO_SPP (opsional)
pub async fn spp_header(client: &mut MonsaktiClient, kode_kl: &str, kdsatker: &str, kd_jns_spp: &str, no_spp: &str) -> Result<serde_json::Value, MonsaktiError> {
    let kl_formatted = format!("KL{}", kode_kl);
    let response = client.fetch("PEM", "sppHeader", vec![kl_formatted, kdsatker.to_string(), kd_jns_spp.to_string(), no_spp.to_string()]).await?;
    response.data.ok_or_else(|| MonsaktiError::ApiError("No data".to_string()))
}

/// Endpoint: /API/PEM/sppPengeluaran/KLxxx/ID_SPP
pub async fn spp_pengeluaran(client: &mut MonsaktiClient, kode_kl: &str, id_spp: &str) -> Result<serde_json::Value, MonsaktiError> {
    let kl_formatted = format!("KL{}", kode_kl);
    let response = client.fetch("PEM", "sppPengeluaran", vec![kl_formatted, id_spp.to_string()]).await?;
    response.data.ok_or_else(|| MonsaktiError::ApiError("No data".to_string()))
}

/// Endpoint: /API/PEM/sppPotongan/KLxxx/ID_SPP
pub async fn spp_potongan(client: &mut MonsaktiClient, kode_kl: &str, id_spp: &str) -> Result<serde_json::Value, MonsaktiError> {
    let kl_formatted = format!("KL{}", kode_kl);
    let response = client.fetch("PEM", "sppPotongan", vec![kl_formatted, id_spp.to_string()]).await?;
    response.data.ok_or_else(|| MonsaktiError::ApiError("No data".to_string()))
}

/// Endpoint: /API/PEM/penerimaSPM/KLxxx/ID_SPP
pub async fn penerima_spm(client: &mut MonsaktiClient, kode_kl: &str, id_spp: &str) -> Result<serde_json::Value, MonsaktiError> {
    let kl_formatted = format!("KL{}", kode_kl);
    let response = client.fetch("PEM", "penerimaSPM", vec![kl_formatted, id_spp.to_string()]).await?;
    response.data.ok_or_else(|| MonsaktiError::ApiError("No data".to_string()))
}
