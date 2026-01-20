use crate::client::MonsaktiClient;
use crate::error::MonsaktiError;

/// Endpoint: /API/PER/persediaTrx/KLxxx/KDSATKER/KDGOL (opsional)/KDBID (opsional)/KDKEL (opsional)/KDSKEL (opsional)/KDBRG (opsional)
#[allow(clippy::too_many_arguments)]
pub async fn persedia_trx(
    client: &mut MonsaktiClient,
    kode_kl: &str,
    kdsatker: &str,
    kdgol: &str,
    kdbid: &str,
    kdkel: &str,
    kdskel: &str,
    kdbrg: &str,
) -> Result<serde_json::Value, MonsaktiError> {
    use tracing::info;

    let kl_formatted = format!("KL{}", kode_kl);
    let mut vars = vec![kl_formatted.clone(), kdsatker.to_string()];

    if !kdgol.is_empty() {
        vars.push(kdgol.to_string());
        if !kdbid.is_empty() {
            vars.push(kdbid.to_string());
            if !kdkel.is_empty() {
                vars.push(kdkel.to_string());
                if !kdskel.is_empty() {
                    vars.push(kdskel.to_string());
                    if !kdbrg.is_empty() {
                        vars.push(kdbrg.to_string());
                    }
                }
            }
        }
    }

    let endpoint = format!("/API/PER/persediaTrx/{}", vars.join("/"));
    info!("🌐 [PER API] Calling endpoint: {}", endpoint);

    let response = client.fetch("PER", "persediaTrx", vars).await?;

    match &response.data {
        Some(data) => {
            if let Some(arr) = data.as_array() {
                info!("📥 [PER API] Response received: {} records", arr.len());
            }
            Ok(data.clone())
        }
        None => {
            info!("⚠️  [PER API] Response has no data field");
            Err(MonsaktiError::ApiError("No data".to_string()))
        }
    }
}
