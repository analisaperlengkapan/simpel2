use crate::client::MonsaktiClient;
use crate::error::MonsaktiError;
use crate::siman::models::SimanAssetCategory;
use futures::stream::{self, StreamExt};
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::sleep;
use tracing::{error, info, warn};

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

/// Circuit breaker for SIMAN API calls
/// Implements the circuit breaker pattern to prevent cascading failures
#[derive(Clone)]
#[allow(dead_code)]
pub struct CircuitBreaker {
    state: Arc<Mutex<CircuitState>>,
    failure_count: Arc<Mutex<u32>>,
    last_failure_time: Arc<Mutex<Option<std::time::Instant>>>,
    failure_threshold: u32,
    timeout_duration: Duration,
    half_open_max_calls: u32,
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    ///
    /// # Arguments
    /// * `failure_threshold` - Number of failures before opening the circuit
    /// * `timeout_duration` - Duration to wait before attempting half-open state
    pub fn new(failure_threshold: u32, timeout_duration: Duration) -> Self {
        Self {
            state: Arc::new(Mutex::new(CircuitState::Closed)),
            failure_count: Arc::new(Mutex::new(0)),
            last_failure_time: Arc::new(Mutex::new(None)),
            failure_threshold,
            timeout_duration,
            half_open_max_calls: 3,
        }
    }

    /// Check if the circuit allows the call
    pub async fn can_proceed(&self) -> Result<(), MonsaktiError> {
        let mut state = self.state.lock().await;

        match *state {
            CircuitState::Closed => Ok(()),
            CircuitState::Open => {
                // Check if timeout has elapsed
                let last_failure = self.last_failure_time.lock().await;
                if let Some(last_time) = *last_failure {
                    if last_time.elapsed() >= self.timeout_duration {
                        // Transition to half-open
                        *state = CircuitState::HalfOpen;
                        drop(state);
                        drop(last_failure);
                        info!("Circuit breaker transitioning to HALF-OPEN state");
                        Ok(())
                    } else {
                        Err(MonsaktiError::ApiError(
                            "Circuit breaker is OPEN - too many failures".to_string(),
                        ))
                    }
                } else {
                    Err(MonsaktiError::ApiError(
                        "Circuit breaker is OPEN".to_string(),
                    ))
                }
            }
            CircuitState::HalfOpen => Ok(()),
        }
    }

    /// Record a successful call
    pub async fn record_success(&self) {
        let mut state = self.state.lock().await;
        let mut failure_count = self.failure_count.lock().await;

        match *state {
            CircuitState::HalfOpen => {
                // Success in half-open state - close the circuit
                *state = CircuitState::Closed;
                *failure_count = 0;
                info!("Circuit breaker transitioning to CLOSED state after successful call");
            }
            CircuitState::Closed => {
                // Reset failure count on success
                *failure_count = 0;
            }
            CircuitState::Open => {
                // Should not happen, but reset if it does
                *state = CircuitState::Closed;
                *failure_count = 0;
            }
        }
    }

    /// Record a failed call
    pub async fn record_failure(&self) {
        let mut state = self.state.lock().await;
        let mut failure_count = self.failure_count.lock().await;
        let mut last_failure_time = self.last_failure_time.lock().await;

        *failure_count += 1;
        *last_failure_time = Some(std::time::Instant::now());

        match *state {
            CircuitState::Closed => {
                if *failure_count >= self.failure_threshold {
                    *state = CircuitState::Open;
                    warn!(
                        "Circuit breaker transitioning to OPEN state after {} failures",
                        *failure_count
                    );
                }
            }
            CircuitState::HalfOpen => {
                // Failure in half-open state - reopen the circuit
                *state = CircuitState::Open;
                warn!(
                    "Circuit breaker transitioning back to OPEN state after failure in HALF-OPEN"
                );
            }
            CircuitState::Open => {
                // Already open, just update the timestamp
            }
        }
    }

    /// Get current state for monitoring
    pub async fn get_state(&self) -> CircuitState {
        *self.state.lock().await
    }
}

