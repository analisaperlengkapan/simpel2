//! gRPC client for Layanan Integrasi service
//!
//! This module provides a gRPC client that connects to the layanan-integrasi
//! service to fetch MySIMKARI data (pegawai and satker).

use std::time::Duration;
use tonic::transport::Channel;
use tonic::Request;
use tracing::{debug, error, info, warn};

// Generated proto code for integrasi service
pub mod integrasi_proto {
    pub mod v1 {
        #![allow(missing_docs)]
        #![allow(unused)]
        tonic::include_proto!("integrasi.v1");
    }
}

use integrasi_proto::v1::{
    integrasi_service_client::IntegrasiServiceClient,
    GetMysimkariPegawaiRequest, GetMysimkariSatkerRequest,
    HealthCheckRequest, MysimkariPegawai, MysimkariSatker,
    Pagination,
};

/// Configuration for the Integrasi gRPC client
#[derive(Debug, Clone)]
pub struct IntegrasiClientConfig {
    /// gRPC endpoint URL (e.g., "http://localhost:50052")
    pub grpc_url: String,
    /// Connection timeout
    pub connection_timeout: Duration,
    /// Request timeout per call
    pub request_timeout: Duration,
    /// Max retry attempts
    pub max_retries: u32,
}

impl Default for IntegrasiClientConfig {
    fn default() -> Self {
        Self {
            grpc_url: "http://localhost:50052".to_string(),
            connection_timeout: Duration::from_secs(10),
            request_timeout: Duration::from_secs(30),
            max_retries: 3,
        }
    }
}

/// gRPC client for the Integrasi service (MySIMKARI data access)
#[derive(Clone)]
pub struct IntegrasiClient {
    client: IntegrasiServiceClient<Channel>,
    config: IntegrasiClientConfig,
}

impl IntegrasiClient {
    /// Create a new IntegrasiClient and connect to the gRPC server
    pub async fn connect(config: IntegrasiClientConfig) -> Result<Self, IntegrasiClientError> {
        info!(
            "Connecting to Integrasi gRPC service at {}",
            config.grpc_url
        );

        let channel = Channel::from_shared(config.grpc_url.clone())
            .map_err(|e| IntegrasiClientError::Connection(format!("Invalid endpoint: {}", e)))?
            .connect_timeout(config.connection_timeout)
            .timeout(config.request_timeout)
            .connect()
            .await
            .map_err(|e| {
                IntegrasiClientError::Connection(format!("Failed to connect: {}", e))
            })?;

        let client = IntegrasiServiceClient::new(channel);

        info!(
            "Successfully connected to Integrasi gRPC service at {}",
            config.grpc_url
        );

        Ok(Self { client, config })
    }

    /// Check if the Integrasi service is healthy
    pub async fn health_check(&self) -> Result<bool, IntegrasiClientError> {
        let mut client = self.client.clone();
        let request = Request::new(HealthCheckRequest {});

        match client.health_check(request).await {
            Ok(response) => {
                let resp = response.into_inner();
                debug!("Integrasi health check: healthy={}", resp.healthy);
                Ok(resp.healthy)
            }
            Err(status) => {
                warn!("Integrasi health check failed: {}", status);
                Err(IntegrasiClientError::from_status(status))
            }
        }
    }

    /// Fetch MySIMKARI satker (work units) data
    ///
    /// If `kode_satker` is provided, filters to that specific satker.
    /// Otherwise returns all satker.
    pub async fn get_mysimkari_satker(
        &self,
        kode_satker: Option<&str>,
    ) -> Result<Vec<MysimkariSatker>, IntegrasiClientError> {
        let mut all_items = Vec::new();
        let mut page = 1;
        let per_page = 100;

        loop {
            let result = self
                .get_mysimkari_satker_page(kode_satker, page, per_page)
                .await?;

            let items_count = result.len();
            all_items.extend(result);

            if items_count < per_page as usize {
                break; // Last page
            }
            page += 1;
        }

        info!("Fetched {} MySIMKARI satker records", all_items.len());
        Ok(all_items)
    }

    /// Fetch a single page of MySIMKARI satker data
    async fn get_mysimkari_satker_page(
        &self,
        kode_satker: Option<&str>,
        page: i32,
        per_page: i32,
    ) -> Result<Vec<MysimkariSatker>, IntegrasiClientError> {
        self.with_retry(|| async {
            let mut client = self.client.clone();
            let request = Request::new(GetMysimkariSatkerRequest {
                kode_satker: kode_satker.unwrap_or_default().to_string(),
                pagination: Some(Pagination {
                    page,
                    per_page,
                    sort_by: String::new(),
                    ascending: true,
                }),
            });

            match client.get_mysimkari_satker(request).await {
                Ok(response) => {
                    let resp = response.into_inner();
                    Ok(resp.items)
                }
                Err(status) => Err(IntegrasiClientError::from_status(status)),
            }
        })
        .await
    }

