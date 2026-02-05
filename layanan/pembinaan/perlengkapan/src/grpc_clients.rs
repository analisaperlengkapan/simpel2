use anyhow::Result;
use std::collections::HashMap;
use tonic::transport::Channel;

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

use secreton::v1::GetSecretRequest;
use secreton::v1::secreton_service_client::SecretonServiceClient;

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
}

#[derive(Clone)]
pub struct AuthencClient {
    client: AuthencServiceClient<Channel>,
}

impl AuthencClient {
    pub async fn connect(addr: String) -> Result<Self> {
        let client = AuthencServiceClient::connect(addr).await?;
        Ok(Self { client })
    }

    pub async fn validate_token(&self, token: &str) -> Result<authenc::v1::ValidateTokenResponse> {
        let mut client = self.client.clone();
        let request = tonic::Request::new(ValidateTokenRequest {
            token: token.to_string(),
            required_scopes: vec![],
        });

        let response = client.validate_token(request).await?;
        Ok(response.into_inner())
    }
}

/// gRPC Client for layanan-integrasi
/// Provides access to MonSAKTI, MySIMKARI, and SIMAN data
#[derive(Clone)]
pub struct IntegrasiClient {
    client: IntegrasiServiceClient<Channel>,
}

impl IntegrasiClient {
    /// Connect to the integrasi gRPC service
    pub async fn connect(addr: String) -> Result<Self> {
        let client = IntegrasiServiceClient::connect(addr).await?;
        Ok(Self { client })
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
        let mut client = self.client.clone();
        let request = tonic::Request::new(GetMonsaktiPersediaanRequest {
            kode_kl: kode_kl.to_string(),
            kode_satker: kode_satker.to_string(),
            pagination: Some(Pagination {
                page,
                per_page,
                sort_by: String::new(),
                ascending: true,
            }),
        });
        let response = client.get_monsakti_persediaan(request).await?;
        Ok(response.into_inner())
    }

    /// Get MySIMKARI satker (work units) data
    pub async fn get_mysimkari_satker(
        &self,
        kode_satker: Option<&str>,
        page: i32,
        per_page: i32,
    ) -> Result<integrasi::v1::GetMysimkariSatkerResponse> {
        let mut client = self.client.clone();
        let request = tonic::Request::new(GetMysimkariSatkerRequest {
            kode_satker: kode_satker.unwrap_or("").to_string(),
            pagination: Some(Pagination {
                page,
                per_page,
                sort_by: String::new(),
                ascending: true,
            }),
        });
        let response = client.get_mysimkari_satker(request).await?;
        Ok(response.into_inner())
    }

    /// Get MySIMKARI pegawai (employees) data
    pub async fn get_mysimkari_pegawai(
        &self,
        kode_satker: &str,
        page: i32,
        per_page: i32,
    ) -> Result<integrasi::v1::GetMysimkariPegawaiResponse> {
        let mut client = self.client.clone();
        let request = tonic::Request::new(GetMysimkariPegawaiRequest {
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
        let response = client.get_mysimkari_pegawai(request).await?;
        Ok(response.into_inner())
    }

    /// Get SIMAN assets by category
    pub async fn get_siman_assets(
        &self,
        category: SimanAssetCategory,
        kode_satker: Option<&str>,
        page: i32,
        per_page: i32,
    ) -> Result<integrasi::v1::GetSimanAssetsResponse> {
        let mut client = self.client.clone();
        let request = tonic::Request::new(GetSimanAssetsRequest {
            category: category.into(),
            kode_satker: kode_satker.unwrap_or("").to_string(),
            pagination: Some(Pagination {
                page,
                per_page,
                sort_by: String::new(),
                ascending: true,
            }),
        });
        let response = client.get_siman_assets(request).await?;
        Ok(response.into_inner())
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
