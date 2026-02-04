//! gRPC Service Implementation for IntegrasiService
//!
//! This module implements the IntegrasiService gRPC interface, providing
//! data access to MonSAKTI, MySIMKARI, and SIMAN data stored in the database.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use tokio_postgres::Client;
use tonic::{Request, Response, Status};
use tracing::{debug, error, info, warn};

use crate::grpc::proto::{
    self, DataSource, GetLastSyncTimestampsRequest, GetLastSyncTimestampsResponse,
    GetMonsaktiAsetTetapRequest, GetMonsaktiAsetTetapResponse, GetMonsaktiKontrakPengadaanRequest,
    GetMonsaktiKontrakPengadaanResponse, GetMonsaktiPersediaanRequest,
    GetMonsaktiPersediaanResponse, GetMonsaktiRealisasiBelanjaRequest,
    GetMonsaktiRealisasiBelanjaResponse, GetMonsaktiReferencesRequest,
    GetMonsaktiReferencesResponse, GetMonsaktiSaldoAnggaranRequest,
    GetMonsaktiSaldoAnggaranResponse, GetMonsaktiSupplierRequest, GetMonsaktiSupplierResponse,
    GetMonsaktiTransaksiRequest, GetMonsaktiTransaksiResponse, GetMysimkariPegawaiRequest,
    GetMysimkariPegawaiResponse, GetMysimkariSatkerRequest, GetMysimkariSatkerResponse,
    GetSimanAssetRequest, GetSimanAssetResponse, GetSimanAssetsRequest, GetSimanAssetsResponse,
    GetSyncStatusRequest, GetSyncStatusResponse, HealthCheckRequest, HealthCheckResponse,
    MonsaktiAsetTetap, MonsaktiPersediaan, MonsaktiReference, MonsaktiTransaksi, MysimkariPegawai,
    MysimkariSatker, PaginationInfo, SaldoAnggaranSummary, SimanAsset, SyncState,
    TriggerSyncRequest, TriggerSyncResponse, integrasi_service_server::IntegrasiService,
};

/// Shared state for the gRPC service
#[derive(Clone)]
pub struct ServiceState {
    pub db_client: Arc<Client>,
    pub sync_status: Arc<RwLock<HashMap<DataSource, SyncStatus>>>,
}

/// Sync status for a data source
#[derive(Clone, Debug)]
pub struct SyncStatus {
    pub state: SyncState,
    pub last_sync_at: Option<chrono::DateTime<chrono::Utc>>,
    pub records_synced: i64,
    pub error_message: Option<String>,
}

impl Default for SyncStatus {
    fn default() -> Self {
        Self {
            state: SyncState::Idle,
            last_sync_at: None,
            records_synced: 0,
            error_message: None,
        }
    }
}

/// Implementation of IntegrasiService gRPC interface
#[derive(Clone)]
pub struct IntegrasiServiceImpl {
    state: ServiceState,
}

impl IntegrasiServiceImpl {
    /// Create a new IntegrasiServiceImpl with database connection
    pub fn new(db_client: Arc<Client>) -> Self {
        let mut sync_status = HashMap::new();
        sync_status.insert(DataSource::Monsakti, SyncStatus::default());
        sync_status.insert(DataSource::Mysimkari, SyncStatus::default());
        sync_status.insert(DataSource::Siman, SyncStatus::default());

        Self {
            state: ServiceState {
                db_client,
                sync_status: Arc::new(RwLock::new(sync_status)),
            },
        }
    }

    /// Helper to create pagination info from query result
    fn create_pagination_info(page: i32, per_page: i32, total_items: i64) -> PaginationInfo {
        let total_pages = if per_page > 0 {
            ((total_items as f64) / (per_page as f64)).ceil() as i32
        } else {
            1
        };

        PaginationInfo {
            current_page: page,
            per_page,
            total_items,
            total_pages,
        }
    }
}

#[tonic::async_trait]
impl IntegrasiService for IntegrasiServiceImpl {
    /// Health check endpoint
    async fn health_check(
        &self,
        _request: Request<HealthCheckRequest>,
    ) -> Result<Response<HealthCheckResponse>, Status> {
        info!("HealthCheck request received");

        // Check database connectivity
        let db_status = match self.state.db_client.simple_query("SELECT 1").await {
            Ok(_) => "connected".to_string(),
            Err(e) => format!("error: {}", e),
        };

        let healthy = db_status == "connected";

        let mut service_status = HashMap::new();
        service_status.insert("database".to_string(), db_status);
        service_status.insert("grpc".to_string(), "running".to_string());

        Ok(Response::new(HealthCheckResponse {
            healthy,
            version: env!("CARGO_PKG_VERSION").to_string(),
            database_status: if healthy {
                "connected".to_string()
            } else {
                "disconnected".to_string()
            },
            service_status,
        }))
    }

