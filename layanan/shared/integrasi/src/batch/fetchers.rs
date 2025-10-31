/// Module-specific fetchers untuk MonSAKTI dan MySIMKARI
/// Menggunakan storage strategy yang dapat dikonfigurasi

use crate::client::MonsaktiClient;
use crate::error::MonsaktiError;
use crate::monsakti::{adm, ang, ast, ben, glp, kom, pem, per};
use crate::mysimkari::mysimkari;
use crate::storage::StorageStrategy;
use tracing::{info, warn};

/// Fetch data ADM untuk satker
pub async fn fetch_adm(
    client: &mut MonsaktiClient,
    storage: &StorageStrategy,
    kode_kl: &str,
    kdsatker: &str,
) -> Result<(), MonsaktiError> {
    let context = format!("satker_{}", kdsatker);

    // Pejabat
    if let Ok(data) = adm::pejabat(client, kode_kl, kdsatker).await {
        storage.save(client, "adm", "pejabat", &data, &context).await?;
    }

    // Ref Admin
    if let Ok(data) = adm::ref_admin(client, kode_kl, kdsatker).await {
        storage.save(client, "adm", "ref_admin", &data, &context).await?;
    }

    Ok(())
}

/// Fetch data ANG untuk satker
pub async fn fetch_ang(
    client: &mut MonsaktiClient,
    storage: &StorageStrategy,
    kode_kl: &str,
    kdsatker: &str,
) -> Result<(), MonsaktiError> {
    let context = format!("satker_{}", kdsatker);
    let mut kode_sts_history = "B00".to_string();

    // Ref STS
    if let Ok(data) = ang::ref_sts(client, kode_kl, kdsatker).await {
        // Extract latest STS history code
        if let Some(array) = data.as_array() {
            if let Some(latest) = array.last() {
                if let Some(obj) = latest.as_object() {
                    if let Some(sts) = obj.get("KODE_STS_HISTORY") {
                        if let Some(sts_str) = sts.as_str() {
                            kode_sts_history = sts_str.to_string();
                        }
                    }
                }
            }
        }
        storage.save(client, "ang", "ref_sts", &data, &context).await?;
    }

    // Data Ang
    if let Ok(data) = ang::data_ang(client, kode_kl, kdsatker, &kode_sts_history).await {
        storage.save(client, "ang", "data_ang", &data, &context).await?;
    }

    // Pendapatan
    if let Ok(data) = ang::pendapatan(client, kode_kl, kdsatker, &kode_sts_history).await {
        storage.save(client, "ang", "pendapatan", &data, &context).await?;
    }

    Ok(())
}

/// Fetch data BEN untuk satker
pub async fn fetch_ben(
    client: &mut MonsaktiClient,
    storage: &StorageStrategy,
    kode_kl: &str,
    kdsatker: &str,
) -> Result<(), MonsaktiError> {
    let context = format!("satker_{}", kdsatker);

    let endpoints: Vec<(&str, fn(&mut MonsaktiClient, &str, &str) -> _)> = vec![
        ("kas_tunai", ben::kas_tunai),
        ("kas_bank", ben::kas_bank),
        ("spby", ben::spby),
        ("kuitansi", ben::kuitansi),
        ("drpp", ben::drpp),
        ("pungut_pajak", ben::pungut_pajak),
        ("setor_pajak", ben::setor_pajak),
        ("pnbp", ben::pnbp),
        ("tup", ben::tup),
        ("pengembalian", ben::pengembalian),
    ];

    for (endpoint_name, fetch_fn) in endpoints {
        if let Ok(data) = fetch_fn(client, kode_kl, kdsatker).await {
            storage.save(client, "ben", endpoint_name, &data, &context).await?;
        }
    }

    Ok(())
}

/// Fetch data PEM untuk satker
pub async fn fetch_pem(
    client: &mut MonsaktiClient,
    storage: &StorageStrategy,
    kode_kl: &str,
    kdsatker: &str,
) -> Result<(), MonsaktiError> {
    let context = format!("satker_{}", kdsatker);

    // Realisasi
    if let Ok(data) = pem::realisasi(client, kode_kl, kdsatker, "", "").await {
        storage.save(client, "pem", "realisasi", &data, &context).await?;
    }

    // SPP Header
    if let Ok(data) = pem::spp_header(client, kode_kl, kdsatker, "", "").await {
        storage.save(client, "pem", "spp_header", &data, &context).await?;
    }

    Ok(())
}

/// Fetch data KOM untuk satker
pub async fn fetch_kom(
    client: &mut MonsaktiClient,
    storage: &StorageStrategy,
    kode_kl: &str,
    kdsatker: &str,
) -> Result<(), MonsaktiError> {
    let context = format!("satker_{}", kdsatker);
    let current_period = chrono::Local::now().format("%Y-%m").to_string();

    // Kontrak Header
    if let Ok(data) = kom::kontrak_header(client, kode_kl, kdsatker).await {
        storage.save(client, "kom", "kontrak_header", &data, &context).await?;
    }

    // Capaian RO
    if let Ok(data) = kom::capaian_ro(client, kode_kl, kdsatker, &current_period).await {
        storage.save(client, "kom", "capaian_ro", &data, &context).await?;
    }

    Ok(())
}

