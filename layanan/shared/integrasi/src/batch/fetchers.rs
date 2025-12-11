/// Module-specific fetchers untuk MonSAKTI dan MySIMKARI
/// Menggunakan storage strategy yang dapat dikonfigurasi
use crate::client::MonsaktiClient;
use crate::error::MonsaktiError;
use crate::monsakti::{adm, ang, ast, ben, glp, kom, pem, per};
use crate::mysimkari::mysimkari;
use crate::storage::StorageStrategy;
use tracing::{debug, error, info, warn};

/// Fetch data ADM GLOBAL (referensi untuk semua satker)
pub async fn fetch_adm_global(
    client: &mut MonsaktiClient,
    storage: &StorageStrategy,
    kode_kl: &str,
) -> Result<(), MonsaktiError> {
    let context = format!("kl_{}", kode_kl);

    info!("📚 Fetching ADM global references...");

    // Ref Bank - Referensi bank untuk semua satker
    if let Ok(data) = adm::ref_bank(client, kode_kl).await {
        info!(
            "✓ Fetched ref_bank: {} records",
            data.as_array().map(|a| a.len()).unwrap_or(0)
        );
        storage
            .save(client, "adm", "ref_bank", &data, &context)
            .await?;
    } else {
        warn!("⚠ ref_bank: no data");
    }

    // Ref Jenis SPP - Referensi jenis SPP
    if let Ok(data) = adm::ref_jns_spp(client, kode_kl, "").await {
        info!(
            "✓ Fetched ref_jns_spp: {} records",
            data.as_array().map(|a| a.len()).unwrap_or(0)
        );
        storage
            .save(client, "adm", "ref_jns_spp", &data, &context)
            .await?;
    } else {
        warn!("⚠ ref_jns_spp: no data");
    }

    // Ref Uraian - Referensi uraian program s.d. komponen
    // Fetch untuk beberapa jenis yang umum
    for jenis in &["program", "kegiatan", "output", "akun"] {
        if let Ok(mut data) = adm::ref_uraian(client, kode_kl, jenis, "").await {
            // Tambahkan field jenis ke setiap record
            if let Some(array) = data.as_array_mut() {
                for item in array.iter_mut() {
                    if let Some(obj) = item.as_object_mut() {
                        obj.insert(
                            "jenis".to_string(),
                            serde_json::Value::String(jenis.to_string()),
                        );
                    }
                }
            }
            info!(
                "✓ Fetched ref_uraian ({}): {} records",
                jenis,
                data.as_array().map(|a| a.len()).unwrap_or(0)
            );
            storage
                .save(client, "adm", "ref_uraian", &data, &context)
                .await?;
        } else {
            warn!("⚠ ref_uraian ({}): no data", jenis);
        }
    }

    // Ref Aset - Referensi pengkodean aset
    // Fetch untuk beberapa jenis yang umum
    for jenis in &["KDGOL", "KDBID", "KDKEL"] {
        if let Ok(mut data) = adm::ref_aset(client, kode_kl, jenis, "").await {
            // Tambahkan field jenis ke setiap record dan normalisasi nama field kode
            if let Some(array) = data.as_array_mut() {
                for item in array.iter_mut() {
                    if let Some(obj) = item.as_object_mut() {
                        obj.insert(
                            "jenis".to_string(),
                            serde_json::Value::String(jenis.to_string()),
                        );
                        // PENTING: API MonSAKTI return field UPPERCASE (KDGOL, KDBID, KDKEL)
                        // Copy ke field 'kode' unified, lalu hapus field asli UPPERCASE
                        if let Some(kode_value) = obj.remove(*jenis) {
                            obj.insert("kode".to_string(), kode_value);
                        }
                    }
                }
            }
            info!(
                "✓ Fetched ref_aset ({}): {} records",
                jenis,
                data.as_array().map(|a| a.len()).unwrap_or(0)
            );
            storage
                .save(client, "adm", "ref_aset", &data, &context)
                .await?;
        } else {
            warn!("⚠ ref_aset ({}): no data", jenis);
        }
    }

    // Pejabat - GLOBAL (bukan per satker)
    match adm::pejabat(client, kode_kl, "").await {
        Ok(data) => {
            info!(
                "✓ Fetched pejabat: {} records",
                data.as_array().map(|a| a.len()).unwrap_or(0)
            );
            storage
                .save(client, "adm", "pejabat", &data, &context)
                .await?;
        }
        Err(e) => {
            error!("✗ Failed to fetch pejabat: {}", e);
        }
    }

    // Ref Admin - GLOBAL (bukan per satker)
    match adm::ref_admin(client, kode_kl, "").await {
        Ok(data) => {
            info!(
                "✓ Fetched ref_admin: {} records",
                data.as_array().map(|a| a.len()).unwrap_or(0)
            );
            storage
                .save(client, "adm", "ref_admin", &data, &context)
                .await?;
        }
        Err(e) => {
            error!("✗ Failed to fetch ref_admin: {}", e);
        }
    }

    info!("✅ ADM global references completed");
    Ok(())
}