    /// Get sync status for a data source
    async fn get_sync_status(
        &self,
        request: Request<GetSyncStatusRequest>,
    ) -> Result<Response<GetSyncStatusResponse>, Status> {
        let req = request.into_inner();
        let source = DataSource::try_from(req.source).unwrap_or(DataSource::Unspecified);

        debug!("GetSyncStatus request for source: {:?}", source);

        let status = self
            .state
            .sync_status
            .read()
            .await
            .get(&source)
            .cloned()
            .unwrap_or_default();

        Ok(Response::new(GetSyncStatusResponse {
            source: source.into(),
            last_sync_at: status
                .last_sync_at
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_default(),
            next_sync_at: String::new(), // Will be populated from K8s CronJob schedule
            state: status.state.into(),
            records_synced: status.records_synced,
            error_message: status.error_message.unwrap_or_default(),
        }))
    }

    /// Get MonSAKTI reference data
    async fn get_monsakti_references(
        &self,
        request: Request<GetMonsaktiReferencesRequest>,
    ) -> Result<Response<GetMonsaktiReferencesResponse>, Status> {
        let req = request.into_inner();
        info!("GetMonsaktiReferences for KL: {}", req.kode_kl);

        let mut references = Vec::new();

        // Query adm_ref_admin table
        let query = r#"
            SELECT kode, nama, keterangan
            FROM integrasi.adm_ref_admin
            LIMIT 1000
        "#;

        match self.state.db_client.query(query, &[]).await {
            Ok(rows) => {
                for row in rows {
                    let code: String = row.try_get("kode").unwrap_or_default();
                    let name: String = row.try_get("nama").unwrap_or_default();
                    let description: String = row.try_get("keterangan").unwrap_or_default();

                    references.push(MonsaktiReference {
                        reference_type: "adm_ref_admin".to_string(),
                        code,
                        name,
                        description,
                        extra_fields: HashMap::new(),
                    });
                }
            }
            Err(e) => {
                warn!("Failed to query adm_ref_admin: {}", e);
            }
        }

        // Query adm_ref_bank table
        let query = r#"
            SELECT kode_bank, nama_bank
            FROM integrasi.adm_ref_bank
            LIMIT 1000
        "#;

        match self.state.db_client.query(query, &[]).await {
            Ok(rows) => {
                for row in rows {
                    let code: String = row.try_get("kode_bank").unwrap_or_default();
                    let name: String = row.try_get("nama_bank").unwrap_or_default();

                    references.push(MonsaktiReference {
                        reference_type: "adm_ref_bank".to_string(),
                        code,
                        name,
                        description: String::new(),
                        extra_fields: HashMap::new(),
                    });
                }
            }
            Err(e) => {
                warn!("Failed to query adm_ref_bank: {}", e);
            }
        }

        let total_count = references.len() as i64;

        Ok(Response::new(GetMonsaktiReferencesResponse {
            references,
            total_count,
        }))
    }

