use crate::client::MonsaktiClient;
use crate::error::MonsaktiError;

/// Endpoint: /API/KOM/capaianRO/KLxxx/KDSATKER (opsional)/KODE_PERIODE (opsional)
pub async fn capaian_ro(
    client: &mut MonsaktiClient,
    kode_kl: &str,
    kdsatker: &str,
    kode_periode: &str,
) -> Result<serde_json::Value, MonsaktiError> {
    let kl_formatted = format!("KL{}", kode_kl);
    let response = client
        .fetch(
            "KOM",
            "capaianRO",
            vec![kl_formatted, kdsatker.to_string(), kode_periode.to_string()],
        )
        .await?;
    response
        .data
        .ok_or_else(|| MonsaktiError::ApiError("No data".to_string()))
}

/// Endpoint: /API/KOM/kontrakHeader/KLxxx/KDSATKER
pub async fn kontrak_header(
    client: &mut MonsaktiClient,
    kode_kl: &str,
    kdsatker: &str,
) -> Result<serde_json::Value, MonsaktiError> {
    let kl_formatted = format!("KL{}", kode_kl);
    let response = client
        .fetch(
            "KOM",
            "kontrakHeader",
            vec![kl_formatted, kdsatker.to_string()],
        )
        .await?;
    response
        .data
        .ok_or_else(|| MonsaktiError::ApiError("No data".to_string()))
}

/// Endpoint: /API/KOM/kontrakLine/KLxxx/KDSATKER/ID_KONTRAK (opsional)/ID_LINE_KONTRAK (opsional)
pub async fn kontrak_line(
    client: &mut MonsaktiClient,
    kode_kl: &str,
    kdsatker: &str,
    id_kontrak: &str,
    id_line_kontrak: &str,
) -> Result<serde_json::Value, MonsaktiError> {
    let kl_formatted = format!("KL{}", kode_kl);
    let response = client
        .fetch(
            "KOM",
            "kontrakLine",
            vec![
                kl_formatted,
                kdsatker.to_string(),
                id_kontrak.to_string(),
                id_line_kontrak.to_string(),
            ],
        )
        .await?;
    response
        .data
        .ok_or_else(|| MonsaktiError::ApiError("No data".to_string()))
}

/// Endpoint: /API/KOM/kontrakTermin/KLxxx/KDSATKER/ID_KONTRAK (opsional)/ID_LINE_KONTRAK (opsional)/ID_JADWAL_PEMBAYARAN (opsional)
pub async fn kontrak_termin(
    client: &mut MonsaktiClient,
    kode_kl: &str,
    kdsatker: &str,
    id_kontrak: &str,
    id_line_kontrak: &str,
    id_jadwal_pembayaran: &str,
) -> Result<serde_json::Value, MonsaktiError> {
    let kl_formatted = format!("KL{}", kode_kl);
    let response = client
        .fetch(
            "KOM",
            "kontrakTermin",
            vec![
                kl_formatted,
                kdsatker.to_string(),
                id_kontrak.to_string(),
                id_line_kontrak.to_string(),
                id_jadwal_pembayaran.to_string(),
            ],
        )
        .await?;
    response
        .data
        .ok_or_else(|| MonsaktiError::ApiError("No data".to_string()))
}

/// Endpoint: /API/KOM/kontrakCOA/KLxxx/KDSATKER/ID_KONTRAK (opsional)/ID_LINE_KONTRAK (opsional)/ID_JADWAL_PEMBAYARAN (opsional)
pub async fn kontrak_coa(
    client: &mut MonsaktiClient,
    kode_kl: &str,
    kdsatker: &str,
    id_kontrak: &str,
    id_line_kontrak: &str,
    id_jadwal_pembayaran: &str,
) -> Result<serde_json::Value, MonsaktiError> {
    let kl_formatted = format!("KL{}", kode_kl);
    let response = client
        .fetch(
            "KOM",
            "kontrakCOA",
            vec![
                kl_formatted,
                kdsatker.to_string(),
                id_kontrak.to_string(),
                id_line_kontrak.to_string(),
                id_jadwal_pembayaran.to_string(),
            ],
        )
        .await?;
    response
        .data
        .ok_or_else(|| MonsaktiError::ApiError("No data".to_string()))
}

/// Endpoint: /API/KOM/BASTKontrakHeader/KLxxx/KDSATKER/ID_BAST (opsional)
pub async fn bast_kontrak_header(
    client: &mut MonsaktiClient,
    kode_kl: &str,
    kdsatker: &str,
    id_bast: &str,
) -> Result<serde_json::Value, MonsaktiError> {
    let kl_formatted = format!("KL{}", kode_kl);
    let vars = if id_bast.is_empty() {
        vec![kl_formatted, kdsatker.to_string()]
    } else {
        vec![kl_formatted, kdsatker.to_string(), id_bast.to_string()]
    };
    let response = client.fetch("KOM", "BASTKontrakHeader", vars).await?;
    response
        .data
        .ok_or_else(|| MonsaktiError::ApiError("No data".to_string()))
}

/// Endpoint: /API/KOM/BASTKontrakDetailBarang/KLxxx/KDSATKER/ID_BAST (opsional)
pub async fn bast_kontrak_detail_barang(
    client: &mut MonsaktiClient,
    kode_kl: &str,
    kdsatker: &str,
    id_bast: &str,
) -> Result<serde_json::Value, MonsaktiError> {
    let kl_formatted = format!("KL{}", kode_kl);
    let vars = if id_bast.is_empty() {
        vec![kl_formatted, kdsatker.to_string()]
    } else {
        vec![kl_formatted, kdsatker.to_string(), id_bast.to_string()]
    };
    let response = client.fetch("KOM", "BASTKontrakDetailBarang", vars).await?;
    response
        .data
        .ok_or_else(|| MonsaktiError::ApiError("No data".to_string()))
}