/// Fetch data ADM untuk satker specific
/// NOTE: Untuk saat ini ADM tidak memiliki data per-satker
/// Semua data ADM (ref_bank, ref_jns_spp, ref_uraian, ref_aset, pejabat, ref_admin) adalah GLOBAL
pub async fn fetch_adm(
    _client: &mut MonsaktiClient,
    _storage: &StorageStrategy,
    _kode_kl: &str,
    _kdsatker: &str,
) -> Result<(), MonsaktiError> {
    // ADM module tidak memiliki data per-satker
    // Semua data sudah di-fetch di fetch_adm_global()
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
        if let Some(array) = data.as_array()
            && let Some(latest) = array.last()
            && let Some(obj) = latest.as_object()
            && let Some(sts) = obj.get("KODE_STS_HISTORY")
            && let Some(sts_str) = sts.as_str()
        {
            kode_sts_history = sts_str.to_string();
        }
        storage
            .save(client, "ang", "ref_sts", &data, &context)
            .await?;
    }

    // Data Ang
    if let Ok(data) = ang::data_ang(client, kode_kl, kdsatker, &kode_sts_history).await {
        storage
            .save(client, "ang", "data_ang", &data, &context)
            .await?;
    }

    // Pendapatan
    if let Ok(data) = ang::pendapatan(client, kode_kl, kdsatker, &kode_sts_history).await {
        storage
            .save(client, "ang", "pendapatan", &data, &context)
            .await?;
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

    // Kas Tunai
    if let Ok(data) = ben::kas_tunai(client, kode_kl, kdsatker).await {
        storage
            .save(client, "ben", "kas_tunai", &data, &context)
            .await?;
    }

    // Kas Bank
    if let Ok(data) = ben::kas_bank(client, kode_kl, kdsatker).await {
        storage
            .save(client, "ben", "kas_bank", &data, &context)
            .await?;
    }

    // SPBY
    if let Ok(data) = ben::spby(client, kode_kl, kdsatker).await {
        storage.save(client, "ben", "spby", &data, &context).await?;
    }

    // Kuitansi
    if let Ok(data) = ben::kuitansi(client, kode_kl, kdsatker).await {
        storage
            .save(client, "ben", "kuitansi", &data, &context)
            .await?;
    }

    // DRPP
    if let Ok(data) = ben::drpp(client, kode_kl, kdsatker).await {
        storage.save(client, "ben", "drpp", &data, &context).await?;
    }

    // Pungut Pajak
    if let Ok(data) = ben::pungut_pajak(client, kode_kl, kdsatker).await {
        storage
            .save(client, "ben", "pungut_pajak", &data, &context)
            .await?;
    }

    // Setor Pajak
    if let Ok(data) = ben::setor_pajak(client, kode_kl, kdsatker).await {
        storage
            .save(client, "ben", "setor_pajak", &data, &context)
            .await?;
    }

    // PNBP
    if let Ok(data) = ben::pnbp(client, kode_kl, kdsatker).await {
        storage.save(client, "ben", "pnbp", &data, &context).await?;
    }

    // TUP
    if let Ok(data) = ben::tup(client, kode_kl, kdsatker).await {
        storage.save(client, "ben", "tup", &data, &context).await?;
    }

    // Pengembalian
    if let Ok(data) = ben::pengembalian(client, kode_kl, kdsatker).await {
        storage
            .save(client, "ben", "pengembalian", &data, &context)
            .await?;
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
        storage
            .save(client, "pem", "realisasi", &data, &context)
            .await?;
    }

    // SPP Header
    if let Ok(data) = pem::spp_header(client, kode_kl, kdsatker, "", "").await {
        storage
            .save(client, "pem", "spp_header", &data, &context)
            .await?;
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
        storage
            .save(client, "kom", "kontrak_header", &data, &context)
            .await?;
    }

    // Capaian RO
    if let Ok(data) = kom::capaian_ro(client, kode_kl, kdsatker, &current_period).await {
        storage
            .save(client, "kom", "capaian_ro", &data, &context)
            .await?;
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

    info!("🔍 [AST] Fetching aset_trx for satker {}", kdsatker);

    // Fetch data tanpa parameter golongan - langsung ke satker saja
    match ast::aset_trx(client, kode_kl, kdsatker, "", "", "", "", "").await {
        Ok(data) => {
            if let Some(arr) = data.as_array() {
                let count = arr.len();

                info!(
                    "📊 [AST] Received {} records for satker {}",
                    count, kdsatker
                );

                if count > 0 {
                    info!(
                        "💾 [AST] Attempting to save {} records to database...",
                        count
                    );
                    match storage
                        .save(client, "ast", "aset_trx", &data, &context)
                        .await
                    {
                        Ok(_) => {
                            info!(
                                "✅ [AST] Successfully saved {} records for satker {}",
                                count, kdsatker
                            );
                        }
                        Err(e) => {
                            error!(
                                "❌ [AST] Failed to save records for satker {}: {}",
                                kdsatker, e
                            );
                        }
                    }
                } else {
                    warn!("⚠️  [AST] Empty array for satker {}", kdsatker);
                }
            } else {
                warn!("⚠️  [AST] Response is not an array for satker {}", kdsatker);
            }
        }
        Err(e) => {
            error!(
                "❌ [AST] Failed to fetch aset_trx for satker {}: {}",
                kdsatker, e
            );
        }
    }

    Ok(())
}

pub async fn fetch_per(
    client: &mut MonsaktiClient,
    storage: &StorageStrategy,
    kode_kl: &str,
    kdsatker: &str,
) -> Result<(), MonsaktiError> {
    let context = format!("satker_{}", kdsatker);

    info!("🔍 [PER] Fetching persedia_trx for satker {}", kdsatker);

    match per::persedia_trx(client, kode_kl, kdsatker, "", "", "", "", "").await {
        Ok(data) => {
            // Debug: check data structure
            if let Some(arr) = data.as_array() {
                info!(
                    "📊 [PER] Received {} records for satker {}",
                    arr.len(),
                    kdsatker
                );

                if arr.is_empty() {
                    warn!("⚠️  [PER] Empty data array for satker {}", kdsatker);
                } else {
                    // Show first record structure for debugging
                    if let Some(first) = arr.first() {
                        debug!(
                            "🔎 [PER] First record structure: {}",
                            serde_json::to_string_pretty(first).unwrap_or_default()
                        );
                    }

                    info!(
                        "💾 [PER] Attempting to save {} records to database...",
                        arr.len()
                    );
                    match storage
                        .save(client, "per", "persedia_trx", &data, &context)
                        .await
                    {
                        Ok(_) => info!(
                            "✅ [PER] Successfully saved {} records for satker {}",
                            arr.len(),
                            kdsatker
                        ),
                        Err(e) => error!(
                            "❌ [PER] Failed to save data for satker {}: {}",
                            kdsatker, e
                        ),
                    }
                }
            } else {
                warn!(
                    "⚠️  [PER] Data is not an array for satker {}: {:?}",
                    kdsatker, data
                );
            }
        }
        Err(e) => {
            error!(
                "❌ [PER] Failed to fetch persedia_trx for satker {}: {}",
                kdsatker, e
            );
        }
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
        storage
            .save(client, "glp", "buku_besar", &data, &context)
            .await?;
    }

    // Neraca Sawal
    if let Ok(data) = glp::neraca_sawal(client, kode_kl, kdsatker).await {
        storage
            .save(client, "glp", "neraca_sawal", &data, &context)
            .await?;
    }

    // FA Detail
    if let Ok(data) = glp::fa_detail(client, kode_kl, kdsatker, &current_month).await {
        storage
            .save(client, "glp", "fa_detail", &data, &context)
            .await?;
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
                if let Some(obj) = item.as_object()
                    && let Some(id) = obj.get("id")
                    && let Some(id_str) = id.as_str()
                {
                    match mysimkari::pegawai_satker(client, id_str).await {
                        Ok(mut pegawai_data) => {
                            // Inject satker_id ke setiap pegawai record
                            if let Some(pegawai_array) = pegawai_data.as_array_mut() {
                                for pegawai in pegawai_array {
                                    if let Some(pegawai_obj) = pegawai.as_object_mut() {
                                        // Add satker_id if not exists
                                        if !pegawai_obj.contains_key("satker_id") {
                                            pegawai_obj.insert("satker_id".to_string(), id.clone());
                                        }
                                    }
                                }
                            }

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

    // ADM global references (bank, jns_spp, uraian, aset)
    fetch_adm_global(client, storage, kode_kl).await?;

    // KOM references
    if let Ok(data) = kom::supplier_header(client, kode_kl).await {
        info!(
            "✓ Fetched supplier_header: {} records",
            data.as_array().map(|a| a.len()).unwrap_or(0)
        );
        storage
            .save(client, "kom", "supplier_header", &data, "global")
            .await?;
    } else {
        warn!("⚠ supplier_header: no data");
    }

    info!("✓ Global reference data completed");
    Ok(())
}