    /// Get MonSAKTI persediaan (inventory) data
    async fn get_monsakti_persediaan(
        &self,
        request: Request<GetMonsaktiPersediaanRequest>,
    ) -> Result<Response<GetMonsaktiPersediaanResponse>, Status> {
        let req = request.into_inner();
        let page = req.pagination.as_ref().map(|p| p.page).unwrap_or(1);
        let per_page = req.pagination.as_ref().map(|p| p.per_page).unwrap_or(100);
        let offset = ((page - 1) * per_page) as i64;

        info!(
            "GetMonsaktiPersediaan for satker: {}, page: {}",
            req.kode_satker, page
        );

        // Count total
        let count_query = "SELECT COUNT(*) FROM integrasi.per_persediaan";
        let total_items: i64 = match self.state.db_client.query_one(count_query, &[]).await {
            Ok(row) => row.try_get(0).unwrap_or(0),
            Err(e) => {
                error!("Failed to count persediaan: {}", e);
                0
            }
        };

        // Query data
        let query = r#"
            SELECT id, kode_satker, kode_barang, nama_barang, satuan,
                   jumlah, harga_satuan, total_nilai, tahun_anggaran
            FROM integrasi.per_persediaan
            ORDER BY id
            LIMIT $1 OFFSET $2
        "#;

        let items: Vec<MonsaktiPersediaan> = match self
            .state
            .db_client
            .query(query, &[&(per_page as i64), &offset])
            .await
        {
            Ok(rows) => rows
                .iter()
                .map(|row| MonsaktiPersediaan {
                    id: row
                        .try_get::<_, uuid::Uuid>("id")
                        .map(|u| u.to_string())
                        .unwrap_or_default(),
                    kode_satker: row.try_get("kode_satker").unwrap_or_default(),
                    kode_barang: row.try_get("kode_barang").unwrap_or_default(),
                    nama_barang: row.try_get("nama_barang").unwrap_or_default(),
                    satuan: row.try_get("satuan").unwrap_or_default(),
                    jumlah: row.try_get::<_, f64>("jumlah").unwrap_or(0.0),
                    harga_satuan: row.try_get::<_, f64>("harga_satuan").unwrap_or(0.0),
                    total_nilai: row.try_get::<_, f64>("total_nilai").unwrap_or(0.0),
                    tahun_anggaran: row.try_get("tahun_anggaran").unwrap_or_default(),
                    extra_fields: HashMap::new(),
                })
                .collect(),
            Err(e) => {
                error!("Failed to query persediaan: {}", e);
                Vec::new()
            }
        };

        Ok(Response::new(GetMonsaktiPersediaanResponse {
            items,
            pagination: Some(Self::create_pagination_info(page, per_page, total_items)),
        }))
    }

    /// Get MonSAKTI aset tetap (fixed assets) data
    async fn get_monsakti_aset_tetap(
        &self,
        request: Request<GetMonsaktiAsetTetapRequest>,
    ) -> Result<Response<GetMonsaktiAsetTetapResponse>, Status> {
        let req = request.into_inner();
        let page = req.pagination.as_ref().map(|p| p.page).unwrap_or(1);
        let per_page = req.pagination.as_ref().map(|p| p.per_page).unwrap_or(100);
        let offset = ((page - 1) * per_page) as i64;

        info!(
            "GetMonsaktiAsetTetap for satker: {}, page: {}",
            req.kode_satker, page
        );

        // Count total
        let count_query = "SELECT COUNT(*) FROM integrasi.ast_aset_tetap";
        let total_items: i64 = match self.state.db_client.query_one(count_query, &[]).await {
            Ok(row) => row.try_get(0).unwrap_or(0),
            Err(e) => {
                error!("Failed to count aset_tetap: {}", e);
                0
            }
        };

        // Query data (using generic structure since actual columns may vary)
        let query = r#"
            SELECT id, data
            FROM integrasi.ast_aset_tetap
            ORDER BY id
            LIMIT $1 OFFSET $2
        "#;

        let items: Vec<MonsaktiAsetTetap> = match self
            .state
            .db_client
            .query(query, &[&(per_page as i64), &offset])
            .await
        {
            Ok(rows) => rows
                .iter()
                .map(|row| {
                    let data: serde_json::Value =
                        row.try_get("data").unwrap_or(serde_json::Value::Null);
                    MonsaktiAsetTetap {
                        id: row
                            .try_get::<_, uuid::Uuid>("id")
                            .map(|u| u.to_string())
                            .unwrap_or_default(),
                        kode_satker: data["kdsatker"].as_str().unwrap_or_default().to_string(),
                        nup: data["nup"].as_str().unwrap_or_default().to_string(),
                        kode_barang: data["kdbarang"].as_str().unwrap_or_default().to_string(),
                        nama_barang: data["nmbarang"].as_str().unwrap_or_default().to_string(),
                        merk_type: data["merktype"].as_str().unwrap_or_default().to_string(),
                        nilai_perolehan: data["nlperolehan"].as_f64().unwrap_or(0.0),
                        nilai_penyusutan: data["nlpenyusutan"].as_f64().unwrap_or(0.0),
                        nilai_buku: data["nlbuku"].as_f64().unwrap_or(0.0),
                        kondisi: data["kondisi"].as_str().unwrap_or_default().to_string(),
                        tahun_perolehan: data["thnperolehan"]
                            .as_str()
                            .unwrap_or_default()
                            .to_string(),
                        extra_fields: HashMap::new(),
                    }
                })
                .collect(),
            Err(e) => {
                error!("Failed to query aset_tetap: {}", e);
                Vec::new()
            }
        };

        Ok(Response::new(GetMonsaktiAsetTetapResponse {
            items,
            pagination: Some(Self::create_pagination_info(page, per_page, total_items)),
        }))
    }

