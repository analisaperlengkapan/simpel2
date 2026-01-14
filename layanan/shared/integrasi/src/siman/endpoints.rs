use crate::client::MonsaktiClient;
use crate::error::MonsaktiError;
use crate::siman::models::SimanAssetCategory;
use futures::stream::{self, StreamExt};
use serde_json::Value;
use tracing::{info, warn};

/// Mendapatkan jumlah baris untuk kategori aset tertentu
///
/// # Arguments
/// * `client` - MonsaktiClient yang sudah dikonfigurasi
/// * `category` - Kategori aset yang akan diquery
///
/// # Returns
/// Total jumlah baris data untuk kategori aset tersebut
///
/// # Example
/// ```no_run
/// use layanan_integrasi::siman::{get_row_count, SimanAssetCategory};
/// use layanan_integrasi::client::MonsaktiClient;
///
/// # async fn example(client: &mut MonsaktiClient) -> Result<(), Box<dyn std::error::Error>> {
/// let count = get_row_count(client, SimanAssetCategory::AlatBesar).await?;
/// println!("Total aset alat besar: {}", count);
/// # Ok(())
/// # }
/// ```
pub async fn get_row_count(
    client: &mut MonsaktiClient,
    category: SimanAssetCategory,
) -> Result<i64, MonsaktiError> {
    let response = client.fetch_siman_row_count(category).await?;

    // Parse response untuk mendapatkan count
    if let Some(data) = response.data {
        if let Some(array) = data.as_array() {
            if let Some(first) = array.first() {
                // Coba extract dari berbagai kemungkinan field name
                if let Some(count) = first.get("row_count").and_then(|v| v.as_i64()) {
                    return Ok(count);
                }
                if let Some(count) = first.get("total").and_then(|v| v.as_i64()) {
                    return Ok(count);
                }
                if let Some(count) = first.get("ROW_COUNT").and_then(|v| v.as_i64()) {
                    return Ok(count);
                }
                if let Some(count) = first.get("TOTAL").and_then(|v| v.as_i64()) {
                    return Ok(count);
                }
            }
        }
    }

    Ok(0)
}

/// Mendapatkan data aset berdasarkan kategori dengan pagination
///
/// # Arguments
/// * `client` - MonsaktiClient yang sudah dikonfigurasi
/// * `category` - Kategori aset yang akan diquery
/// * `start_id` - Index awal data (1-based)
/// * `end_id` - Index akhir data (inklusif)
///
/// # Returns
/// Vector JSON Value yang berisi data aset
///
/// # Example
/// ```no_run
/// use layanan_integrasi::siman::{get_aset_by_category, SimanAssetCategory};
/// use layanan_integrasi::client::MonsaktiClient;
///
/// # async fn example(client: &mut MonsaktiClient) -> Result<(), Box<dyn std::error::Error>> {
/// let data = get_aset_by_category(client, SimanAssetCategory::Tanah, 1, 100).await?;
/// println!("Retrieved {} records", data.len());
/// # Ok(())
/// # }
/// ```
pub async fn get_aset_by_category(
    client: &mut MonsaktiClient,
    category: SimanAssetCategory,
    start_id: u32,
    end_id: u32,
) -> Result<Vec<Value>, MonsaktiError> {
    let response = client.fetch_siman_data(category, start_id, end_id).await?;

    if let Some(data) = response.data {
        if let Some(array) = data.as_array() {
            return Ok(array.clone());
        }
    }

    Ok(vec![])
}

