/// Orchestrator untuk batch processing
/// Mengkoordinasikan fetching data dari multiple sources
use crate::batch::fetchers::*;
use crate::client::MonsaktiClient;
use crate::error::MonsaktiError;
use crate::monsakti::adm;
use crate::storage::StorageStrategy;
use tracing::{error, info};

/// Get list of all satker untuk KL tertentu
pub async fn get_satker_list(
    client: &mut MonsaktiClient,
    kode_kl: &str,
) -> Result<Vec<String>, MonsaktiError> {
    info!("Fetching list of all satker for KL{}...", kode_kl);
    let data = adm::ref_admin(client, kode_kl, "").await?;
    let mut satker_list = Vec::new();

    if let Some(array) = data.as_array() {
        for item in array {
            if let Some(obj) = item.as_object() {
                if let Some(kdsatker) = obj.get("KDSATKER") {
                    if let Some(kdsatker_str) = kdsatker.as_str() {
                        if !kdsatker_str.is_empty() && kdsatker_str != "000000" {
                            satker_list.push(kdsatker_str.to_string());
                        }
                    }
                }
            }
        }
    }

    info!("Found {} satker(s)", satker_list.len());
    Ok(satker_list)
}

/// Fetch semua data MonSAKTI untuk satu satker
pub async fn fetch_satker_complete(
    client: &mut MonsaktiClient,
    storage: &StorageStrategy,
    kode_kl: &str,
    kdsatker: &str,
) -> Result<(), MonsaktiError> {
    info!("[{}] Processing satker...", kdsatker);

    // Fetch semua modul secara sequential untuk menghindari rate limiting
    let _ = fetch_adm(client, storage, kode_kl, kdsatker).await;
    let _ = fetch_ang(client, storage, kode_kl, kdsatker).await;
    let _ = fetch_ben(client, storage, kode_kl, kdsatker).await;
    let _ = fetch_pem(client, storage, kode_kl, kdsatker).await;
    let _ = fetch_kom(client, storage, kode_kl, kdsatker).await;
    let _ = fetch_ast(client, storage, kode_kl, kdsatker).await;
    let _ = fetch_per(client, storage, kode_kl, kdsatker).await;
    let _ = fetch_glp(client, storage, kode_kl, kdsatker).await;

    info!("[{}] ✓ Completed", kdsatker);
    Ok(())
}

/// Fetch semua satker untuk KL tertentu
pub async fn fetch_all_satker(
    client: &mut MonsaktiClient,
    storage: &StorageStrategy,
    kode_kl: &str,
) -> Result<(), MonsaktiError> {
    let satker_list = get_satker_list(client, kode_kl).await?;

    info!(
        "Processing {} satker(s) for KL{}...",
        satker_list.len(),
        kode_kl
    );

    for (idx, kdsatker) in satker_list.iter().enumerate() {
        info!(
            "Processing satker {}/{}: {}",
            idx + 1,
            satker_list.len(),
            kdsatker
        );

        match fetch_satker_complete(client, storage, kode_kl, kdsatker).await {
            Ok(_) => info!("✓ Satker {} completed", kdsatker),
            Err(e) => error!("✗ Satker {} failed: {}", kdsatker, e),
        }

        // Rate limiting
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }

    info!("✓ All satker processing completed for KL{}", kode_kl);
    Ok(())
}

/// Fetch semua data (MySIMKARI + MonSAKTI) untuk KL tertentu
pub async fn fetch_all_data(
    client: &mut MonsaktiClient,
    storage: &StorageStrategy,
    kode_kl: &str,
) -> Result<(), MonsaktiError> {
    info!("=== Starting complete data fetch for KL{} ===", kode_kl);

    // Step 1: Global references
    info!("Step 1/3: Fetching global reference data...");
    match fetch_global_references(client, storage, kode_kl).await {
        Ok(_) => info!("✓ Global references completed"),
        Err(e) => error!("✗ Global references failed: {}", e),
    }

    // Step 2: MySIMKARI data
    info!("Step 2/3: Fetching MySIMKARI data...");
    match fetch_mysimkari(client, storage).await {
        Ok(_) => info!("✓ MySIMKARI data completed"),
        Err(e) => error!("✗ MySIMKARI data failed: {}", e),
    }

    // Step 3: MonSAKTI data untuk semua satker
    info!("Step 3/3: Fetching MonSAKTI data for all satker...");
    match fetch_all_satker(client, storage, kode_kl).await {
        Ok(_) => info!("✓ MonSAKTI data completed"),
        Err(e) => error!("✗ MonSAKTI data failed: {}", e),
    }

    info!("=== Complete data fetch finished for KL{} ===", kode_kl);
    Ok(())
}

