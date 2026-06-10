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

use authenc_types::domain::satker::{Satker, SatkerType};
use std::collections::HashMap;

/// Classify a satker into the authenc `SatkerType`. The Kejaksaan classification
/// (kejagung/kejati/kejari/cabjari) may live in EITHER `tipe_satker` OR
/// `kategori_satker` upstream, so we match against both. Best-effort keyword
/// match — MySIMKARI is the identity SoT (#42); verify exact values vs the real
/// API pull at staging. Order matters: check `cabang/cabjari` before
/// `negeri/kejari` (a cabjari sits under a kejari but is its own type).
pub fn classify_satker_type(tipe_satker: &str, kategori_satker: &str) -> SatkerType {
    let t = format!("{tipe_satker} {kategori_satker}").to_lowercase();
    if t.contains("agung") || t.contains("kejagung") || t.contains("pusat") {
        SatkerType::Pusat
    } else if t.contains("tinggi") || t.contains("kejati") {
        SatkerType::KejaksaanTinggi
    } else if t.contains("cabang") || t.contains("cabjari") {
        SatkerType::Cabang
    } else if t.contains("negeri") || t.contains("kejari") {
        SatkerType::KejaksaanNegeri
    } else {
        SatkerType::UnitKhusus
    }
}

/// Convert an integrasi `MysimkariSatker` (SoT identity) into the authenc
/// read-model `Satker`. authenc does NOT own this data — it derives it from
/// integrasi (1-way), so `id` is synthesized and timestamps are set at fetch.
///
/// Hierarchy is an **adjacency list keyed by upstream id**: `parent_id` references
/// the PARENT row's `api_id` (not its `kode_satker`). authenc's `SatkerHierarchy`
/// keys by `code`, so we resolve `parent_id -> parent's kode_satker` via
/// `api_id_to_code`. Fallback: if `parent_id` doesn't match any `api_id` it may
/// already be a `kode_satker` (kept as-is). `level` stays 0 (RBAC traversal uses
/// `parent_code`, not level).
pub fn mysimkari_to_satker(ms: &MysimkariSatker, api_id_to_code: &HashMap<&str, &str>) -> Satker {
    let now = chrono::Utc::now();
    let parent_code = {
        let p = ms.parent_id.trim();
        if p.is_empty() {
            None
        } else if let Some(code) = api_id_to_code.get(p) {
            Some((*code).to_string())
        } else {
            Some(p.to_string()) // fallback: parent_id may already be a kode_satker
        }
    };
    let mut attributes = serde_json::Map::new();
    if !ms.wilayah.is_empty() {
        attributes.insert("wilayah".into(), ms.wilayah.clone().into());
    }
    if !ms.provinsi.is_empty() {
        attributes.insert("provinsi".into(), ms.provinsi.clone().into());
    }
    if !ms.kategori_satker.is_empty() {
        attributes.insert("kategori_satker".into(), ms.kategori_satker.clone().into());
    }
    Satker {
        // Synthetic id — the read-model is keyed by `code` (RBAC key), not id.
        id: uuid::Uuid::new_v4(),
        code: ms.kode_satker.clone(),
        name: ms.nama_satker.clone(),
        description: if ms.alamat.is_empty() {
            None
        } else {
            Some(ms.alamat.clone())
        },
        parent_code,
        level: 0,
        satker_type: classify_satker_type(&ms.tipe_satker, &ms.kategori_satker),
        active: true,
        attributes: if attributes.is_empty() {
            None
        } else {
            Some(serde_json::Value::Object(attributes))
        },
        created_at: now,
        updated_at: now,
    }
}

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

    /// Get a single satker by its code, returning `None` when not found.
    pub async fn get_satker_by_code(&self, kode_satker: &str) -> Result<Option<MysimkariSatker>> {
        debug!(kode_satker = %kode_satker, "Fetching satker by code from integrasi");

        let mut client = self.client.clone();
        let request = GetMysimkariSatkerRequest {
            kode_satker: kode_satker.to_string(),
            pagination: Some(Pagination {
                page: 1,
                per_page: 1,
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

        Ok(response.into_inner().items.into_iter().next())
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

    /// Fetch the full satker list and map it to the authenc `Satker` read-model
    /// (identity SoT = integrasi/MySIMKARI). Used to build the RBAC hierarchy.
    ///
    /// Two-pass: first index `api_id -> kode_satker`, then map each record so the
    /// adjacency-list `parent_id` (which references the parent's upstream id)
    /// resolves to the parent's `kode_satker` (authenc's hierarchy key).
    pub async fn get_satker_readmodel(&self) -> Result<Vec<Satker>> {
        let items = self.get_satker_list().await?;
        let api_id_to_code: HashMap<&str, &str> = items
            .iter()
            .filter(|m| !m.api_id.is_empty())
            .map(|m| (m.api_id.as_str(), m.kode_satker.as_str()))
            .collect();
        Ok(items
            .iter()
            .map(|m| mysimkari_to_satker(m, &api_id_to_code))
            .collect())
    }
}