/// Mengambil semua data aset dengan pagination otomatis
///
/// Fungsi ini akan:
/// 1. Mendapatkan total row count
/// 2. Melakukan pagination dengan chunk size 1000
/// 3. Mengumpulkan semua data
///
/// # Arguments
/// * `client` - MonsaktiClient yang sudah dikonfigurasi
/// * `category` - Kategori aset yang akan diquery
/// * `chunk_size` - Ukuran per batch (default: 1000)
///
/// # Returns
/// Vector semua data aset untuk kategori tersebut
pub async fn fetch_all_aset_paginated(
    client: &mut MonsaktiClient,
    category: SimanAssetCategory,
    chunk_size: u32,
) -> Result<Vec<Value>, MonsaktiError> {
    info!("Fetching all data for category: {}", category.description());

    // Get total count first
    let total_count = get_row_count(client, category).await?;
    info!("Total rows for {}: {}", category.description(), total_count);

    if total_count == 0 {
        warn!("No data found for category: {}", category.description());
        return Ok(vec![]);
    }

    // Generate ranges
    let mut ranges = Vec::new();
    let mut start_id = 1u32;
    while start_id <= total_count as u32 {
        let end_id = (start_id + chunk_size - 1).min(total_count as u32);
        ranges.push((start_id, end_id));
        start_id = end_id + 1;
    }

    // Process concurrent requests
    let results = stream::iter(ranges)
        .map(|(start_id, end_id)| {
            let mut client_clone = client.clone();
            let category_clone = category.clone(); // SimanAssetCategory is Clone/Copy

            async move {
                info!(
                    "Fetching {} records {}-{} of {}",
                    category_clone.description(),
                    start_id,
                    end_id,
                    total_count
                );

                get_aset_by_category(&mut client_clone, category_clone, start_id, end_id).await
                    .map_err(|e| {
                         warn!(
                            "Error fetching {}-{} for {}: {}",
                            start_id,
                            end_id,
                            category_clone.description(),
                            e
                        );
                        e
                    })
            }
        })
        .buffered(10) // Concurrent limit
        .collect::<Vec<Result<Vec<Value>, MonsaktiError>>>()
        .await;

    // Collect results
    let mut all_data = Vec::new();
    for result in results {
        match result {
            Ok(data) => all_data.extend(data),
            Err(_) => {
                // Warning already logged in stream map
                // Continue with partial data as per original implementation intent
                // Original implementation: warn and continue
            }
        }
    }

    info!(
        "Successfully fetched {} total records for {}",
        all_data.len(),
        category.description()
    );

    Ok(all_data)
}

// === Convenience functions untuk setiap kategori aset ===

/// Mengambil data Aset Alat Besar
pub async fn get_aset_alat_besar(
    client: &mut MonsaktiClient,
    start_id: u32,
    end_id: u32,
) -> Result<Vec<Value>, MonsaktiError> {
    get_aset_by_category(client, SimanAssetCategory::AlatBesar, start_id, end_id).await
}

/// Mengambil data Aset Angkutan Bermotor
pub async fn get_aset_angkutan_bermotor(
    client: &mut MonsaktiClient,
    start_id: u32,
    end_id: u32,
) -> Result<Vec<Value>, MonsaktiError> {
    get_aset_by_category(
        client,
        SimanAssetCategory::AngkutanBermotor,
        start_id,
        end_id,
    )
    .await
}

/// Mengambil data Aset Alat Persenjataan
pub async fn get_aset_alat_persenjataan(
    client: &mut MonsaktiClient,
    start_id: u32,
    end_id: u32,
) -> Result<Vec<Value>, MonsaktiError> {
    get_aset_by_category(
        client,
        SimanAssetCategory::AlatPersenjataan,
        start_id,
        end_id,
    )
    .await
}

/// Mengambil data Aset Tak Berwujud
pub async fn get_aset_tak_berwujud(
    client: &mut MonsaktiClient,
    start_id: u32,
    end_id: u32,
) -> Result<Vec<Value>, MonsaktiError> {
    get_aset_by_category(client, SimanAssetCategory::TakBerwujud, start_id, end_id).await
}

/// Mengambil data Aset Bangunan Air
pub async fn get_aset_bangunan_air(
    client: &mut MonsaktiClient,
    start_id: u32,
    end_id: u32,
) -> Result<Vec<Value>, MonsaktiError> {
    get_aset_by_category(client, SimanAssetCategory::BangunanAir, start_id, end_id).await
}