    /// Get MonSAKTI transaksi data
    async fn get_monsakti_transaksi(
        &self,
        request: Request<GetMonsaktiTransaksiRequest>,
    ) -> Result<Response<GetMonsaktiTransaksiResponse>, Status> {
        let req = request.into_inner();
        let page = req.pagination.as_ref().map(|p| p.page).unwrap_or(1);
        let per_page = req.pagination.as_ref().map(|p| p.per_page).unwrap_or(100);
        let offset = ((page - 1) * per_page) as i64;

        info!(
            "GetMonsaktiTransaksi for satker: {}, page: {}",
            req.kode_satker, page
        );

        // Count total
        let count_query = "SELECT COUNT(*) FROM integrasi.ang_transaksi";
        let total_items: i64 = match self.state.db_client.query_one(count_query, &[]).await {
            Ok(row) => row.try_get(0).unwrap_or(0),
            Err(e) => {
                error!("Failed to count transaksi: {}", e);
                0
            }
        };

        // Query data
        let query = r#"
            SELECT id, data
            FROM integrasi.ang_transaksi
            ORDER BY id
            LIMIT $1 OFFSET $2
        "#;

        let items: Vec<MonsaktiTransaksi> = match self
            .state
            .db_client
            .query(query, &[&(per_page as i64), &offset])
            .await
        {
            Ok(rows) => rows
                .iter()
                .map(|row| {
                    let data: serde_json::Value =
                        row.try_get("data").unwrap_or(serde_json::Value::Null);
                    MonsaktiTransaksi {
                        id: row
                            .try_get::<_, uuid::Uuid>("id")
                            .map(|u| u.to_string())
                            .unwrap_or_default(),
                        kode_satker: data["kdsatker"].as_str().unwrap_or_default().to_string(),
                        no_dokumen: data["nodok"].as_str().unwrap_or_default().to_string(),
                        tanggal_dokumen: data["tgldok"].as_str().unwrap_or_default().to_string(),
                        jenis_transaksi: data["jenistrx"].as_str().unwrap_or_default().to_string(),
                        nilai: data["nilai"].as_f64().unwrap_or(0.0),
                        keterangan: data["keterangan"].as_str().unwrap_or_default().to_string(),
                        extra_fields: HashMap::new(),
                    }
                })
                .collect(),
            Err(e) => {
                error!("Failed to query transaksi: {}", e);
                Vec::new()
            }
        };

        Ok(Response::new(GetMonsaktiTransaksiResponse {
            items,
            pagination: Some(Self::create_pagination_info(page, per_page, total_items)),
        }))
    }

    /// Get MonSAKTI kontrak pengadaan data
    async fn get_monsakti_kontrak_pengadaan(
        &self,
        request: Request<GetMonsaktiKontrakPengadaanRequest>,
    ) -> Result<Response<GetMonsaktiKontrakPengadaanResponse>, Status> {
        let req = request.into_inner();
        let page = req.pagination.as_ref().map(|p| p.page).unwrap_or(1);
        let per_page = req.pagination.as_ref().map(|p| p.per_page).unwrap_or(100);

        info!(
            "GetMonsaktiKontrakPengadaan for satker: {}, page: {}",
            req.kode_satker, page
        );

        // Return empty for now - table may not exist yet
        Ok(Response::new(GetMonsaktiKontrakPengadaanResponse {
            items: Vec::new(),
            pagination: Some(Self::create_pagination_info(page, per_page, 0)),
        }))
    }

    /// Get MonSAKTI supplier data
    async fn get_monsakti_supplier(
        &self,
        request: Request<GetMonsaktiSupplierRequest>,
    ) -> Result<Response<GetMonsaktiSupplierResponse>, Status> {
        let req = request.into_inner();
        let page = req.pagination.as_ref().map(|p| p.page).unwrap_or(1);
        let per_page = req.pagination.as_ref().map(|p| p.per_page).unwrap_or(100);

        info!(
            "GetMonsaktiSupplier for satker: {}, filter: {}",
            req.kode_satker, req.nama_filter
        );

        // Return empty for now - table may not exist yet
        Ok(Response::new(GetMonsaktiSupplierResponse {
            items: Vec::new(),
            pagination: Some(Self::create_pagination_info(page, per_page, 0)),
        }))
    }

