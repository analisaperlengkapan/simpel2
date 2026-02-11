use crate::client::MonsaktiClient;
use crate::error::MonsaktiError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{error, info, warn};

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

/// Circuit breaker for MySIMKARI API
#[derive(Debug, Clone)]
pub struct MySIMKARICircuitBreaker {
    state: Arc<Mutex<CircuitState>>,
    failure_count: Arc<Mutex<u32>>,
    last_failure_time: Arc<Mutex<Option<Instant>>>,
    failure_threshold: u32,
    timeout_duration: Duration,
    half_open_timeout: Duration,
}

impl MySIMKARICircuitBreaker {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(CircuitState::Closed)),
            failure_count: Arc::new(Mutex::new(0)),
            last_failure_time: Arc::new(Mutex::new(None)),
            failure_threshold: 5,
            timeout_duration: Duration::from_secs(60),
            half_open_timeout: Duration::from_secs(30),
        }
    }

    pub async fn call<F, Fut, T>(&self, f: F) -> Result<T, MonsaktiError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, MonsaktiError>>,
    {
        let state = *self.state.lock().await;

        match state {
            CircuitState::Open => {
                let last_failure = self.last_failure_time.lock().await;
                if let Some(last_time) = *last_failure {
                    if last_time.elapsed() > self.timeout_duration {
                        drop(last_failure);
                        *self.state.lock().await = CircuitState::HalfOpen;
                        info!("Circuit breaker transitioning to HalfOpen state");
                    } else {
                        return Err(MonsaktiError::ApiError(
                            "Circuit breaker is OPEN - MySIMKARI API unavailable".to_string(),
                        ));
                    }
                }
            }
            CircuitState::HalfOpen => {
                info!("Circuit breaker in HalfOpen state - testing MySIMKARI API");
            }
            CircuitState::Closed => {}
        }

        match f().await {
            Ok(result) => {
                self.on_success().await;
                Ok(result)
            }
            Err(e) => {
                self.on_failure().await;
                Err(e)
            }
        }
    }

    pub async fn on_success(&self) {
        let mut state = self.state.lock().await;
        if *state == CircuitState::HalfOpen {
            info!("Circuit breaker transitioning to Closed state after successful call");
            *state = CircuitState::Closed;
        }
        *self.failure_count.lock().await = 0;
    }

    pub async fn on_failure(&self) {
        let mut failure_count = self.failure_count.lock().await;
        *failure_count += 1;

        if *failure_count >= self.failure_threshold {
            let mut state = self.state.lock().await;
            if *state != CircuitState::Open {
                warn!(
                    "Circuit breaker opening after {} failures",
                    self.failure_threshold
                );
                *state = CircuitState::Open;
                *self.last_failure_time.lock().await = Some(Instant::now());
            }
        }
    }

    pub async fn get_state(&self) -> CircuitState {
        *self.state.lock().await
    }
}

impl Default for MySIMKARICircuitBreaker {
    fn default() -> Self {
        Self::new()
    }
}

/// MySIMKARI Satker data model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MySIMKARISatker {
    pub id: String,
    pub kode_satker: String,
    pub nama_satker: String,
    pub wilayah: Option<String>,
    pub tipe_satker: Option<String>,
}

/// MySIMKARI Pegawai data model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MySIMKARIPegawai {
    pub nip: String,
    pub nama: String,
    pub satker_id: String,
    pub satker_code: Option<String>,
    pub jabatan: Option<String>,
    pub golongan: Option<String>,
    pub status_pegawai: Option<String>,
}

/// Retry with exponential backoff
async fn retry_with_backoff<F, Fut, T>(
    mut f: F,
    max_retries: u32,
    initial_delay: Duration,
) -> Result<T, MonsaktiError>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, MonsaktiError>>,
{
    let mut retries = 0;
    let mut delay = initial_delay;

    loop {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                if retries >= max_retries {
                    error!(
                        "Max retries ({}) reached for MySIMKARI API call",
                        max_retries
                    );
                    return Err(e);
                }

                retries += 1;
                warn!(
                    "MySIMKARI API call failed (attempt {}/{}): {:?}. Retrying in {:?}",
                    retries, max_retries, e, delay
                );

                tokio::time::sleep(delay).await;

                // Exponential backoff with jitter
                delay = delay.mul_f32(2.0);
                let jitter = Duration::from_millis(rand::random::<u64>() % 1000);
                delay += jitter;

                // Cap at 30 seconds
                if delay > Duration::from_secs(30) {
                    delay = Duration::from_secs(30);
                }
            }
        }
    }
}

/// Endpoint: <https://mysimkari.kejaksaan.go.id/api/anbut/get-satker>
/// Mengambil data satker dari MySIMKARI dengan circuit breaker
pub async fn get_satker(
    client: &mut MonsaktiClient,
    circuit_breaker: &MySIMKARICircuitBreaker,
) -> Result<serde_json::Value, MonsaktiError> {
    // Check circuit breaker state
    let state = circuit_breaker.get_state().await;
    if state == CircuitState::Open {
        return Err(MonsaktiError::ApiError(
            "Circuit breaker is OPEN - MySIMKARI API unavailable".to_string(),
        ));
    }

    // Attempt the API call
    let result = async {
        let response = client.fetch_mysimkari("get-satker", vec![]).await?;
        response
            .data
            .ok_or_else(|| MonsaktiError::ApiError("No data".to_string()))
    }
    .await;

    // Update circuit breaker based on result
    match &result {
        Ok(_) => circuit_breaker.on_success().await,
        Err(_) => circuit_breaker.on_failure().await,
    }

    result
}