/// Mengambil data Aset Gedung dan Bangunan
pub async fn get_aset_gedung_bangunan(
    client: &mut MonsaktiClient,
    start_id: u32,
    end_id: u32,
) -> Result<Vec<Value>, MonsaktiError> {
    get_aset_by_category(client, SimanAssetCategory::GedungBangunan, start_id, end_id).await
}

/// Mengambil data Aset Instalasi dan Jaringan
pub async fn get_aset_instalasi_jaringan(
    client: &mut MonsaktiClient,
    start_id: u32,
    end_id: u32,
) -> Result<Vec<Value>, MonsaktiError> {
    get_aset_by_category(
        client,
        SimanAssetCategory::InstalasiJaringan,
        start_id,
        end_id,
    )
    .await
}

/// Mengambil data Aset Jalan dan Jembatan
pub async fn get_aset_jalan_jembatan(
    client: &mut MonsaktiClient,
    start_id: u32,
    end_id: u32,
) -> Result<Vec<Value>, MonsaktiError> {
    get_aset_by_category(
        client,
        SimanAssetCategory::JalandanJembatan,
        start_id,
        end_id,
    )
    .await
}

/// Mengambil data Aset Non-TIK
pub async fn get_aset_non_tik(
    client: &mut MonsaktiClient,
    start_id: u32,
    end_id: u32,
) -> Result<Vec<Value>, MonsaktiError> {
    get_aset_by_category(client, SimanAssetCategory::NonTIK, start_id, end_id).await
}

/// Mengambil data Aset Rumah
pub async fn get_aset_rumah(
    client: &mut MonsaktiClient,
    start_id: u32,
    end_id: u32,
) -> Result<Vec<Value>, MonsaktiError> {
    get_aset_by_category(client, SimanAssetCategory::Rumah, start_id, end_id).await
}

/// Mengambil data Aset Tanah
pub async fn get_aset_tanah(
    client: &mut MonsaktiClient,
    start_id: u32,
    end_id: u32,
) -> Result<Vec<Value>, MonsaktiError> {
    get_aset_by_category(client, SimanAssetCategory::Tanah, start_id, end_id).await
}

/// Mengambil data Aset Tetap Lainnya
pub async fn get_aset_tetap_lainnya(
    client: &mut MonsaktiClient,
    start_id: u32,
    end_id: u32,
) -> Result<Vec<Value>, MonsaktiError> {
    get_aset_by_category(client, SimanAssetCategory::TetapLainnya, start_id, end_id).await
}

/// Mengambil data Konstruksi Dalam Pengerjaan (KDP)
pub async fn get_aset_kdp(
    client: &mut MonsaktiClient,
    start_id: u32,
    end_id: u32,
) -> Result<Vec<Value>, MonsaktiError> {
    get_aset_by_category(client, SimanAssetCategory::KDP, start_id, end_id).await
}

/// Mengambil data Aset Khusus TIK
pub async fn get_aset_khusus_tik(
    client: &mut MonsaktiClient,
    start_id: u32,
    end_id: u32,
) -> Result<Vec<Value>, MonsaktiError> {
    get_aset_by_category(client, SimanAssetCategory::KhususTIK, start_id, end_id).await
}

/// Mengambil data Aset Tetap Renovasi
pub async fn get_aset_tetap_renovasi(
    client: &mut MonsaktiClient,
    start_id: u32,
    end_id: u32,
) -> Result<Vec<Value>, MonsaktiError> {
    get_aset_by_category(client, SimanAssetCategory::TetapRenovasi, start_id, end_id).await
}