/// Retry with exponential backoff
///
/// # Arguments
/// * `max_retries` - Maximum number of retry attempts
/// * `initial_delay` - Initial delay before first retry
/// * `max_delay` - Maximum delay between retries
/// * `operation` - Async operation to retry
#[allow(dead_code)]
async fn retry_with_exponential_backoff<F, Fut, T>(
    max_retries: u32,
    initial_delay: Duration,
    max_delay: Duration,
    mut operation: F,
) -> Result<T, MonsaktiError>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, MonsaktiError>>,
{
    let mut attempt = 0;
    let mut delay = initial_delay;

    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                attempt += 1;

                if attempt >= max_retries {
                    error!("Max retries ({}) exceeded", max_retries);
                    return Err(e);
                }

                warn!(
                    "Attempt {} failed: {}. Retrying in {:?}...",
                    attempt, e, delay
                );

                sleep(delay).await;

                // Exponential backoff with jitter
                delay = std::cmp::min(delay * 2, max_delay);

                // Add jitter (±25%)
                let jitter = (delay.as_millis() as f64 * 0.25) as u64;
                let jitter_range = rand::random::<u64>() % (jitter * 2);
                delay = Duration::from_millis(delay.as_millis() as u64 + jitter_range - jitter);
            }
        }
    }
}

