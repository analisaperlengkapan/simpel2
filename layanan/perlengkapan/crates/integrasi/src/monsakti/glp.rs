use crate::client::MonsaktiClient;
use crate::error::MonsaktiError;

/// Endpoint: /API/GLP/bukuBesar/KLxxx/KDSATKER/PERIODE/OPSI_FILTER (opsional)
/// OPSI_FILTER: 'BB' untuk filter hanya buku besar atau 'kas' untuk filter hanya jurnal berbasis kas
pub async fn buku_besar(
    client: &mut MonsaktiClient,
    kode_kl: &str,
    kdsatker: &str,
    periode: &str,
    opsi_filter: &str,
) -> Result<serde_json::Value, MonsaktiError> {
    let kl_formatted = format!("KL{}", kode_kl);
    let vars = if opsi_filter.is_empty() {
        vec![kl_formatted, kdsatker.to_string(), periode.to_string()]
    } else {
        vec![
            kl_formatted,
            kdsatker.to_string(),
            periode.to_string(),
            opsi_filter.to_string(),
        ]
    };
    let response = client.fetch("GLP", "bukuBesar", vars).await?;
    response
        .data
        .ok_or_else(|| MonsaktiError::ApiError("No data".to_string()))
}

/// Endpoint: /API/GLP/neracaSawal/KLxxx/KDSATKER
pub async fn neraca_sawal(
    client: &mut MonsaktiClient,
    kode_kl: &str,
    kdsatker: &str,
) -> Result<serde_json::Value, MonsaktiError> {
    let kl_formatted = format!("KL{}", kode_kl);
    let response = client
        .fetch(
            "GLP",
            "neracaSawal",
            vec![kl_formatted, kdsatker.to_string()],
        )
        .await?;
    response
        .data
        .ok_or_else(|| MonsaktiError::ApiError("No data".to_string()))
}

/// Endpoint: /API/GLP/faDetail/KLxxx/KDSATKER/PERIODE
pub async fn fa_detail(
    client: &mut MonsaktiClient,
    kode_kl: &str,
    kdsatker: &str,
    periode: &str,
) -> Result<serde_json::Value, MonsaktiError> {
    let kl_formatted = format!("KL{}", kode_kl);
    let response = client
        .fetch(
            "GLP",
            "faDetail",
            vec![kl_formatted, kdsatker.to_string(), periode.to_string()],
        )
        .await?;
    response
        .data
        .ok_or_else(|| MonsaktiError::ApiError("No data".to_string()))
}