    /// Get MonSAKTI realisasi belanja data
    async fn get_monsakti_realisasi_belanja(
        &self,
        request: Request<GetMonsaktiRealisasiBelanjaRequest>,
    ) -> Result<Response<GetMonsaktiRealisasiBelanjaResponse>, Status> {
        let req = request.into_inner();
        let page = req.pagination.as_ref().map(|p| p.page).unwrap_or(1);
        let per_page = req.pagination.as_ref().map(|p| p.per_page).unwrap_or(100);

        info!(
            "GetMonsaktiRealisasiBelanja for satker: {}, tahun: {}",
            req.kode_satker, req.tahun_anggaran
        );

        // Return empty for now - table may not exist yet
        Ok(Response::new(GetMonsaktiRealisasiBelanjaResponse {
            items: Vec::new(),
            pagination: Some(Self::create_pagination_info(page, per_page, 0)),
            total_pagu: 0.0,
            total_realisasi: 0.0,
            persentase_realisasi: 0.0,
        }))
    }

    /// Get MonSAKTI saldo anggaran data
    async fn get_monsakti_saldo_anggaran(
        &self,
        request: Request<GetMonsaktiSaldoAnggaranRequest>,
    ) -> Result<Response<GetMonsaktiSaldoAnggaranResponse>, Status> {
        let req = request.into_inner();

        info!(
            "GetMonsaktiSaldoAnggaran for satker: {}, tahun: {}",
            req.kode_satker, req.tahun_anggaran
        );

        // Return empty for now - table may not exist yet
        Ok(Response::new(GetMonsaktiSaldoAnggaranResponse {
            items: Vec::new(),
            summary: Some(SaldoAnggaranSummary {
                total_pagu: 0.0,
                total_realisasi: 0.0,
                total_sisa: 0.0,
                persentase_realisasi: 0.0,
            }),
        }))
    }

    /// Get MySIMKARI satker data
    async fn get_mysimkari_satker(
        &self,
        request: Request<GetMysimkariSatkerRequest>,
    ) -> Result<Response<GetMysimkariSatkerResponse>, Status> {
        let req = request.into_inner();
        let page = req.pagination.as_ref().map(|p| p.page).unwrap_or(1);
        let per_page = req.pagination.as_ref().map(|p| p.per_page).unwrap_or(100);
        let offset = ((page - 1) * per_page) as i64;

        info!(
            "GetMysimkariSatker filter: {}, page: {}",
            req.kode_satker, page
        );

        // Count total
        let count_query = "SELECT COUNT(*) FROM integrasi.mysimkari_satker";
        let total_items: i64 = match self.state.db_client.query_one(count_query, &[]).await {
            Ok(row) => row.try_get(0).unwrap_or(0),
            Err(e) => {
                error!("Failed to count mysimkari_satker: {}", e);
                0
            }
        };

        // Query data
        let query = r#"
            SELECT kode_satker, nama_satker, alamat, telepon, email,
                   kode_wilayah, nama_wilayah, jumlah_pegawai
            FROM integrasi.mysimkari_satker
            ORDER BY kode_satker
            LIMIT $1 OFFSET $2
        "#;

        let items: Vec<MysimkariSatker> = match self
            .state
            .db_client
            .query(query, &[&(per_page as i64), &offset])
            .await
        {
            Ok(rows) => rows
                .iter()
                .map(|row| MysimkariSatker {
                    kode_satker: row.try_get("kode_satker").unwrap_or_default(),
                    nama_satker: row.try_get("nama_satker").unwrap_or_default(),
                    alamat: row.try_get("alamat").unwrap_or_default(),
                    telepon: row.try_get("telepon").unwrap_or_default(),
                    email: row.try_get("email").unwrap_or_default(),
                    kode_wilayah: row.try_get("kode_wilayah").unwrap_or_default(),
                    nama_wilayah: row.try_get("nama_wilayah").unwrap_or_default(),
                    jumlah_pegawai: row.try_get::<_, i32>("jumlah_pegawai").unwrap_or(0),
                    extra_fields: HashMap::new(),
                })
                .collect(),
            Err(e) => {
                error!("Failed to query mysimkari_satker: {}", e);
                Vec::new()
            }
        };

        Ok(Response::new(GetMysimkariSatkerResponse {
            items,
            pagination: Some(Self::create_pagination_info(page, per_page, total_items)),
        }))
    }

