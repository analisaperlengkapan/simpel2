use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tonic::transport::Channel;

use crate::shared::resilience::{
    CircuitBreaker, CircuitBreakerConfig, CircuitState, ResiliencePolicy, guarded,
};

pub mod secreton {
    pub mod v1 {
        tonic::include_proto!("secreton.v1");
    }
}

pub mod authenc {
    pub mod v1 {
        tonic::include_proto!("authenc.v1");
    }
}

pub mod common {
    pub mod v1 {
        tonic::include_proto!("common.v1");
    }
}

pub mod integrasi {
    pub mod v1 {
        tonic::include_proto!("integrasi.v1");
    }
}

use secreton::v1::secreton_service_client::SecretonServiceClient;
use secreton::v1::{GenerateDatabaseCredentialsRequest, GetSecretRequest, RenewLeaseRequest};

use authenc::v1::ValidateTokenRequest;
use authenc::v1::authenc_service_client::AuthencServiceClient;

use integrasi::v1::{
    DataSource, GetLastSyncTimestampsRequest, GetMonsaktiPersediaanRequest,
    GetMysimkariPegawaiRequest, GetMysimkariSatkerRequest, GetSimanAssetsRequest,
    GetSyncStatusRequest, HealthCheckRequest, Pagination, SimanAssetCategory,
    integrasi_service_client::IntegrasiServiceClient,
};

#[derive(Clone)]
pub struct SecretonClient {
    client: SecretonServiceClient<Channel>,
}

impl SecretonClient {
    pub async fn connect(addr: String) -> Result<Self> {
        let client = SecretonServiceClient::connect(addr).await?;
        Ok(Self { client })
    }

    pub async fn get_secret(&self, path: &str) -> Result<HashMap<String, String>> {
        let mut client = self.client.clone();
        let request = tonic::Request::new(GetSecretRequest {
            path: path.to_string(),
            version: None,
        });

        let response = client.get_secret(request).await?;
        Ok(response.into_inner().data)
    }

    /// Generate short-lived dynamic database credentials for `role_name` from
    /// Secreton's database secrets engine (Vault-style lease). The returned
    /// `connection_url` is used to build the pool; the `lease_id` must be kept
    /// alive via [`Self::renew_lease`] (see the renewal task in `main.rs`).
    pub async fn generate_database_credentials(
        &self,
        role_name: &str,
        ttl_seconds: Option<u32>,
    ) -> Result<DynamicDbCredentials> {
        let mut client = self.client.clone();
        let request = tonic::Request::new(GenerateDatabaseCredentialsRequest {
            role_name: role_name.to_string(),
            ttl_seconds,
        });

        let resp = client
            .generate_database_credentials(request)
            .await?
            .into_inner();

        let creds = resp
            .credentials
            .ok_or_else(|| anyhow::anyhow!("Secreton returned no database credentials"))?;
        let connection_url = creds.connection_url.ok_or_else(|| {
            anyhow::anyhow!("Secreton database credentials missing connection_url")
        })?;

        Ok(DynamicDbCredentials {
            connection_url,
            lease_id: resp.lease_id,
            lease_duration: resp.lease_duration,
            renewable: resp.renewable,
        })
    }

    /// Renew a lease (e.g. the dynamic DB credentials lease) so the underlying
    /// Postgres role stays valid and the existing pool keeps working without a
    /// credential change. `increment` requests a new TTL in seconds.
    pub async fn renew_lease(&self, lease_id: &str, increment: Option<i64>) -> Result<LeaseRenewal> {
        let mut client = self.client.clone();
        let request = tonic::Request::new(RenewLeaseRequest {
            lease_id: lease_id.to_string(),
            increment,
        });

        let resp = client.renew_lease(request).await?.into_inner();
        Ok(LeaseRenewal {
            lease_duration: resp.lease_duration,
            renewable: resp.renewable,
            renew_count: resp.renew_count,
            max_renewals: resp.max_renewals,
        })
    }
}

/// Dynamic database credentials issued by Secreton with an attached lease.
#[derive(Clone, Debug)]
pub struct DynamicDbCredentials {
    /// Full DSN to build the connection pool from.
    pub connection_url: String,
    /// Lease handle for renewal/revocation.
    pub lease_id: String,
    /// Lease validity in seconds.
    pub lease_duration: i64,
    /// Whether the lease can be renewed (vs. requiring re-issue/restart).
    pub renewable: bool,
}

/// Result of a [`SecretonClient::renew_lease`] call.
#[derive(Clone, Debug)]
pub struct LeaseRenewal {
    pub lease_duration: i64,
    pub renewable: bool,
    pub renew_count: u32,
    pub max_renewals: Option<u32>,
}