/// Fetch data AST untuk satker
pub async fn fetch_ast(
    client: &mut MonsaktiClient,
    storage: &StorageStrategy,
    kode_kl: &str,
    kdsatker: &str,
) -> Result<(), MonsaktiError> {
    let context = format!("satker_{}", kdsatker);
    let golongan_aset = vec!["01", "02", "03", "04", "05", "06", "07"];

    for kdgol in golongan_aset {
        if let Ok(data) = ast::aset_trx(client, kode_kl, kdsatker, kdgol, "", "", "", "").await {
            storage.save(client, "ast", "aset_trx", &data, &context).await?;
        }
    }

    Ok(())
}

/// Fetch data PER untuk satker
pub async fn fetch_per(
    client: &mut MonsaktiClient,
    storage: &StorageStrategy,
    kode_kl: &str,
    kdsatker: &str,
) -> Result<(), MonsaktiError> {
    let context = format!("satker_{}", kdsatker);

    if let Ok(data) = per::persedia_trx(client, kode_kl, kdsatker, "", "", "", "", "").await {
        storage.save(client, "per", "persedia_trx", &data, &context).await?;
    }

    Ok(())
}

/// Fetch data GLP untuk satker
pub async fn fetch_glp(
    client: &mut MonsaktiClient,
    storage: &StorageStrategy,
    kode_kl: &str,
    kdsatker: &str,
) -> Result<(), MonsaktiError> {
    let context = format!("satker_{}", kdsatker);
    let current_month = chrono::Local::now().format("%m").to_string();

    // Buku Besar
    if let Ok(data) = glp::buku_besar(client, kode_kl, kdsatker, &current_month, "").await {
        storage.save(client, "glp", "buku_besar", &data, &context).await?;
    }

    // Neraca Sawal
    if let Ok(data) = glp::neraca_sawal(client, kode_kl, kdsatker).await {
        storage.save(client, "glp", "neraca_sawal", &data, &context).await?;
    }

    // FA Detail
    if let Ok(data) = glp::fa_detail(client, kode_kl, kdsatker, &current_month).await {
        storage.save(client, "glp", "fa_detail", &data, &context).await?;
    }

    Ok(())
}

/// Fetch data MySIMKARI (satker dan pegawai)
pub async fn fetch_mysimkari(
    client: &mut MonsaktiClient,
    storage: &StorageStrategy,
) -> Result<(), MonsaktiError> {
    info!("Fetching MySIMKARI data...");

    // Get Satker
    if let Ok(data) = mysimkari::get_satker(client).await {
        storage
            .save_with_table(client, "mysimkari_satker", &data, "global")
            .await?;

        // Fetch pegawai untuk setiap satker
        if let Some(array) = data.as_array() {
            for item in array {
                if let Some(obj) = item.as_object() {
                    if let Some(id) = obj.get("id") {
                        if let Some(id_str) = id.as_str() {
                            match mysimkari::pegawai_satker(client, id_str).await {
                                Ok(pegawai_data) => {
                                    storage
                                        .save_with_table(
                                            client,
                                            "mysimkari_pegawai",
                                            &pegawai_data,
                                            "global",
                                        )
                                        .await?;
                                }
                                Err(e) => {
                                    warn!("MySIMKARI pegawai_satker failed for {}: {}", id_str, e)
                                }
                            }
                            // Rate limiting
                            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                        }
                    }
                }
            }
        }
    }

    info!("✓ MySIMKARI data fetch completed");
    Ok(())
}

/// Fetch global reference data (tidak spesifik per satker)
pub async fn fetch_global_references(
    client: &mut MonsaktiClient,
    storage: &StorageStrategy,
    kode_kl: &str,
) -> Result<(), MonsaktiError> {
    info!("Fetching global reference data...");

    // ADM references
    if let Ok(data) = adm::ref_admin(client, kode_kl, "").await {
        storage.save(client, "adm", "ref_admin", &data, "global").await?;
    }

    if let Ok(data) = adm::ref_uraian(client, kode_kl, "program", "").await {
        storage
            .save(client, "adm", "ref_uraian_program", &data, "global")
            .await?;
    }

    if let Ok(data) = adm::ref_uraian(client, kode_kl, "akun", "").await {
        storage
            .save(client, "adm", "ref_uraian_akun", &data, "global")
            .await?;
    }

    if let Ok(data) = adm::ref_bank(client, kode_kl).await {
        storage.save(client, "adm", "ref_bank", &data, "global").await?;
    }

    if let Ok(data) = adm::ref_jns_spp(client, kode_kl, "").await {
        storage
            .save(client, "adm", "ref_jns_spp", &data, "global")
            .await?;
    }

    // KOM references
    if let Ok(data) = kom::supplier_header(client, kode_kl).await {
        storage
            .save(client, "kom", "supplier_header", &data, "global")
            .await?;
    }

    info!("✓ Global reference data completed");
    Ok(())
}