/// Fetch dengan parallel processing (experimental)
/// WARNING: Dapat menyebabkan rate limiting, gunakan dengan hati-hati
pub async fn fetch_satker_parallel(
    client: &mut MonsaktiClient,
    storage: &StorageStrategy,
    kode_kl: &str,
    kdsatker: &str,
) -> Result<(), MonsaktiError> {
    info!("[{}] Processing satker (parallel mode)...", kdsatker);

    let kode_kl = kode_kl.to_string();
    let kdsatker_str = kdsatker.to_string();
    let storage = storage.clone();

    // Clone client untuk parallel tasks
    let mut client_adm = client.clone();
    let mut client_ang = client.clone();
    let mut client_ben = client.clone();
    let mut client_pem = client.clone();
    let mut client_kom = client.clone();
    let mut client_ast = client.clone();
    let mut client_per = client.clone();
    let mut client_glp = client.clone();

    let storage_adm = storage.clone();
    let storage_ang = storage.clone();
    let storage_ben = storage.clone();
    let storage_pem = storage.clone();
    let storage_kom = storage.clone();
    let storage_ast = storage.clone();
    let storage_per = storage.clone();
    let storage_glp = storage.clone();

    let kl_adm = kode_kl.clone();
    let kl_ang = kode_kl.clone();
    let kl_ben = kode_kl.clone();
    let kl_pem = kode_kl.clone();
    let kl_kom = kode_kl.clone();
    let kl_ast = kode_kl.clone();
    let kl_per = kode_kl.clone();
    let kl_glp = kode_kl.clone();

    let satker_adm = kdsatker_str.clone();
    let satker_ang = kdsatker_str.clone();
    let satker_ben = kdsatker_str.clone();
    let satker_pem = kdsatker_str.clone();
    let satker_kom = kdsatker_str.clone();
    let satker_ast = kdsatker_str.clone();
    let satker_per = kdsatker_str.clone();
    let satker_glp = kdsatker_str.clone();

    let task_adm = tokio::spawn(async move {
        let _ = fetch_adm(&mut client_adm, &storage_adm, &kl_adm, &satker_adm).await;
    });

    let task_ang = tokio::spawn(async move {
        let _ = fetch_ang(&mut client_ang, &storage_ang, &kl_ang, &satker_ang).await;
    });

    let task_ben = tokio::spawn(async move {
        let _ = fetch_ben(&mut client_ben, &storage_ben, &kl_ben, &satker_ben).await;
    });

    let task_pem = tokio::spawn(async move {
        let _ = fetch_pem(&mut client_pem, &storage_pem, &kl_pem, &satker_pem).await;
    });

    let task_kom = tokio::spawn(async move {
        let _ = fetch_kom(&mut client_kom, &storage_kom, &kl_kom, &satker_kom).await;
    });

    let task_ast = tokio::spawn(async move {
        let _ = fetch_ast(&mut client_ast, &storage_ast, &kl_ast, &satker_ast).await;
    });

    let task_per = tokio::spawn(async move {
        let _ = fetch_per(&mut client_per, &storage_per, &kl_per, &satker_per).await;
    });

    let task_glp = tokio::spawn(async move {
        let _ = fetch_glp(&mut client_glp, &storage_glp, &kl_glp, &satker_glp).await;
    });

    // Wait for all tasks
    let _ = tokio::join!(
        task_adm, task_ang, task_ben, task_pem, task_kom, task_ast, task_per, task_glp
    );

    info!("[{}] ✓ Completed (parallel)", kdsatker);
    Ok(())
}