#[derive(Clone)]
pub struct AuthencClient {
    client: Option<AuthencServiceClient<Channel>>,
}

impl AuthencClient {
    pub async fn connect(addr: String) -> Result<Self> {
        let client = AuthencServiceClient::connect(addr).await?;
        Ok(Self {
            client: Some(client),
        })
    }

    /// Create a dummy/mock client for development without Authenc
    pub fn dummy() -> Self {
        Self { client: None }
    }

    pub async fn validate_token(&self, token: &str) -> Result<authenc::v1::ValidateTokenResponse> {
        if let Some(ref client) = self.client {
            let mut client = client.clone();
            let request = tonic::Request::new(ValidateTokenRequest {
                token: token.to_string(),
                required_scopes: vec![],
            });
            let response = client.validate_token(request).await?;
            Ok(response.into_inner())
        } else {
            // In dev mode without Authenc, accept all tokens
            tracing::warn!("Authenc not connected - accepting token in dev mode");

            let mut user_id = Some("00000000-0000-0000-0000-000000000001".to_string());
            let mut role = "admin".to_string();
            let mut satker_code = None;

            if token.starts_with("mock::") {
                let parts: Vec<&str> = token.split("::").collect();
                if parts.len() >= 4 {
                    role = parts[1].to_string();
                    user_id = Some(parts[2].to_string());
                    satker_code = Some(parts[3].to_string());
                }
            }

            Ok(authenc::v1::ValidateTokenResponse {
                valid: true,
                user_id,
                scopes: vec![],
                expires_at: None,
                error: None,
                username: None,
                name: None,
                nip: None,
                jabatan: None,
                satker_code,
                realm_roles: vec![role],
            })
        }
    }
}

/// Circuit breaker per sumber data eksternal — gagal independen, sehingga
/// down-nya MySIMKARI tidak ikut membuka sirkuit SIMAN/MonSAKTI.
#[derive(Debug)]
struct IntegrasiBreakers {
    mysimkari: CircuitBreaker,
    siman: CircuitBreaker,
    monsakti: CircuitBreaker,
}

impl IntegrasiBreakers {
    fn new() -> Self {
        let cfg = CircuitBreakerConfig::default();
        Self {
            mysimkari: CircuitBreaker::new("mysimkari", cfg.clone()),
            siman: CircuitBreaker::new("siman", cfg.clone()),
            monsakti: CircuitBreaker::new("monsakti", cfg),
        }
    }
}

/// gRPC Client for layanan-integrasi
/// Provides access to MonSAKTI, MySIMKARI, and SIMAN data.
///
/// Setiap panggilan data dibungkus circuit breaker + retry + timeout
/// (Fase 2.2) agar downtime sumber eksternal tidak menggantungkan request.
#[derive(Clone)]
pub struct IntegrasiClient {
    client: IntegrasiServiceClient<Channel>,
    breakers: Arc<IntegrasiBreakers>,
    policy: ResiliencePolicy,
}

impl IntegrasiClient {
    /// Connect to the integrasi gRPC service
    pub async fn connect(addr: String) -> Result<Self> {
        let client = IntegrasiServiceClient::connect(addr).await?;
        Ok(Self {
            client,
            breakers: Arc::new(IntegrasiBreakers::new()),
            policy: ResiliencePolicy::default(),
        })
    }

    /// Snapshot status sirkuit per sumber (untuk health endpoint / banner UI).
    pub fn circuit_states(&self) -> Vec<(&'static str, CircuitState)> {
        vec![
            ("mysimkari", self.breakers.mysimkari.state()),
            ("siman", self.breakers.siman.state()),
            ("monsakti", self.breakers.monsakti.state()),
        ]
    }

    /// Health check
    pub async fn health_check(&self) -> Result<integrasi::v1::HealthCheckResponse> {
        let mut client = self.client.clone();
        let request = tonic::Request::new(HealthCheckRequest {});
        let response = client.health_check(request).await?;
        Ok(response.into_inner())
    }

    /// Get sync status for a data source
    pub async fn get_sync_status(
        &self,
        source: DataSource,
    ) -> Result<integrasi::v1::GetSyncStatusResponse> {
        let mut client = self.client.clone();
        let request = tonic::Request::new(GetSyncStatusRequest {
            source: source.into(),
        });
        let response = client.get_sync_status(request).await?;
        Ok(response.into_inner())
    }

    /// Get last sync timestamps for all sources
    pub async fn get_last_sync_timestamps(
        &self,
    ) -> Result<integrasi::v1::GetLastSyncTimestampsResponse> {
        let mut client = self.client.clone();
        let request = tonic::Request::new(GetLastSyncTimestampsRequest {});
        let response = client.get_last_sync_timestamps(request).await?;
        Ok(response.into_inner())
    }

