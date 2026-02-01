use crate::{errors::AppResult, models::*, repository::PerlengkapanRepository};
use async_trait::async_trait;
use mockall::mock;
use uuid::Uuid;

// Define the mock repository at file scope so it's visible to submodules
mock! {
    pub Repository {}
    #[async_trait]
    impl PerlengkapanRepository for Repository {
        async fn get_dashboard_stats(&self) -> AppResult<DashboardStats>;
        async fn get_all_assets(&self, page: i32, per_page: i32, category: Option<String>) -> AppResult<(Vec<Asset>, i64)>;
        async fn get_asset_by_id(&self, id: Uuid) -> AppResult<Asset>;
        async fn get_all_pengadaan(&self, page: i32, per_page: i32) -> AppResult<(Vec<Pengadaan>, i64)>;
        async fn get_pengadaan_by_id(&self, id: Uuid) -> AppResult<Pengadaan>;
        async fn create_pengadaan(&self, request: CreatePengadaanRequest, user_id: Option<Uuid>) -> AppResult<Pengadaan>;

        // Sub-documents
        async fn create_pengadaan_hps(&self, request: CreatePengadaanHpsRequest) -> AppResult<PengadaanHps>;
        async fn get_pengadaan_hps(&self, pengadaan_id: Uuid) -> AppResult<Vec<PengadaanHps>>;

        async fn create_pengadaan_skppbj(&self, request: CreatePengadaanSkppbjRequest) -> AppResult<PengadaanSkppbj>;
        async fn get_pengadaan_skppbj(&self, pengadaan_id: Uuid) -> AppResult<Vec<PengadaanSkppbj>>;

        async fn create_pengadaan_spk(&self, request: CreatePengadaanSpkRequest) -> AppResult<PengadaanSpk>;
        async fn get_pengadaan_spk(&self, pengadaan_id: Uuid) -> AppResult<Vec<PengadaanSpk>>;

        async fn create_pengadaan_ringkasan(&self, request: CreatePengadaanRingkasanRequest) -> AppResult<PengadaanRingkasan>;
        async fn get_pengadaan_ringkasan(&self, pengadaan_id: Uuid) -> AppResult<Vec<PengadaanRingkasan>>;

        async fn create_pengadaan_kontrak(&self, request: CreatePengadaanKontrakRequest) -> AppResult<PengadaanKontrak>;
        async fn get_pengadaan_kontrak(&self, pengadaan_id: Uuid) -> AppResult<Vec<PengadaanKontrak>>;

        async fn create_pengadaan_bast(&self, request: CreatePengadaanBastRequest) -> AppResult<PengadaanBast>;
        async fn get_pengadaan_bast(&self, pengadaan_id: Uuid) -> AppResult<Vec<PengadaanBast>>;

        async fn create_pengadaan_nodis(&self, request: CreatePengadaanNodisRequest) -> AppResult<PengadaanNodis>;
        async fn get_pengadaan_nodis(&self, pengadaan_id: Uuid) -> AppResult<Vec<PengadaanNodis>>;

        async fn get_all_analisis(&self, page: i32, per_page: i32) -> AppResult<(Vec<AnalisisKebutuhan>, i64)>;
        async fn create_analisis(&self, request: CreateAnalisisRequest, user_id: Option<Uuid>) -> AppResult<AnalisisKebutuhan>;
        async fn get_analisis_by_id(&self, id: Uuid) -> AppResult<AnalisisKebutuhan>;
        async fn get_all_pemakaian(&self, page: i32, per_page: i32) -> AppResult<(Vec<Pemakaian>, i64)>;
        async fn create_pemakaian(&self, request: CreatePemakaianRequest, user_id: Option<Uuid>) -> AppResult<Pemakaian>;
        async fn get_pemakaian_by_id(&self, id: Uuid) -> AppResult<Pemakaian>;
        async fn get_all_hibah(&self, page: i32, per_page: i32) -> AppResult<(Vec<Hibah>, i64)>;
        async fn create_hibah(&self, request: CreateHibahRequest, user_id: Option<Uuid>) -> AppResult<Hibah>;
        async fn get_hibah_by_id(&self, id: Uuid) -> AppResult<Hibah>;
        async fn get_all_mutasi(&self, page: i32, per_page: i32) -> AppResult<(Vec<Mutasi>, i64)>;
        async fn create_mutasi(&self, request: CreateMutasiRequest, user_id: Option<Uuid>) -> AppResult<Mutasi>;
        async fn get_mutasi_by_id(&self, id: Uuid) -> AppResult<Mutasi>;
        async fn get_all_penghapusan(&self, page: i32, per_page: i32) -> AppResult<(Vec<Penghapusan>, i64)>;
        async fn create_penghapusan(&self, request: CreatePenghapusanRequest, user_id: Option<Uuid>) -> AppResult<Penghapusan>;
        async fn get_penghapusan_by_id(&self, id: Uuid) -> AppResult<Penghapusan>;
        async fn get_all_pengalihan(&self, page: i32, per_page: i32) -> AppResult<(Vec<Pengalihan>, i64)>;
        async fn create_pengalihan(&self, request: CreatePengalihanRequest, user_id: Option<Uuid>) -> AppResult<Pengalihan>;
        async fn get_pengalihan_by_id(&self, id: Uuid) -> AppResult<Pengalihan>;
        async fn get_all_pemeliharaan(&self, page: i32, per_page: i32) -> AppResult<(Vec<Pemeliharaan>, i64)>;
        async fn create_pemeliharaan(&self, request: CreatePemeliharaanRequest, user_id: Option<Uuid>) -> AppResult<Pemeliharaan>;
        async fn get_pemeliharaan_by_id(&self, id: Uuid) -> AppResult<Pemeliharaan>;
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use crate::services::PerlengkapanService;
    use chrono::Utc;
    use mockall::predicate::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_get_dashboard_stats() {
        let mut mock_repo = MockRepository::new();

        mock_repo
            .expect_get_dashboard_stats()
            .times(1)
            .returning(|| {
                Ok(DashboardStats {
                    total_aset: 100,
                    total_nilai_aset: 1000000.0,
                    total_satker: 5,
                    aset_baik: 80,
                    aset_rusak: 20,
                    categories: vec![],
                })
            });

        let service = PerlengkapanService::new(Arc::new(mock_repo));
        let stats = service.get_dashboard_stats().await.unwrap();

        assert_eq!(stats.total_aset, 100);
        assert_eq!(stats.aset_baik, 80);
    }

    #[tokio::test]
    async fn test_get_all_assets() {
        let mut mock_repo = MockRepository::new();
        let asset_id = Uuid::new_v4();

        mock_repo
            .expect_get_all_assets()
            .with(eq(1), eq(20), eq(None))
            .times(1)
            .returning(move |_, _, _| {
                Ok((
                    vec![Asset {
                        id: asset_id,
                        kategori_aset: "Tanah".to_string(),
                        no_aset: "1".to_string(),
                        nama_aset: Some("Tanah Kantor".to_string()),
                        kode_barang: Some("101".to_string()),
                        merk: None,
                        tipe: None,
                        kondisi: Some("Baik".to_string()),
                        lokasi: Some("Jakarta".to_string()),
                        satker: Some("Pusat".to_string()),
                        nilai_perolehan: Some(1000000.0),
                        tgl_perolehan: Some("2023-01-01".to_string()),
                        updated_at: Utc::now(),
                    }],
                    1,
                ))
            });

        let service = PerlengkapanService::new(Arc::new(mock_repo));
        let (assets, total) = service.get_all_assets(1, 20, None).await.unwrap();

        assert_eq!(total, 1);
        assert_eq!(assets[0].id, asset_id);
    }

    #[tokio::test]
    async fn test_get_asset_by_id() {
        let mut mock_repo = MockRepository::new();
        let asset_id = Uuid::new_v4();

        mock_repo
            .expect_get_asset_by_id()
            .with(eq(asset_id))
            .times(1)
            .returning(move |_| {
                Ok(Asset {
                    id: asset_id,
                    kategori_aset: "Tanah".to_string(),
                    no_aset: "1".to_string(),
                    nama_aset: Some("Tanah Kantor".to_string()),
                    kode_barang: Some("101".to_string()),
                    merk: None,
                    tipe: None,
                    kondisi: Some("Baik".to_string()),
                    lokasi: Some("Jakarta".to_string()),
                    satker: Some("Pusat".to_string()),
                    nilai_perolehan: Some(1000000.0),
                    tgl_perolehan: Some("2023-01-01".to_string()),
                    updated_at: Utc::now(),
                })
            });

        let service = PerlengkapanService::new(Arc::new(mock_repo));
        let asset = service.get_asset_by_id(asset_id).await.unwrap();

        assert_eq!(asset.id, asset_id);
    }

    #[tokio::test]
    async fn test_create_pengadaan() {
        let mut mock_repo = MockRepository::new();
        let user_id = Uuid::new_v4();
        let req = CreatePengadaanRequest {
            judul: "New Pengadaan".to_string(),
            deskripsi: Some("Desc".to_string()),
            jenis: "TIK".to_string(),
            anggaran: Some(5000000.0),
            target_selesai: None,
            pic_user_id: None,
        };

        mock_repo
            .expect_create_pengadaan()
            .with(always(), eq(Some(user_id)))
            .times(1)
            .returning(|req, uid| {
                Ok(Pengadaan {
                    id: Uuid::new_v4(),
                    judul: req.judul,
                    deskripsi: req.deskripsi,
                    jenis: req.jenis,
                    status: "perencanaan".to_string(),
                    anggaran: req.anggaran,
                    target_selesai: req.target_selesai,
                    pic_user_id: req.pic_user_id,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    created_by: uid,
                    updated_by: uid,
                })
            });

        let service = PerlengkapanService::new(Arc::new(mock_repo));
        let result = service.create_pengadaan(req, Some(user_id)).await.unwrap();

        assert_eq!(result.judul, "New Pengadaan");
        assert_eq!(result.created_by, Some(user_id));
    }

    #[tokio::test]
    async fn test_create_pengadaan_hps() {
        let mut mock_repo = MockRepository::new();
        let pengadaan_id = Uuid::new_v4();
        let req = CreatePengadaanHpsRequest {
            pengadaan_id,
            no_hps: "HPS-001".to_string(),
            tgl_hps: chrono::NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            nip_penandatangan: "123".to_string(),
            nama_penandatangan: "Pejabat".to_string(),
            pangkat_penandatangan: "IV/a".to_string(),
            barang: serde_json::json!([{ "item": "Laptop", "price": 1000 }]),
        };

        mock_repo
            .expect_create_pengadaan_hps()
            .with(always())
            .times(1)
            .returning(move |req| {
                Ok(PengadaanHps {
                    id: Uuid::new_v4(),
                    pengadaan_id: req.pengadaan_id,
                    no_hps: req.no_hps,
                    tgl_hps: req.tgl_hps,
                    nip_penandatangan: req.nip_penandatangan,
                    nama_penandatangan: req.nama_penandatangan,
                    pangkat_penandatangan: req.pangkat_penandatangan,
                    barang: req.barang,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                })
            });

        let service = PerlengkapanService::new(Arc::new(mock_repo));
        let result = service.create_pengadaan_hps(req).await.unwrap();

        assert_eq!(result.no_hps, "HPS-001");
        assert_eq!(result.pengadaan_id, pengadaan_id);
    }

    #[tokio::test]
    async fn test_create_analisis() {
        let mut mock_repo = MockRepository::new();
        let user_id = Uuid::new_v4();
        let req = CreateAnalisisRequest {
            judul: "New Analisis".to_string(),
            kategori: "TIK".to_string(),
            deskripsi: Some("Desc".to_string()),
            prioritas: "tinggi".to_string(),
            estimasi_biaya: Some(100000.0),
            justifikasi: None,
        };

        mock_repo
            .expect_create_analisis()
            .with(always(), eq(Some(user_id)))
            .times(1)
            .returning(|req, uid| {
                Ok(AnalisisKebutuhan {
                    id: Uuid::new_v4(),
                    judul: req.judul,
                    kategori: req.kategori,
                    deskripsi: req.deskripsi,
                    prioritas: req.prioritas,
                    status: "draft".to_string(),
                    estimasi_biaya: req.estimasi_biaya,
                    justifikasi: req.justifikasi,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    created_by: uid,
                    updated_by: uid,
                })
            });

        let service = PerlengkapanService::new(Arc::new(mock_repo));
        let result = service.create_analisis(req, Some(user_id)).await.unwrap();

        assert_eq!(result.judul, "New Analisis");
    }

    #[tokio::test]
    async fn test_create_pemakaian() {
        let mut mock_repo = MockRepository::new();
        let user_id = Uuid::new_v4();
        let asset_id = Uuid::new_v4();
        let req = CreatePemakaianRequest {
            asset_id,
            piminjam_nama: "John Doe".to_string(),
            tanggal_mulai: chrono::NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            tanggal_selesai: None,
            keperluan: Some("Project A".to_string()),
        };

        mock_repo
            .expect_create_pemakaian()
            .with(always(), eq(Some(user_id)))
            .times(1)
            .returning(|req, uid| {
                Ok(Pemakaian {
                    id: Uuid::new_v4(),
                    asset_id: req.asset_id,
                    piminjam_nama: req.piminjam_nama,
                    tanggal_mulai: req.tanggal_mulai,
                    tanggal_selesai: req.tanggal_selesai,
                    status: "dipinjam".to_string(),
                    keperluan: req.keperluan,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    created_by: uid,
                    updated_by: uid,
                })
            });

        let service = PerlengkapanService::new(Arc::new(mock_repo));
        let result = service.create_pemakaian(req, Some(user_id)).await.unwrap();

        assert_eq!(result.piminjam_nama, "John Doe");
        assert_eq!(result.status, "dipinjam");
    }

    #[tokio::test]
    async fn test_create_hibah() {
        let mut mock_repo = MockRepository::new();
        let user_id = Uuid::new_v4();
        let asset_id = Uuid::new_v4();
        let req = CreateHibahRequest {
            asset_id,
            pemberi: "Donor A".to_string(),
            penerima: "Satker B".to_string(),
            tanggal_hibah: chrono::NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            keterangan: Some("Hibah aset TI".to_string()),
        };

        mock_repo
            .expect_create_hibah()
            .with(always(), eq(Some(user_id)))
            .times(1)
            .returning(|req, uid| {
                Ok(Hibah {
                    id: Uuid::new_v4(),
                    asset_id: req.asset_id,
                    pemberi: req.pemberi,
                    penerima: req.penerima,
                    tanggal_hibah: req.tanggal_hibah,
                    keterangan: req.keterangan,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    created_by: uid,
                    updated_by: uid,
                })
            });

        let service = PerlengkapanService::new(Arc::new(mock_repo));
        let result = service.create_hibah(req, Some(user_id)).await.unwrap();

        assert_eq!(result.pemberi, "Donor A");
        assert_eq!(result.penerima, "Satker B");
    }

    #[tokio::test]
    async fn test_create_mutasi() {
        let mut mock_repo = MockRepository::new();
        let user_id = Uuid::new_v4();
        let asset_id = Uuid::new_v4();
        let req = CreateMutasiRequest {
            asset_id,
            asal_satker: "Satker A".to_string(),
            tujuan_satker: "Satker B".to_string(),
            penanggung_jawab: "Officer X".to_string(),
            tanggal_mutasi: chrono::NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            keterangan: Some("Mutasi rutin".to_string()),
        };

        mock_repo
            .expect_create_mutasi()
            .with(always(), eq(Some(user_id)))
            .times(1)
            .returning(|req, uid| {
                Ok(Mutasi {
                    id: Uuid::new_v4(),
                    asset_id: req.asset_id,
                    asal_satker: req.asal_satker,
                    tujuan_satker: req.tujuan_satker,
                    penanggung_jawab: req.penanggung_jawab,
                    tanggal_mutasi: req.tanggal_mutasi,
                    status: "proses".to_string(),
                    keterangan: req.keterangan,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    created_by: uid,
                    updated_by: uid,
                })
            });

        let service = PerlengkapanService::new(Arc::new(mock_repo));
        let result = service.create_mutasi(req, Some(user_id)).await.unwrap();

        assert_eq!(result.asal_satker, "Satker A");
        assert_eq!(result.tujuan_satker, "Satker B");
    }

    #[tokio::test]
    async fn test_create_penghapusan() {
        let mut mock_repo = MockRepository::new();
        let user_id = Uuid::new_v4();
        let asset_id = Uuid::new_v4();
        let req = CreatePenghapusanRequest {
            asset_id,
            tanggal_penghapusan: chrono::NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            alasan: "Rusak berat".to_string(),
            metode_penghapusan: "Lelang".to_string(),
            nilai_residu: Some(100000.0),
        };

        mock_repo
            .expect_create_penghapusan()
            .with(always(), eq(Some(user_id)))
            .times(1)
            .returning(|req, uid| {
                Ok(Penghapusan {
                    id: Uuid::new_v4(),
                    asset_id: req.asset_id,
                    tanggal_penghapusan: req.tanggal_penghapusan,
                    alasan: req.alasan,
                    metode_penghapusan: req.metode_penghapusan,
                    status: "usulan".to_string(),
                    nilai_residu: req.nilai_residu,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    created_by: uid,
                    updated_by: uid,
                })
            });

        let service = PerlengkapanService::new(Arc::new(mock_repo));
        let result = service
            .create_penghapusan(req, Some(user_id))
            .await
            .unwrap();

        assert_eq!(result.alasan, "Rusak berat");
        assert_eq!(result.status, "usulan");
    }

    #[tokio::test]
    async fn test_create_pengalihan() {
        let mut mock_repo = MockRepository::new();
        let user_id = Uuid::new_v4();
        let asset_id = Uuid::new_v4();
        let req = CreatePengalihanRequest {
            asset_id,
            pihak_lama: "Lama".to_string(),
            pihak_baru: "Baru".to_string(),
            tanggal_pengalihan: chrono::NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            dasar_pengalihan: Some("SK 123".to_string()),
            keterangan: Some("Ket".to_string()),
        };

        mock_repo
            .expect_create_pengalihan()
            .with(always(), eq(Some(user_id)))
            .times(1)
            .returning(|req, uid| {
                Ok(Pengalihan {
                    id: Uuid::new_v4(),
                    asset_id: req.asset_id,
                    pihak_lama: req.pihak_lama,
                    pihak_baru: req.pihak_baru,
                    tanggal_pengalihan: req.tanggal_pengalihan,
                    dasar_pengalihan: req.dasar_pengalihan,
                    status: "proses".to_string(),
                    keterangan: req.keterangan,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    created_by: uid,
                    updated_by: uid,
                })
            });

        let service = PerlengkapanService::new(Arc::new(mock_repo));
        let result = service.create_pengalihan(req, Some(user_id)).await.unwrap();

        assert_eq!(result.pihak_lama, "Lama");
        assert_eq!(result.status, "proses");
    }

    #[tokio::test]
    async fn test_create_pemeliharaan() {
        let mut mock_repo = MockRepository::new();
        let user_id = Uuid::new_v4();
        let asset_id = Uuid::new_v4();
        let req = CreatePemeliharaanRequest {
            asset_id,
            jenis_pemeliharaan: "Rutin".to_string(),
            biaya: Some(500000.0),
            tanggal_mulai: chrono::NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            tanggal_selesai: None,
            pelaksana: "Internal".to_string(),
            keterangan: Some("Servis AC".to_string()),
        };

        mock_repo
            .expect_create_pemeliharaan()
            .with(always(), eq(Some(user_id)))
            .times(1)
            .returning(|req, uid| {
                Ok(Pemeliharaan {
                    id: Uuid::new_v4(),
                    asset_id: req.asset_id,
                    jenis_pemeliharaan: req.jenis_pemeliharaan,
                    biaya: req.biaya,
                    tanggal_mulai: req.tanggal_mulai,
                    tanggal_selesai: req.tanggal_selesai,
                    pelaksana: req.pelaksana,
                    status: "terjadwal".to_string(),
                    keterangan: req.keterangan,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    created_by: uid,
                    updated_by: uid,
                })
            });

        let service = PerlengkapanService::new(Arc::new(mock_repo));
        let result = service
            .create_pemeliharaan(req, Some(user_id))
            .await
            .unwrap();

        assert_eq!(result.jenis_pemeliharaan, "Rutin");
        assert_eq!(result.status, "terjadwal");
    }
}

// Register handler tests
#[cfg(test)]
mod handlers_test;