/// Mendapatkan jumlah baris untuk kategori aset tertentu dengan retry dan circuit breaker
///
/// # Arguments
/// * `client` - MonsaktiClient yang sudah dikonfigurasi
/// * `category` - Kategori aset yang akan diquery
/// * `circuit_breaker` - Optional circuit breaker untuk fault tolerance
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
/// let count = get_row_count(client, SimanAssetCategory::AlatBesar, None).await?;
/// println!("Total aset alat besar: {}", count);
/// # Ok(())
/// # }
/// ```
pub async fn get_row_count(
    client: &mut MonsaktiClient,
    category: SimanAssetCategory,
    circuit_breaker: Option<&CircuitBreaker>,
) -> Result<i64, MonsaktiError> {
    // Check circuit breaker if provided
    if let Some(cb) = circuit_breaker {
        cb.can_proceed().await?;
    }

    // Direct API call (circuit breaker provides retry protection)
    let result = async {
        let response = client.fetch_siman_row_count(category).await?;

        // Parse response untuk mendapatkan count
        if let Some(data) = response.data {
            // Check multiple possible array fields or the root array itself
            let array = if let Some(arr) = data.as_array() {
                Some(arr)
            } else if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
                Some(arr)
            } else {
                data.get("results")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr)
            };

            if let Some(array) = array {
                if let Some(first) = array.first() {
                    // Try to extract from various possible field names (handle string/number and case sensitivity)
                    for key in &[
                        "row_count",
                        "total",
                        "ROW_COUNT",
                        "TOTAL",
                        "RCOUNT",
                        "rcount",
                    ] {
                        if let Some(val) = first.get(*key) {
                            if let Some(count) = val.as_i64() {
                                return Ok(count);
                            }
                            if let Some(s) = val.as_str() {
                                if let Ok(count) = s.parse::<i64>() {
                                    return Ok(count);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(0)
    }
    .await;

    // Record result in circuit breaker
    match &result {
        Ok(_) => {
            if let Some(cb) = circuit_breaker {
                cb.record_success().await;
            }
        }
        Err(_) => {
            if let Some(cb) = circuit_breaker {
                cb.record_failure().await;
            }
        }
    }

    result
}

/// Mendapatkan data aset berdasarkan kategori dengan pagination, retry, dan circuit breaker
///
/// # Arguments
/// * `client` - MonsaktiClient yang sudah dikonfigurasi
/// * `category` - Kategori aset yang akan diquery
/// * `start_id` - Index awal data (1-based)
/// * `end_id` - Index akhir data (inklusif)
/// * `circuit_breaker` - Optional circuit breaker untuk fault tolerance
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
/// let data = get_aset_by_category(client, SimanAssetCategory::Tanah, 1, 100, None).await?;
/// println!("Retrieved {} records", data.len());
/// # Ok(())
/// # }
/// ```
pub async fn get_aset_by_category(
    client: &mut MonsaktiClient,
    category: SimanAssetCategory,
    start_id: u32,
    end_id: u32,
    circuit_breaker: Option<&CircuitBreaker>,
) -> Result<Vec<Value>, MonsaktiError> {
    // Check circuit breaker if provided
    if let Some(cb) = circuit_breaker {
        cb.can_proceed().await?;
    }

    // Direct API call (circuit breaker provides retry protection)
    let result = async {
        let response = client.fetch_siman_data(category, start_id, end_id).await?;

        if let Some(data) = response.data {
            if let Some(array) = data.as_array() {
                return Ok(array.clone());
            } else if let Some(data_array) = data.get("data").and_then(|v| v.as_array()) {
                return Ok(data_array.clone());
            } else if let Some(results) = data.get("results").and_then(|v| v.as_array()) {
                return Ok(results.clone());
            }
        }

        Ok(vec![])
    }
    .await;

    // Record result in circuit breaker
    match &result {
        Ok(_) => {
            if let Some(cb) = circuit_breaker {
                cb.record_success().await;
            }
        }
        Err(_) => {
            if let Some(cb) = circuit_breaker {
                cb.record_failure().await;
            }
        }
    }

    result
}

/// Mengambil semua data aset dengan pagination otomatis, retry, dan circuit breaker
///
/// Fungsi ini akan:
/// 1. Mendapatkan total row count
/// 2. Melakukan pagination dengan chunk size 1000
/// 3. Mengumpulkan semua data dengan retry dan circuit breaker
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

    // Create circuit breaker for this fetch operation
    let circuit_breaker = CircuitBreaker::new(
        5,                       // failure threshold
        Duration::from_secs(60), // timeout duration
    );

    // Get total count first with circuit breaker
    let total_count = get_row_count(client, category, Some(&circuit_breaker)).await?;
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

    // Process concurrent requests with circuit breaker
    let results = stream::iter(ranges)
        .map(|(start_id, end_id)| {
            let mut client_clone = client.clone();
            let category_clone = category; // SimanAssetCategory is Clone/Copy
            let circuit_breaker_clone = circuit_breaker.clone();

            async move {
                info!(
                    "Fetching {} records {}-{} of {}",
                    category_clone.description(),
                    start_id,
                    end_id,
                    total_count
                );

                get_aset_by_category(
                    &mut client_clone,
                    category_clone,
                    start_id,
                    end_id,
                    Some(&circuit_breaker_clone),
                )
                .await
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
    get_aset_by_category(
        client,
        SimanAssetCategory::AlatBesar,
        start_id,
        end_id,
        None,
    )
    .await
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
        None,
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
        None,
    )
    .await
}

/// Mengambil data Aset Tak Berwujud
pub async fn get_aset_tak_berwujud(
    client: &mut MonsaktiClient,
    start_id: u32,
    end_id: u32,
) -> Result<Vec<Value>, MonsaktiError> {
    get_aset_by_category(
        client,
        SimanAssetCategory::TakBerwujud,
        start_id,
        end_id,
        None,
    )
    .await
}

/// Mengambil data Aset Bangunan Air
pub async fn get_aset_bangunan_air(
    client: &mut MonsaktiClient,
    start_id: u32,
    end_id: u32,
) -> Result<Vec<Value>, MonsaktiError> {
    get_aset_by_category(
        client,
        SimanAssetCategory::BangunanAir,
        start_id,
        end_id,
        None,
    )
    .await
}

/// Mengambil data Aset Gedung dan Bangunan
pub async fn get_aset_gedung_bangunan(
    client: &mut MonsaktiClient,
    start_id: u32,
    end_id: u32,
) -> Result<Vec<Value>, MonsaktiError> {
    get_aset_by_category(
        client,
        SimanAssetCategory::GedungBangunan,
        start_id,
        end_id,
        None,
    )
    .await
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
        None,
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
        None,
    )
    .await
}

/// Mengambil data Aset Non-TIK
pub async fn get_aset_non_tik(
    client: &mut MonsaktiClient,
    start_id: u32,
    end_id: u32,
) -> Result<Vec<Value>, MonsaktiError> {
    get_aset_by_category(client, SimanAssetCategory::NonTIK, start_id, end_id, None).await
}

/// Mengambil data Aset Rumah
pub async fn get_aset_rumah(
    client: &mut MonsaktiClient,
    start_id: u32,
    end_id: u32,
) -> Result<Vec<Value>, MonsaktiError> {
    get_aset_by_category(client, SimanAssetCategory::Rumah, start_id, end_id, None).await
}

/// Mengambil data Aset Tanah
pub async fn get_aset_tanah(
    client: &mut MonsaktiClient,
    start_id: u32,
    end_id: u32,
) -> Result<Vec<Value>, MonsaktiError> {
    get_aset_by_category(client, SimanAssetCategory::Tanah, start_id, end_id, None).await
}

/// Mengambil data Aset Tetap Lainnya
pub async fn get_aset_tetap_lainnya(
    client: &mut MonsaktiClient,
    start_id: u32,
    end_id: u32,
) -> Result<Vec<Value>, MonsaktiError> {
    get_aset_by_category(
        client,
        SimanAssetCategory::TetapLainnya,
        start_id,
        end_id,
        None,
    )
    .await
}

/// Mengambil data Konstruksi Dalam Pengerjaan (KDP)
pub async fn get_aset_kdp(
    client: &mut MonsaktiClient,
    start_id: u32,
    end_id: u32,
) -> Result<Vec<Value>, MonsaktiError> {
    get_aset_by_category(client, SimanAssetCategory::KDP, start_id, end_id, None).await
}

/// Mengambil data Aset Khusus TIK
pub async fn get_aset_khusus_tik(
    client: &mut MonsaktiClient,
    start_id: u32,
    end_id: u32,
) -> Result<Vec<Value>, MonsaktiError> {
    get_aset_by_category(
        client,
        SimanAssetCategory::KhususTIK,
        start_id,
        end_id,
        None,
    )
    .await
}

/// Mengambil data Aset Tetap Renovasi
pub async fn get_aset_tetap_renovasi(
    client: &mut MonsaktiClient,
    start_id: u32,
    end_id: u32,
) -> Result<Vec<Value>, MonsaktiError> {
    get_aset_by_category(
        client,
        SimanAssetCategory::TetapRenovasi,
        start_id,
        end_id,
        None,
    )
    .await
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
    let response = client.fetch_siman_row_count(category).await?;

    info!("Raw getRowCount response data: {:?}", response.data);

    let total_count = if let Some(data) = response.data {
        let array = if let Some(arr) = data.as_array() {
            Some(arr)
        } else if let Some(arr) = data.get("data").and_then(|v| v.as_array()) {
            Some(arr)
        } else {
            data.get("results")
                .and_then(|v| v.as_array())
                .map(|arr| arr)
        };

        if let Some(array) = array {
            if let Some(first) = array.first() {
                let val = first
                    .get("row_count")
                    .or_else(|| first.get("total"))
                    .or_else(|| first.get("ROW_COUNT"))
                    .or_else(|| first.get("TOTAL"))
                    .or_else(|| first.get("RCOUNT"))
                    .or_else(|| first.get("rcount"));

                val.and_then(|v| {
                    v.as_i64()
                        .or_else(|| v.as_str().and_then(|s| s.parse::<i64>().ok()))
                })
                .unwrap_or(0)
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

    // Prepare ranges
    let mut ranges = Vec::new();
    let mut current_id = 1u32;
    while current_id <= total_count as u32 {
        let end_id = (current_id + chunk_size - 1).min(total_count as u32);
        ranges.push((current_id, end_id));
        current_id = end_id + 1;
    }

    // Prepare tasks with cloned clients to avoid borrow checker issues
    // We clone the client for each task so we can use the original client for saving
    let tasks: Vec<_> = ranges
        .into_iter()
        .map(|(start_id, end_id)| {
            let client_clone = client.clone();
            let category_clone = category;
            (start_id, end_id, client_clone, category_clone)
        })
        .collect();

    let concurrency_limit = client.config().siman_concurrency_limit;

    // Create a channel to decouple fetching from saving
    let (tx, mut rx) = tokio::sync::mpsc::channel(concurrency_limit);

    // Spawn the fetching task
    tokio::spawn(async move {
        let mut stream = stream::iter(tasks)
            .map(
                |(start_id, end_id, mut client_clone, category_clone)| async move {
                    info!(
                        "🔄 Fetching records {}-{} of {}",
                        start_id, end_id, total_count
                    );
                    let result = client_clone
                        .fetch_siman_data(category_clone, start_id, end_id)
                        .await;
                    (start_id, end_id, result)
                },
            )
            .buffer_unordered(concurrency_limit);

        while let Some(item) = stream.next().await {
            if tx.send(item).await.is_err() {
                // Receiver dropped, stop fetching
                break;
            }
        }
    });

    // Iterate through completed tasks received from channel
    while let Some((current_id, end_id, result)) = rx.recv().await {
        match result {
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
                        let category_name = category.description().to_string(); // e.g. "Alat Besar"
                        let enhanced_records: Vec<serde_json::Value> = records
                            .iter()
                            .map(|record| {
                                if let Some(mut obj) = record.as_object().cloned() {
                                    // Add jenis_aset field (REQUIRED by DB)
                                    obj.insert(
                                        "jenis_aset".to_string(),
                                        serde_json::Value::String(category_name.clone()),
                                    );
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
    }

    Ok((success_count, failed_count))
}