    /// Get MonSAKTI persediaan (inventory) data
    pub async fn get_monsakti_persediaan(
        &self,
        kode_kl: &str,
        kode_satker: &str,
        page: i32,
        per_page: i32,
    ) -> Result<integrasi::v1::GetMonsaktiPersediaanResponse> {
        guarded(&self.breakers.monsakti, &self.policy, || {
            let mut client = self.client.clone();
            let request = GetMonsaktiPersediaanRequest {
                kode_kl: kode_kl.to_string(),
                kode_satker: kode_satker.to_string(),
                pagination: Some(Pagination {
                    page,
                    per_page,
                    sort_by: String::new(),
                    ascending: true,
                }),
            };
            async move {
                client
                    .get_monsakti_persediaan(tonic::Request::new(request))
                    .await
                    .map(|r| r.into_inner())
            }
        })
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))
    }

    /// Get MySIMKARI satker (work units) data
    pub async fn get_mysimkari_satker(
        &self,
        kode_satker: Option<&str>,
        page: i32,
        per_page: i32,
    ) -> Result<integrasi::v1::GetMysimkariSatkerResponse> {
        guarded(&self.breakers.mysimkari, &self.policy, || {
            let mut client = self.client.clone();
            let request = GetMysimkariSatkerRequest {
                kode_satker: kode_satker.unwrap_or("").to_string(),
                pagination: Some(Pagination {
                    page,
                    per_page,
                    sort_by: String::new(),
                    ascending: true,
                }),
            };
            async move {
                client
                    .get_mysimkari_satker(tonic::Request::new(request))
                    .await
                    .map(|r| r.into_inner())
            }
        })
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))
    }

    /// Get MySIMKARI pegawai (employees) data
    pub async fn get_mysimkari_pegawai(
        &self,
        kode_satker: &str,
        page: i32,
        per_page: i32,
    ) -> Result<integrasi::v1::GetMysimkariPegawaiResponse> {
        guarded(&self.breakers.mysimkari, &self.policy, || {
            let mut client = self.client.clone();
            let request = GetMysimkariPegawaiRequest {
                kode_satker: kode_satker.to_string(),
                nama_filter: String::new(),
                nip_filter: String::new(),
                pagination: Some(Pagination {
                    page,
                    per_page,
                    sort_by: String::new(),
                    ascending: true,
                }),
            };
            async move {
                client
                    .get_mysimkari_pegawai(tonic::Request::new(request))
                    .await
                    .map(|r| r.into_inner())
            }
        })
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))
    }

    /// Get SIMAN assets by category
    pub async fn get_siman_assets(
        &self,
        category: SimanAssetCategory,
        kode_satker: Option<&str>,
        page: i32,
        per_page: i32,
    ) -> Result<integrasi::v1::GetSimanAssetsResponse> {
        guarded(&self.breakers.siman, &self.policy, || {
            let mut client = self.client.clone();
            let request = GetSimanAssetsRequest {
                category: category.into(),
                kode_satker: kode_satker.unwrap_or("").to_string(),
                pagination: Some(Pagination {
                    page,
                    per_page,
                    sort_by: String::new(),
                    ascending: true,
                }),
            };
            async move {
                client
                    .get_siman_assets(tonic::Request::new(request))
                    .await
                    .map(|r| r.into_inner())
            }
        })
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))
    }

    /// Get SIMAN tanah (land) assets
    pub async fn get_siman_tanah(
        &self,
        kode_satker: Option<&str>,
        page: i32,
        per_page: i32,
    ) -> Result<integrasi::v1::GetSimanAssetsResponse> {
        self.get_siman_assets(SimanAssetCategory::Tanah, kode_satker, page, per_page)
            .await
    }

    /// Get SIMAN gedung bangunan (buildings) assets
    pub async fn get_siman_gedung_bangunan(
        &self,
        kode_satker: Option<&str>,
        page: i32,
        per_page: i32,
    ) -> Result<integrasi::v1::GetSimanAssetsResponse> {
        self.get_siman_assets(
            SimanAssetCategory::GedungBangunan,
            kode_satker,
            page,
            per_page,
        )
        .await
    }

    /// Get SIMAN angkutan bermotor (vehicles) assets
    pub async fn get_siman_angkutan_bermotor(
        &self,
        kode_satker: Option<&str>,
        page: i32,
        per_page: i32,
    ) -> Result<integrasi::v1::GetSimanAssetsResponse> {
        self.get_siman_assets(
            SimanAssetCategory::AngkutanBermotor,
            kode_satker,
            page,
            per_page,
        )
        .await
    }
}

// Re-export commonly used types from integrasi proto