    /// Get MySIMKARI pegawai data
    async fn get_mysimkari_pegawai(
        &self,
        request: Request<GetMysimkariPegawaiRequest>,
    ) -> Result<Response<GetMysimkariPegawaiResponse>, Status> {
        let req = request.into_inner();
        let page = req.pagination.as_ref().map(|p| p.page).unwrap_or(1);
        let per_page = req.pagination.as_ref().map(|p| p.per_page).unwrap_or(100);
        let offset = ((page - 1) * per_page) as i64;

        info!(
            "GetMysimkariPegawai for satker: {}, page: {}",
            req.kode_satker, page
        );

        // Count total
        let (count_query, count_params): (String, Vec<&(dyn tokio_postgres::types::ToSql + Sync)>) =
            if req.kode_satker.is_empty() {
                (
                    "SELECT COUNT(*) FROM integrasi.mysimkari_pegawai".to_string(),
                    vec![],
                )
            } else {
                (
                    "SELECT COUNT(*) FROM integrasi.mysimkari_pegawai WHERE kode_satker = $1"
                        .to_string(),
                    vec![&req.kode_satker],
                )
            };

        let total_items: i64 = match self
            .state
            .db_client
            .query_one(&count_query, &count_params)
            .await
        {
            Ok(row) => row.try_get(0).unwrap_or(0),
            Err(e) => {
                error!("Failed to count mysimkari_pegawai: {}", e);
                0
            }
        };

        // Query data
        let per_page_i64 = per_page as i64;
        let (query, params): (String, Vec<&(dyn tokio_postgres::types::ToSql + Sync)>) =
            if req.kode_satker.is_empty() {
                (
                    r#"
                SELECT nip, nama, jabatan, pangkat, golongan, unit_kerja,
                       kode_satker, email, telepon, status
                FROM integrasi.mysimkari_pegawai
                ORDER BY nama
                LIMIT $1 OFFSET $2
            "#
                    .to_string(),
                    vec![&per_page_i64, &offset],
                )
            } else {
                (
                    r#"
                SELECT nip, nama, jabatan, pangkat, golongan, unit_kerja,
                       kode_satker, email, telepon, status
                FROM integrasi.mysimkari_pegawai
                WHERE kode_satker = $1
                ORDER BY nama
                LIMIT $2 OFFSET $3
            "#
                    .to_string(),
                    vec![&req.kode_satker, &per_page_i64, &offset],
                )
            };

        let items: Vec<MysimkariPegawai> = match self.state.db_client.query(&query, &params).await {
            Ok(rows) => rows
                .iter()
                .map(|row| MysimkariPegawai {
                    nip: row.try_get("nip").unwrap_or_default(),
                    nama: row.try_get("nama").unwrap_or_default(),
                    jabatan: row.try_get("jabatan").unwrap_or_default(),
                    pangkat: row.try_get("pangkat").unwrap_or_default(),
                    golongan: row.try_get("golongan").unwrap_or_default(),
                    unit_kerja: row.try_get("unit_kerja").unwrap_or_default(),
                    kode_satker: row.try_get("kode_satker").unwrap_or_default(),
                    email: row.try_get("email").unwrap_or_default(),
                    telepon: row.try_get("telepon").unwrap_or_default(),
                    status: row.try_get("status").unwrap_or_default(),
                    extra_fields: HashMap::new(),
                })
                .collect(),
            Err(e) => {
                error!("Failed to query mysimkari_pegawai: {}", e);
                Vec::new()
            }
        };

        Ok(Response::new(GetMysimkariPegawaiResponse {
            items,
            pagination: Some(Self::create_pagination_info(page, per_page, total_items)),
        }))
    }

    /// Get SIMAN tanah assets
    async fn get_siman_tanah(
        &self,
        request: Request<GetSimanAssetRequest>,
    ) -> Result<Response<GetSimanAssetResponse>, Status> {
        self.get_siman_assets_by_table(request, "siman_aset_tanah")
            .await
    }