/// Endpoint: <https://mysimkari.kejaksaan.go.id/api/anbut/pegawai-satker/{id}>
/// Mengambil data pegawai untuk satker tertentu dengan retry dan circuit breaker
pub async fn pegawai_satker(
    client: &mut MonsaktiClient,
    satker_id: &str,
    circuit_breaker: &MySIMKARICircuitBreaker,
) -> Result<serde_json::Value, MonsaktiError> {
    // Check circuit breaker state
    let state = circuit_breaker.get_state().await;
    if state == CircuitState::Open {
        return Err(MonsaktiError::ApiError(
            "Circuit breaker is OPEN - MySIMKARI API unavailable".to_string(),
        ));
    }

    // Attempt the API call
    let result = async {
        let response = client
            .fetch_mysimkari("pegawai-satker", vec![satker_id.to_string()])
            .await?;
        response
            .data
            .ok_or_else(|| MonsaktiError::ApiError("No data".to_string()))
    }
    .await;

    // Update circuit breaker based on result
    match &result {
        Ok(_) => circuit_breaker.on_success().await,
        Err(_) => circuit_breaker.on_failure().await,
    }

    result
}

/// Endpoint: <https://mysimkari.kejaksaan.go.id/api/anbut/pegawai/{nip}>
/// Mengambil data pegawai berdasarkan NIP dengan retry dan circuit breaker
pub async fn get_pegawai_by_nip(
    client: &mut MonsaktiClient,
    nip: &str,
    circuit_breaker: &MySIMKARICircuitBreaker,
) -> Result<serde_json::Value, MonsaktiError> {
    // Check circuit breaker state
    let state = circuit_breaker.get_state().await;
    if state == CircuitState::Open {
        return Err(MonsaktiError::ApiError(
            "Circuit breaker is OPEN - MySIMKARI API unavailable".to_string(),
        ));
    }

    // Attempt the API call
    let result = async {
        let response = client
            .fetch_mysimkari("pegawai", vec![nip.to_string()])
            .await?;
        response
            .data
            .ok_or_else(|| MonsaktiError::ApiError("No data".to_string()))
    }
    .await;

    // Update circuit breaker based on result
    match &result {
        Ok(_) => circuit_breaker.on_success().await,
        Err(_) => circuit_breaker.on_failure().await,
    }

    result
}

/// Endpoint: <https://mysimkari.kejaksaan.go.id/api/anbut/pegawai-aktif>
/// Mengambil data semua pegawai aktif dengan circuit breaker
pub async fn get_pegawai_aktif(
    client: &mut MonsaktiClient,
    circuit_breaker: &MySIMKARICircuitBreaker,
) -> Result<serde_json::Value, MonsaktiError> {
    // Check circuit breaker state
    let state = circuit_breaker.get_state().await;
    if state == CircuitState::Open {
        return Err(MonsaktiError::ApiError(
            "Circuit breaker is OPEN - MySIMKARI API unavailable".to_string(),
        ));
    }

    // Attempt the API call
    let result = async {
        let response = client.fetch_mysimkari("pegawai-aktif", vec![]).await?;
        response
            .data
            .ok_or_else(|| MonsaktiError::ApiError("No data".to_string()))
    }
    .await;

    // Update circuit breaker based on result
    match &result {
        Ok(_) => circuit_breaker.on_success().await,
        Err(_) => circuit_breaker.on_failure().await,
    }

    result
}

/// Endpoint: <https://mysimkari.kejaksaan.go.id/api/anbut/pegawai-mutasi/{start_date}/{end_date}>
/// Mengambil data pegawai yang mengalami mutasi dalam periode tertentu
pub async fn get_pegawai_mutasi(
    client: &mut MonsaktiClient,
    start_date: &str,
    end_date: &str,
    circuit_breaker: &MySIMKARICircuitBreaker,
) -> Result<serde_json::Value, MonsaktiError> {
    // Check circuit breaker state
    let state = circuit_breaker.get_state().await;
    if state == CircuitState::Open {
        return Err(MonsaktiError::ApiError(
            "Circuit breaker is OPEN - MySIMKARI API unavailable".to_string(),
        ));
    }

    // Attempt the API call
    let result = async {
        let response = client
            .fetch_mysimkari(
                "pegawai-mutasi",
                vec![start_date.to_string(), end_date.to_string()],
            )
            .await?;
        response
            .data
            .ok_or_else(|| MonsaktiError::ApiError("No data".to_string()))
    }
    .await;

    // Update circuit breaker based on result
    match &result {
        Ok(_) => circuit_breaker.on_success().await,
        Err(_) => circuit_breaker.on_failure().await,
    }

    result
}