    /// Fetch MySIMKARI pegawai (employees) for a specific satker
    pub async fn get_mysimkari_pegawai(
        &self,
        kode_satker: &str,
    ) -> Result<Vec<MysimkariPegawai>, IntegrasiClientError> {
        let mut all_items = Vec::new();
        let mut page = 1;
        let per_page = 100;

        loop {
            let result = self
                .get_mysimkari_pegawai_page(kode_satker, page, per_page)
                .await?;

            let items_count = result.len();
            all_items.extend(result);

            if items_count < per_page as usize {
                break; // Last page
            }
            page += 1;
        }

        debug!(
            "Fetched {} MySIMKARI pegawai for satker {}",
            all_items.len(),
            kode_satker
        );
        Ok(all_items)
    }

    /// Fetch a single page of MySIMKARI pegawai data
    async fn get_mysimkari_pegawai_page(
        &self,
        kode_satker: &str,
        page: i32,
        per_page: i32,
    ) -> Result<Vec<MysimkariPegawai>, IntegrasiClientError> {
        self.with_retry(|| async {
            let mut client = self.client.clone();
            let request = Request::new(GetMysimkariPegawaiRequest {
                kode_satker: kode_satker.to_string(),
                nama_filter: String::new(),
                nip_filter: String::new(),
                pagination: Some(Pagination {
                    page,
                    per_page,
                    sort_by: String::new(),
                    ascending: true,
                }),
            });

            match client.get_mysimkari_pegawai(request).await {
                Ok(response) => {
                    let resp = response.into_inner();
                    Ok(resp.items)
                }
                Err(status) => Err(IntegrasiClientError::from_status(status)),
            }
        })
        .await
    }

    /// Execute an operation with retry logic (exponential backoff)
    async fn with_retry<F, Fut, T>(&self, operation: F) -> Result<T, IntegrasiClientError>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T, IntegrasiClientError>>,
    {
        let mut last_error = None;

        for attempt in 0..=self.config.max_retries {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(err) => {
                    // Don't retry on non-retryable errors
                    if !err.is_retryable() {
                        return Err(err);
                    }

                    last_error = Some(err);

                    if attempt < self.config.max_retries {
                        let delay = Duration::from_millis(100 * 2_u64.pow(attempt));
                        warn!(
                            "Integrasi gRPC call failed (attempt {}/{}), retrying in {:?}",
                            attempt + 1,
                            self.config.max_retries,
                            delay
                        );
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or(IntegrasiClientError::Unknown("Max retries exceeded".into())))
    }
}

/// Errors from the Integrasi gRPC client
#[derive(Debug, Clone)]
pub enum IntegrasiClientError {
    /// Connection or transport error
    Connection(String),
    /// Server returned an error
    Server(String),
    /// Request timeout
    Timeout(String),
    /// Service unavailable
    Unavailable(String),
    /// Unknown error
    Unknown(String),
}

impl IntegrasiClientError {
    /// Convert a tonic Status to IntegrasiClientError
    fn from_status(status: tonic::Status) -> Self {
        match status.code() {
            tonic::Code::Unavailable => {
                IntegrasiClientError::Unavailable(status.message().to_string())
            }
            tonic::Code::DeadlineExceeded => {
                IntegrasiClientError::Timeout(status.message().to_string())
            }
            _ => IntegrasiClientError::Server(format!(
                "gRPC error ({}): {}",
                status.code(),
                status.message()
            )),
        }
    }

    /// Check if this error is retryable
    fn is_retryable(&self) -> bool {
        matches!(
            self,
            IntegrasiClientError::Unavailable(_)
                | IntegrasiClientError::Timeout(_)
                | IntegrasiClientError::Connection(_)
        )
    }
}

impl std::fmt::Display for IntegrasiClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntegrasiClientError::Connection(msg) => write!(f, "Connection error: {}", msg),
            IntegrasiClientError::Server(msg) => write!(f, "Server error: {}", msg),
            IntegrasiClientError::Timeout(msg) => write!(f, "Timeout: {}", msg),
            IntegrasiClientError::Unavailable(msg) => write!(f, "Unavailable: {}", msg),
            IntegrasiClientError::Unknown(msg) => write!(f, "Unknown error: {}", msg),
        }
    }
}

impl std::error::Error for IntegrasiClientError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = IntegrasiClientConfig::default();
        assert_eq!(config.grpc_url, "http://localhost:50052");
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_error_retryable() {
        assert!(IntegrasiClientError::Unavailable("test".into()).is_retryable());
        assert!(IntegrasiClientError::Timeout("test".into()).is_retryable());
        assert!(IntegrasiClientError::Connection("test".into()).is_retryable());
        assert!(!IntegrasiClientError::Server("test".into()).is_retryable());
    }
}
