//! gRPC client for layanan-integrasi
//!
//! Provides on-demand access to MySIMKARI pegawai and satker data
//! via the integrasi gRPC service.

use authenc_core::config::IntegrasiConfig;
use authenc_types::{AuthencError, result::Result};
use std::time::Duration;
use tonic::transport::Channel;
use tracing::{debug, info, warn};

// Include the generated gRPC client code
pub mod integrasi_proto {
    tonic::include_proto!("integrasi.v1");
}

use integrasi_proto::integrasi_service_client::IntegrasiServiceClient;
use integrasi_proto::{
    GetMysimkariPegawaiRequest, GetMysimkariSatkerRequest, HealthCheckRequest, MysimkariPegawai,
    MysimkariSatker, Pagination,
};

/// gRPC client wrapper for the integrasi service
#[derive(Clone)]
pub struct IntegrasiGrpcClient {
    client: IntegrasiServiceClient<Channel>,
}

impl IntegrasiGrpcClient {
    /// Create a new IntegrasiGrpcClient from config
    pub async fn new(config: &IntegrasiConfig) -> Result<Self> {
        info!(url = %config.grpc_url, "Connecting to integrasi gRPC service");

        let channel = Channel::from_shared(config.grpc_url.clone())
            .map_err(|e| AuthencError::config(format!("Invalid integrasi gRPC URL: {}", e)))?
            .connect_timeout(Duration::from_secs(config.connection_timeout_secs))
            .timeout(Duration::from_secs(config.request_timeout_secs))
            .connect()
            .await
            .map_err(|e| {
                AuthencError::internal(format!("Failed to connect to integrasi gRPC: {}", e))
            })?;

        info!("Connected to integrasi gRPC service");
        Ok(Self {
            client: IntegrasiServiceClient::new(channel),
        })
    }

    /// Create a lazy client (connects on first use) from config
    pub fn new_lazy(config: &IntegrasiConfig) -> Result<Self> {
        info!(url = %config.grpc_url, "Creating lazy integrasi gRPC client");

        let channel = Channel::from_shared(config.grpc_url.clone())
            .map_err(|e| AuthencError::config(format!("Invalid integrasi gRPC URL: {}", e)))?
            .connect_timeout(Duration::from_secs(config.connection_timeout_secs))
            .timeout(Duration::from_secs(config.request_timeout_secs))
            .connect_lazy();

        Ok(Self {
            client: IntegrasiServiceClient::new(channel),
        })
    }

    /// Health check the integrasi service
    pub async fn health_check(&self) -> Result<bool> {
        let mut client = self.client.clone();
        match client
            .health_check(tonic::Request::new(HealthCheckRequest {}))
            .await
        {
            Ok(response) => Ok(response.into_inner().healthy),
            Err(status) => {
                warn!(
                    code = ?status.code(),
                    message = %status.message(),
                    "Integrasi health check failed"
                );
                Ok(false)
            }
        }
    }

    /// Get a single pegawai by NIP
    pub async fn get_pegawai_by_nip(&self, nip: &str) -> Result<Option<MysimkariPegawai>> {
        debug!(nip = %nip, "Fetching pegawai by NIP from integrasi");

        let mut client = self.client.clone();
        let request = GetMysimkariPegawaiRequest {
            kode_satker: String::new(),
            nama_filter: String::new(),
            nip_filter: nip.to_string(),
            pagination: Some(Pagination {
                page: 1,
                per_page: 1,
                sort_by: String::new(),
                ascending: true,
            }),
        };

        let response = client
            .get_mysimkari_pegawai(tonic::Request::new(request))
            .await
            .map_err(|e| {
                AuthencError::internal(format!(
                    "gRPC call get_mysimkari_pegawai failed: {}",
                    e.message()
                ))
            })?;

        let items = response.into_inner().items;
        Ok(items.into_iter().next())
    }

    /// Get pegawai list by satker code
    pub async fn get_pegawai_by_satker(&self, kode_satker: &str) -> Result<Vec<MysimkariPegawai>> {
        debug!(kode_satker = %kode_satker, "Fetching pegawai by satker from integrasi");

        let mut client = self.client.clone();
        let request = GetMysimkariPegawaiRequest {
            kode_satker: kode_satker.to_string(),
            nama_filter: String::new(),
            nip_filter: String::new(),
            pagination: Some(Pagination {
                page: 1,
                per_page: 1000,
                sort_by: String::new(),
                ascending: true,
            }),
        };

        let response = client
            .get_mysimkari_pegawai(tonic::Request::new(request))
            .await
            .map_err(|e| {
                AuthencError::internal(format!(
                    "gRPC call get_mysimkari_pegawai failed: {}",
                    e.message()
                ))
            })?;

        Ok(response.into_inner().items)
    }

    /// Get satker list
    pub async fn get_satker_list(&self) -> Result<Vec<MysimkariSatker>> {
        debug!("Fetching satker list from integrasi");

        let mut client = self.client.clone();
        let request = GetMysimkariSatkerRequest {
            kode_satker: String::new(),
            pagination: Some(Pagination {
                page: 1,
                per_page: 10000,
                sort_by: String::new(),
                ascending: true,
            }),
        };

        let response = client
            .get_mysimkari_satker(tonic::Request::new(request))
            .await
            .map_err(|e| {
                AuthencError::internal(format!(
                    "gRPC call get_mysimkari_satker failed: {}",
                    e.message()
                ))
            })?;

        Ok(response.into_inner().items)
    }
}