/// Endpoint: /API/KOM/BASTKontrakCOA/KLxxx/KDSATKER/ID_BAST (opsional)
pub async fn bast_kontrak_coa(
    client: &mut MonsaktiClient,
    kode_kl: &str,
    kdsatker: &str,
    id_bast: &str,
) -> Result<serde_json::Value, MonsaktiError> {
    let kl_formatted = format!("KL{}", kode_kl);
    let vars = if id_bast.is_empty() {
        vec![kl_formatted, kdsatker.to_string()]
    } else {
        vec![kl_formatted, kdsatker.to_string(), id_bast.to_string()]
    };
    let response = client.fetch("KOM", "BASTKontrakCOA", vars).await?;
    response
        .data
        .ok_or_else(|| MonsaktiError::ApiError("No data".to_string()))
}

/// Endpoint: /API/KOM/BASTNonKontrakHeader/KLxxx/KDSATKER/ID_BAST (opsional)
pub async fn bast_non_kontrak_header(
    client: &mut MonsaktiClient,
    kode_kl: &str,
    kdsatker: &str,
    id_bast: &str,
) -> Result<serde_json::Value, MonsaktiError> {
    let kl_formatted = format!("KL{}", kode_kl);
    let vars = if id_bast.is_empty() {
        vec![kl_formatted, kdsatker.to_string()]
    } else {
        vec![kl_formatted, kdsatker.to_string(), id_bast.to_string()]
    };
    let response = client.fetch("KOM", "BASTNonKontrakHeader", vars).await?;
    response
        .data
        .ok_or_else(|| MonsaktiError::ApiError("No data".to_string()))
}

/// Endpoint: /API/KOM/BASTNonKontrakDetailBarang/KLxxx/KDSATKER/ID_BAST (opsional)
pub async fn bast_non_kontrak_detail_barang(
    client: &mut MonsaktiClient,
    kode_kl: &str,
    kdsatker: &str,
    id_bast: &str,
) -> Result<serde_json::Value, MonsaktiError> {
    let kl_formatted = format!("KL{}", kode_kl);
    let vars = if id_bast.is_empty() {
        vec![kl_formatted, kdsatker.to_string()]
    } else {
        vec![kl_formatted, kdsatker.to_string(), id_bast.to_string()]
    };
    let response = client
        .fetch("KOM", "BASTNonKontrakDetailBarang", vars)
        .await?;
    response
        .data
        .ok_or_else(|| MonsaktiError::ApiError("No data".to_string()))
}

/// Endpoint: /API/KOM/BASTNonKontrakCOA/KLxxx/KDSATKER/ID_BAST (opsional)
pub async fn bast_non_kontrak_coa(
    client: &mut MonsaktiClient,
    kode_kl: &str,
    kdsatker: &str,
    id_bast: &str,
) -> Result<serde_json::Value, MonsaktiError> {
    let kl_formatted = format!("KL{}", kode_kl);
    let vars = if id_bast.is_empty() {
        vec![kl_formatted, kdsatker.to_string()]
    } else {
        vec![kl_formatted, kdsatker.to_string(), id_bast.to_string()]
    };
    let response = client.fetch("KOM", "BASTNonKontrakCOA", vars).await?;
    response
        .data
        .ok_or_else(|| MonsaktiError::ApiError("No data".to_string()))
}

/// Endpoint: /API/KOM/supplierHeader/KLxxx
pub async fn supplier_header(
    client: &mut MonsaktiClient,
    kode_kl: &str,
) -> Result<serde_json::Value, MonsaktiError> {
    let kl_formatted = format!("KL{}", kode_kl);
    let response = client
        .fetch("KOM", "supplierHeader", vec![kl_formatted])
        .await?;
    response
        .data
        .ok_or_else(|| MonsaktiError::ApiError("No data".to_string()))
}

/// Endpoint: /API/KOM/supplierAddress/KLxxx/ID_SUPPLIER
pub async fn supplier_address(
    client: &mut MonsaktiClient,
    kode_kl: &str,
    id_supplier: &str,
) -> Result<serde_json::Value, MonsaktiError> {
    let kl_formatted = format!("KL{}", kode_kl);
    let response = client
        .fetch(
            "KOM",
            "supplierAddress",
            vec![kl_formatted, id_supplier.to_string()],
        )
        .await?;
    response
        .data
        .ok_or_else(|| MonsaktiError::ApiError("No data".to_string()))
}

/// Endpoint: /API/KOM/supplierBank/KLxxx/ID_SUPPLIER/ID_SUPPLIER_ADDRESS/NAMA_BANK (opsional)
pub async fn supplier_bank(
    client: &mut MonsaktiClient,
    kode_kl: &str,
    id_supplier: &str,
    id_supplier_address: &str,
    nama_bank: &str,
) -> Result<serde_json::Value, MonsaktiError> {
    let kl_formatted = format!("KL{}", kode_kl);
    let vars = if nama_bank.is_empty() {
        vec![
            kl_formatted,
            id_supplier.to_string(),
            id_supplier_address.to_string(),
        ]
    } else {
        vec![
            kl_formatted,
            id_supplier.to_string(),
            id_supplier_address.to_string(),
            nama_bank.to_string(),
        ]
    };
    let response = client.fetch("KOM", "supplierBank", vars).await?;
    response
        .data
        .ok_or_else(|| MonsaktiError::ApiError("No data".to_string()))
}
