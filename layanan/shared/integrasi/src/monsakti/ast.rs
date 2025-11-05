use crate::client::MonsaktiClient;
use crate::error::MonsaktiError;

/// Endpoint: /API/AST/asetTrx/KLxxx/KDSATKER/KDGOL/KDBID/KDKEL (opsional)/KDSKEL (opsional)/KDBRG (opsional)
pub async fn aset_trx(
    client: &mut MonsaktiClient,
    kode_kl: &str,
    kdsatker: &str,
    kdgol: &str,
    kdbid: &str,
    kdkel: &str,
    kdskel: &str,
    kdbrg: &str,
) -> Result<serde_json::Value, MonsaktiError> {
    let kl_formatted = format!("KL{}", kode_kl);
    let mut vars = vec![
        kl_formatted,
        kdsatker.to_string(),
        kdgol.to_string(),
        kdbid.to_string(),
    ];

    if !kdkel.is_empty() {
        vars.push(kdkel.to_string());
        if !kdskel.is_empty() {
            vars.push(kdskel.to_string());
            if !kdbrg.is_empty() {
                vars.push(kdbrg.to_string());
            }
        }
    }

    let response = client.fetch("AST", "asetTrx", vars).await?;
    response
        .data
        .ok_or_else(|| MonsaktiError::ApiError("No data".to_string()))
}