/// Fetch all assets with pagination and save to storage
///
/// This function fetches all assets for a given category and saves them using the storage strategy.
/// Returns (success_count, failed_count)
pub async fn fetch_all_assets_with_pagination(
    client: &mut MonsaktiClient,
    storage: &crate::StorageStrategy,
    category: SimanAssetCategory,
) -> Result<(usize, usize), MonsaktiError> {
    info!("📥 Starting fetch for: {}", category.description());

    // Get row count
    let response = client.fetch_siman_row_count(category.clone()).await?;

    let total_count = if let Some(data) = response.data {
        if let Some(results) = data.get("results").and_then(|r| r.as_array()) {
            if let Some(first) = results.first() {
                first.get("RCOUNT").and_then(|v| v.as_i64()).unwrap_or(0)
            } else {
                0
            }
        } else {
            0
        }
    } else {
        0
    };

    info!(
        "📊 Total records for {}: {}",
        category.description(),
        total_count
    );

    if total_count == 0 {
        warn!("⚠️  No data available for {}", category.description());
        return Ok((0, 0));
    }

    let mut success_count = 0usize;
    let mut failed_count = 0usize;
    let chunk_size = 1000u32; // Batch size per request - increased for faster fetching
    let mut current_id = 1u32;

    while current_id <= total_count as u32 {
        let end_id = (current_id + chunk_size - 1).min(total_count as u32);

        info!(
            "🔄 Fetching records {}-{} of {}",
            current_id, end_id, total_count
        );

        match client
            .fetch_siman_data(category.clone(), current_id, end_id)
            .await
        {
            Ok(response) => {
                if let Some(data) = response.data {
                    // Extract results array
                    let records =
                        if let Some(results) = data.get("results").and_then(|r| r.as_array()) {
                            results.clone()
                        } else if let Some(results_array) = data.as_array() {
                            results_array.clone()
                        } else {
                            vec![]
                        };

                    if records.is_empty() {
                        warn!("⚠️  No records in response for {}-{}", current_id, end_id);
                        failed_count += (end_id - current_id + 1) as usize;
                    } else {
                        // Inject required fields for SIMAN database schema
                        let category_name = category.description().to_string(); // "Alat Besar", etc
                        let enhanced_records: Vec<serde_json::Value> = records
                            .iter()
                            .map(|record| {
                                if let Some(mut obj) = record.as_object().cloned() {
                                    // Add kategori_aset field (REQUIRED by DB)
                                    obj.insert(
                                        "kategori_aset".to_string(),
                                        serde_json::Value::String(category_name.clone()),
                                    );
                                    // Add raw_data field (store complete API response)
                                    obj.insert("raw_data".to_string(), record.clone());
                                    serde_json::Value::Object(obj)
                                } else {
                                    record.clone()
                                }
                            })
                            .collect();

                        let json_data = serde_json::Value::Array(enhanced_records);

                        // Use storage strategy to save
                        // module = "siman", endpoint = "aset" (unified table), context = batch range
                        let context = format!("{}-{}", current_id, end_id);

                        match storage
                            .save(client, "siman", "aset", &json_data, &context)
                            .await
                        {
                            Ok(saved_count) => {
                                success_count += saved_count;
                                if saved_count > 0 {
                                    info!(
                                        "💾 Saved {} records ({}-{})",
                                        saved_count, current_id, end_id
                                    );
                                } else {
                                    warn!(
                                        "⚠️  0 records inserted ({}-{}) - check for errors above",
                                        current_id, end_id
                                    );
                                    failed_count += records.len();
                                }
                            }
                            Err(e) => {
                                tracing::error!(
                                    "❌ Storage save failed for {}-{}: {}",
                                    current_id,
                                    end_id,
                                    e
                                );
                                failed_count += records.len();
                            }
                        }
                    }
                }
            }
            Err(e) => {
                tracing::error!("❌ API fetch failed for {}-{}: {}", current_id, end_id, e);
                failed_count += (end_id - current_id + 1) as usize;
            }
        }

        current_id = end_id + 1;

        // Rate limiting - small delay between requests
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    Ok((success_count, failed_count))
}
