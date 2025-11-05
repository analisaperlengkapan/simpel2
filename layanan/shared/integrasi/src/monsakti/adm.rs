use crate::client::MonsaktiClient;
use crate::error::MonsaktiError;

/// Endpoint: /API/ADM/refUraian/KLxxx/jenis/kode (opsional)
/// Jenis: program, kegiatan, output, suboutput, komponen, akun
pub async fn ref_uraian(
    client: &mut MonsaktiClient,
    kode_kl: &str,
    jenis: &str,
    kode: &str,
) -> Result<serde_json::Value, MonsaktiError> {
    let kl_formatted = format!("KL{}", kode_kl);
    let vars = if kode.is_empty() {
        vec![kl_formatted, jenis.to_string()]
    } else {
        vec![kl_formatted, jenis.to_string(), kode.to_string()]
    };
    let response = client.fetch("ADM", "refUraian", vars).await?;
    response
        .data
        .ok_or_else(|| MonsaktiError::ApiError("No data".to_string()))
}

/// Endpoint: /API/ADM/refAdmin/KLxxx/KDSATKER (opsional)
pub async fn ref_admin(
    client: &mut MonsaktiClient,
    kode_kl: &str,
    kdsatker: &str,
) -> Result<serde_json::Value, MonsaktiError> {
    let kl_formatted = format!("KL{}", kode_kl);
    let response = client
        .fetch("ADM", "refAdmin", vec![kl_formatted, kdsatker.to_string()])
        .await?;
    response
        .data
        .ok_or_else(|| MonsaktiError::ApiError("No data".to_string()))
}

/// Endpoint: /API/ADM/pejabat/KLxxx/KDSATKER (opsional)
pub async fn pejabat(
    client: &mut MonsaktiClient,
    kode_kl: &str,
    kdsatker: &str,
) -> Result<serde_json::Value, MonsaktiError> {
    let kl_formatted = format!("KL{}", kode_kl);
    let response = client
        .fetch("ADM", "pejabat", vec![kl_formatted, kdsatker.to_string()])
        .await?;
    response
        .data
        .ok_or_else(|| MonsaktiError::ApiError("No data".to_string()))
}

/// Endpoint: /API/ADM/refBank/KLxxx
pub async fn ref_bank(
    client: &mut MonsaktiClient,
    kode_kl: &str,
) -> Result<serde_json::Value, MonsaktiError> {
    let kl_formatted = format!("KL{}", kode_kl);
    let response = client.fetch("ADM", "refBank", vec![kl_formatted]).await?;
    response
        .data
        .ok_or_else(|| MonsaktiError::ApiError("No data".to_string()))
}

/// Endpoint: /API/ADM/refJnsSPP/KLxxx/KODE (opsional)
pub async fn ref_jns_spp(
    client: &mut MonsaktiClient,
    kode_kl: &str,
    kode: &str,
) -> Result<serde_json::Value, MonsaktiError> {
    let kl_formatted = format!("KL{}", kode_kl);
    let vars = if kode.is_empty() {
        vec![kl_formatted]
    } else {
        vec![kl_formatted, kode.to_string()]
    };
    let response = client.fetch("ADM", "refJnsSPP", vars).await?;
    response
        .data
        .ok_or_else(|| MonsaktiError::ApiError("No data".to_string()))
}

/// Endpoint: /API/ADM/refAset/KLxxx/jenis/kode (opsional)
/// Jenis: KDTRX, KDGOL, KDBID, KDKEL, KDSKEL, KDBRG
pub async fn ref_aset(
    client: &mut MonsaktiClient,
    kode_kl: &str,
    jenis: &str,
    kode: &str,
) -> Result<serde_json::Value, MonsaktiError> {
    let kl_formatted = format!("KL{}", kode_kl);
    let vars = if kode.is_empty() {
        vec![kl_formatted, jenis.to_string()]
    } else {
        vec![kl_formatted, jenis.to_string(), kode.to_string()]
    };
    let response = client.fetch("ADM", "refAset", vars).await?;
    response
        .data
        .ok_or_else(|| MonsaktiError::ApiError("No data".to_string()))
}
