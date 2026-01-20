use crate::client::MonsaktiClient;
use crate::error::MonsaktiError;

/// Endpoint: /API/BEN/kasTunai/KLxxx/KDSATKER
pub async fn kas_tunai(
    client: &mut MonsaktiClient,
    kode_kl: &str,
    kdsatker: &str,
) -> Result<serde_json::Value, MonsaktiError> {
    let kl_formatted = format!("KL{}", kode_kl);
    let response = client
        .fetch("BEN", "kasTunai", vec![kl_formatted, kdsatker.to_string()])
        .await?;
    response
        .data
        .ok_or_else(|| MonsaktiError::ApiError("No data".to_string()))
}

/// Endpoint: /API/BEN/kasBank/KLxxx/KDSATKER
pub async fn kas_bank(
    client: &mut MonsaktiClient,
    kode_kl: &str,
    kdsatker: &str,
) -> Result<serde_json::Value, MonsaktiError> {
    let kl_formatted = format!("KL{}", kode_kl);
    let response = client
        .fetch("BEN", "kasBank", vec![kl_formatted, kdsatker.to_string()])
        .await?;
    response
        .data
        .ok_or_else(|| MonsaktiError::ApiError("No data".to_string()))
}

/// Endpoint: /API/BEN/spby/KLxxx/KDSATKER
pub async fn spby(
    client: &mut MonsaktiClient,
    kode_kl: &str,
    kdsatker: &str,
) -> Result<serde_json::Value, MonsaktiError> {
    let kl_formatted = format!("KL{}", kode_kl);
    let response = client
        .fetch("BEN", "spby", vec![kl_formatted, kdsatker.to_string()])
        .await?;
    response
        .data
        .ok_or_else(|| MonsaktiError::ApiError("No data".to_string()))
}

/// Endpoint: /API/BEN/kuitansi/KLxxx/KDSATKER
pub async fn kuitansi(
    client: &mut MonsaktiClient,
    kode_kl: &str,
    kdsatker: &str,
) -> Result<serde_json::Value, MonsaktiError> {
    let kl_formatted = format!("KL{}", kode_kl);
    let response = client
        .fetch("BEN", "kuitansi", vec![kl_formatted, kdsatker.to_string()])
        .await?;
    response
        .data
        .ok_or_else(|| MonsaktiError::ApiError("No data".to_string()))
}

/// Endpoint: /API/BEN/drpp/KLxxx/KDSATKER
pub async fn drpp(
    client: &mut MonsaktiClient,
    kode_kl: &str,
    kdsatker: &str,
) -> Result<serde_json::Value, MonsaktiError> {
    let kl_formatted = format!("KL{}", kode_kl);
    let response = client
        .fetch("BEN", "drpp", vec![kl_formatted, kdsatker.to_string()])
        .await?;
    response
        .data
        .ok_or_else(|| MonsaktiError::ApiError("No data".to_string()))
}

/// Endpoint: /API/BEN/pungutPajak/KLxxx/KDSATKER
pub async fn pungut_pajak(
    client: &mut MonsaktiClient,
    kode_kl: &str,
    kdsatker: &str,
) -> Result<serde_json::Value, MonsaktiError> {
    let kl_formatted = format!("KL{}", kode_kl);
    let response = client
        .fetch(
            "BEN",
            "pungutPajak",
            vec![kl_formatted, kdsatker.to_string()],
        )
        .await?;
    response
        .data
        .ok_or_else(|| MonsaktiError::ApiError("No data".to_string()))
}

/// Endpoint: /API/BEN/setorPajak/KLxxx/KDSATKER
pub async fn setor_pajak(
    client: &mut MonsaktiClient,
    kode_kl: &str,
    kdsatker: &str,
) -> Result<serde_json::Value, MonsaktiError> {
    let kl_formatted = format!("KL{}", kode_kl);
    let response = client
        .fetch(
            "BEN",
            "setorPajak",
            vec![kl_formatted, kdsatker.to_string()],
        )
        .await?;
    response
        .data
        .ok_or_else(|| MonsaktiError::ApiError("No data".to_string()))
}

/// Endpoint: /API/BEN/pnbp/KLxxx/KDSATKER
pub async fn pnbp(
    client: &mut MonsaktiClient,
    kode_kl: &str,
    kdsatker: &str,
) -> Result<serde_json::Value, MonsaktiError> {
    let kl_formatted = format!("KL{}", kode_kl);
    let response = client
        .fetch("BEN", "pnbp", vec![kl_formatted, kdsatker.to_string()])
        .await?;
    response
        .data
        .ok_or_else(|| MonsaktiError::ApiError("No data".to_string()))
}

/// Endpoint: /API/BEN/tup/KLxxx/KDSATKER
pub async fn tup(
    client: &mut MonsaktiClient,
    kode_kl: &str,
    kdsatker: &str,
) -> Result<serde_json::Value, MonsaktiError> {
    let kl_formatted = format!("KL{}", kode_kl);
    let response = client
        .fetch("BEN", "tup", vec![kl_formatted, kdsatker.to_string()])
        .await?;
    response
        .data
        .ok_or_else(|| MonsaktiError::ApiError("No data".to_string()))
}

/// Endpoint: /API/BEN/pengembalian/KLxxx/KDSATKER
pub async fn pengembalian(
    client: &mut MonsaktiClient,
    kode_kl: &str,
    kdsatker: &str,
) -> Result<serde_json::Value, MonsaktiError> {
    let kl_formatted = format!("KL{}", kode_kl);
    let response = client
        .fetch(
            "BEN",
            "pengembalian",
            vec![kl_formatted, kdsatker.to_string()],
        )
        .await?;
    response
        .data
        .ok_or_else(|| MonsaktiError::ApiError("No data".to_string()))
}