    /// Get SIMAN gedung bangunan assets
    async fn get_siman_gedung_bangunan(
        &self,
        request: Request<GetSimanAssetRequest>,
    ) -> Result<Response<GetSimanAssetResponse>, Status> {
        self.get_siman_assets_by_table(request, "siman_aset_gedung_bangunan")
            .await
    }

    /// Get SIMAN alat besar assets
    async fn get_siman_alat_besar(
        &self,
        request: Request<GetSimanAssetRequest>,
    ) -> Result<Response<GetSimanAssetResponse>, Status> {
        self.get_siman_assets_by_table(request, "siman_aset_alat_besar")
            .await
    }

    /// Get SIMAN angkutan bermotor assets
    async fn get_siman_angkutan_bermotor(
        &self,
        request: Request<GetSimanAssetRequest>,
    ) -> Result<Response<GetSimanAssetResponse>, Status> {
        self.get_siman_assets_by_table(request, "siman_aset_angkutan_bermotor")
            .await
    }

    /// Get all SIMAN assets with category filter
    async fn get_siman_assets(
        &self,
        request: Request<GetSimanAssetsRequest>,
    ) -> Result<Response<GetSimanAssetsResponse>, Status> {
        let req = request.into_inner();
        let category = proto::SimanAssetCategory::try_from(req.category)
            .unwrap_or(proto::SimanAssetCategory::Unspecified);

        let table_name = match category {
            proto::SimanAssetCategory::Tanah => "siman_aset_tanah",
            proto::SimanAssetCategory::GedungBangunan => "siman_aset_gedung_bangunan",
            proto::SimanAssetCategory::AlatBesar => "siman_aset_alat_besar",
            proto::SimanAssetCategory::AngkutanBermotor => "siman_aset_angkutan_bermotor",
            proto::SimanAssetCategory::AlatPersenjataan => "siman_aset_alat_persenjataan",
            proto::SimanAssetCategory::TakBerwujud => "siman_aset_tak_berwujud",
            proto::SimanAssetCategory::TetapLainnya => "siman_aset_tetap_lainnya",
            proto::SimanAssetCategory::BangunanAir => "siman_aset_bangunan_air",
            proto::SimanAssetCategory::InstalasiJaringan => "siman_aset_instalasi_jaringan",
            proto::SimanAssetCategory::JalanJembatan => "siman_aset_jalan_jembatan",
            proto::SimanAssetCategory::Kdp => "siman_aset_kdp",
            proto::SimanAssetCategory::KhususTik => "siman_aset_khusus_tik",
            proto::SimanAssetCategory::NonTik => "siman_aset_non_tik",
            proto::SimanAssetCategory::Rumah => "siman_aset_rumah",
            proto::SimanAssetCategory::TetapRenovasi => "siman_aset_tetap_renovasi",
            proto::SimanAssetCategory::Unspecified => {
                return Err(Status::invalid_argument("Category must be specified"));
            }
        };

        let inner_request = Request::new(GetSimanAssetRequest {
            kode_satker: req.kode_satker,
            pagination: req.pagination,
        });

        let response = self
            .get_siman_assets_by_table(inner_request, table_name)
            .await?;
        let inner = response.into_inner();

        Ok(Response::new(GetSimanAssetsResponse {
            category: req.category,
            items: inner.items,
            pagination: inner.pagination,
        }))
    }

    /// Trigger a sync operation
    async fn trigger_sync(
        &self,
        request: Request<TriggerSyncRequest>,
    ) -> Result<Response<TriggerSyncResponse>, Status> {
        let req = request.into_inner();
        let source = DataSource::try_from(req.source).unwrap_or(DataSource::Unspecified);

        info!("TriggerSync requested for source: {:?}", source);

        // Update sync status to running
        {
            let mut status_map = self.state.sync_status.write().await;
            if let Some(status) = status_map.get_mut(&source) {
                status.state = SyncState::Running;
            }
        }

        // In production, this would trigger the actual sync via K8s Job or similar
        // For now, we just acknowledge the request

        Ok(Response::new(TriggerSyncResponse {
            accepted: true,
            job_id: uuid::Uuid::new_v4().to_string(),
            message: format!(
                "Sync triggered for {:?}. Use CronJob for scheduled syncs.",
                source
            ),
            estimated_completion: String::new(),
        }))
    }

    /// Get last sync timestamps for all sources
    async fn get_last_sync_timestamps(
        &self,
        _request: Request<GetLastSyncTimestampsRequest>,
    ) -> Result<Response<GetLastSyncTimestampsResponse>, Status> {
        info!("GetLastSyncTimestamps requested");

        let mut timestamps = HashMap::new();
        let mut record_counts = HashMap::new();

        // Get last sync from data_sync_log table
        let query = r#"
            SELECT source, MAX(sync_started_at) as last_sync, SUM(records_synced) as total_records
            FROM integrasi.data_sync_log
            GROUP BY source
        "#;

        match self.state.db_client.query(query, &[]).await {
            Ok(rows) => {
                for row in rows {
                    let source: String = row.try_get("source").unwrap_or_default();
                    let last_sync: Option<chrono::DateTime<chrono::Utc>> =
                        row.try_get("last_sync").ok();
                    let total_records: i64 = row.try_get("total_records").unwrap_or(0);

                    if let Some(ts) = last_sync {
                        timestamps.insert(source.clone(), ts.to_rfc3339());
                    }
                    record_counts.insert(source, total_records);
                }
            }
            Err(e) => {
                warn!("Failed to query data_sync_log: {}", e);
            }
        }

        Ok(Response::new(GetLastSyncTimestampsResponse {
            timestamps,
            record_counts,
        }))
    }
}

impl IntegrasiServiceImpl {
    /// Helper to query SIMAN assets from a specific table
    async fn get_siman_assets_by_table(
        &self,
        request: Request<GetSimanAssetRequest>,
        table_name: &str,
    ) -> Result<Response<GetSimanAssetResponse>, Status> {
        let req = request.into_inner();
        let page = req.pagination.as_ref().map(|p| p.page).unwrap_or(1);
        let per_page = req.pagination.as_ref().map(|p| p.per_page).unwrap_or(100);
        let offset = ((page - 1) * per_page) as i64;

        info!(
            "GetSimanAssets from table: {}, satker: {}, page: {}",
            table_name, req.kode_satker, page
        );

        // Count total
        let count_query = format!("SELECT COUNT(*) FROM integrasi.{}", table_name);
        let total_items: i64 = match self.state.db_client.query_one(&count_query, &[]).await {
            Ok(row) => row.try_get(0).unwrap_or(0),
            Err(e) => {
                error!("Failed to count {}: {}", table_name, e);
                0
            }
        };

        // Query data
        let query = format!(
            r#"
            SELECT id, nup, kode_barang, nama_barang, kode_satker, nama_satker,
                   nilai_perolehan, nilai_buku, kondisi, tahun_perolehan, lokasi, status_penggunaan
            FROM integrasi.{}
            ORDER BY id
            LIMIT $1 OFFSET $2
        "#,
            table_name
        );

        let items: Vec<SimanAsset> = match self
            .state
            .db_client
            .query(&query, &[&(per_page as i64), &offset])
            .await
        {
            Ok(rows) => rows
                .iter()
                .map(|row| SimanAsset {
                    id: row
                        .try_get::<_, uuid::Uuid>("id")
                        .map(|u| u.to_string())
                        .unwrap_or_default(),
                    nup: row.try_get("nup").unwrap_or_default(),
                    kode_barang: row.try_get("kode_barang").unwrap_or_default(),
                    nama_barang: row.try_get("nama_barang").unwrap_or_default(),
                    kode_satker: row.try_get("kode_satker").unwrap_or_default(),
                    nama_satker: row.try_get("nama_satker").unwrap_or_default(),
                    category: 0, // Will be set by caller
                    nilai_perolehan: row.try_get::<_, f64>("nilai_perolehan").unwrap_or(0.0),
                    nilai_buku: row.try_get::<_, f64>("nilai_buku").unwrap_or(0.0),
                    kondisi: row.try_get("kondisi").unwrap_or_default(),
                    tahun_perolehan: row.try_get("tahun_perolehan").unwrap_or_default(),
                    lokasi: row.try_get("lokasi").unwrap_or_default(),
                    status_penggunaan: row.try_get("status_penggunaan").unwrap_or_default(),
                    extra_fields: HashMap::new(),
                })
                .collect(),
            Err(e) => {
                error!("Failed to query {}: {}", table_name, e);
                Vec::new()
            }
        };

        Ok(Response::new(GetSimanAssetResponse {
            items,
            pagination: Some(Self::create_pagination_info(page, per_page, total_items)),
        }))
    }
}
