#[cfg(test)]
mod tests {
    use crate::tests::MockRepository; // Accessing MockRepository from parent tests module
    use crate::{handlers::*, middleware::Claims, models::*, services::PerlengkapanService};
    use axum::extract::{Path, Query, State};
    use axum::Json;
    use chrono::Utc;
    use std::sync::Arc;
    use uuid::Uuid;

    fn create_mock_claims() -> Claims {
        Claims {
            user_id: Uuid::new_v4(),
            username: "testuser".to_string(),
            role: "admin".to_string(),
            permissions: vec![],
        }
    }

    #[tokio::test]
    async fn test_handler_get_all_assets() {
        let mut mock_repo = MockRepository::new();

        mock_repo
            .expect_get_all_assets()
            .with(
                mockall::predicate::eq(1),
                mockall::predicate::eq(20),
                mockall::predicate::eq(None),
            )
            .times(1)
            .returning(|_, _, _| Ok((vec![], 0)));

        let service = PerlengkapanService::new(Arc::new(mock_repo));
        let state = State(service);
        let pagination = Query(PaginationQuery {
            page: 1,
            per_page: 20,
            category: None,
        });
        let claims = create_mock_claims();

        let result = get_all_assets(state, pagination, claims).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        // Check response content if needed (response.0 is the Json wrapper)
        assert_eq!(response.0.page, 1);
        assert_eq!(response.0.data.len(), 0);
    }

    #[tokio::test]
    async fn test_handler_get_asset_by_id_found() {
        let mut mock_repo = MockRepository::new();
        let id = Uuid::new_v4();

        mock_repo
            .expect_get_asset_by_id()
            .with(mockall::predicate::eq(id))
            .times(1)
            .returning(move |_| {
                Ok(Asset {
                    id,
                    kategori_aset: "Tanah".to_string(),
                    no_aset: "1".to_string(),
                    nama_aset: Some("Found Asset".to_string()),
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
        let state = State(service);
        let claims = create_mock_claims();

        let result = get_asset_by_id(state, Path(id), claims).await;

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().0.data.nama_aset,
            Some("Found Asset".to_string())
        );
    }

    #[tokio::test]
    async fn test_handler_get_all_pengadaan() {
        let mut mock_repo = MockRepository::new();

        mock_repo
            .expect_get_all_pengadaan()
            .with(mockall::predicate::eq(1), mockall::predicate::eq(20))
            .times(1)
            .returning(|_, _| {
                Ok((
                    vec![Pengadaan {
                        id: Uuid::new_v4(),
                        judul: "Pengadaan Laptop".to_string(),
                        deskripsi: None,
                        jenis: "TIK".to_string(),
                        status: "perencanaan".to_string(),
                        anggaran: Some(10000000.0),
                        target_selesai: None,
                        pic_user_id: None,
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                        created_by: None,
                        updated_by: None,
                    }],
                    1,
                ))
            });

        let service = PerlengkapanService::new(Arc::new(mock_repo));
        let state = State(service);
        let pagination = Query(PaginationQuery {
            page: 1,
            per_page: 20,
            category: None,
        });
        let claims = create_mock_claims();

        let result = get_all_pengadaan(state, pagination, claims).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.0.data.len(), 1);
        assert_eq!(response.0.data[0].judul, "Pengadaan Laptop");
    }

    #[tokio::test]
    async fn test_handler_get_all_analisis() {
        let mut mock_repo = MockRepository::new();

        mock_repo
            .expect_get_all_analisis()
            .with(mockall::predicate::eq(1), mockall::predicate::eq(20))
            .times(1)
            .returning(|_, _| {
                Ok((
                    vec![AnalisisKebutuhan {
                        id: Uuid::new_v4(),
                        judul: "Analisis Server".to_string(),
                        kategori: "TIK".to_string(),
                        deskripsi: None,
                        prioritas: "tinggi".to_string(),
                        status: "draft".to_string(),
                        estimasi_biaya: Some(50000000.0),
                        justifikasi: None,
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                        created_by: None,
                        updated_by: None,
                    }],
                    1,
                ))
            });

        let service = PerlengkapanService::new(Arc::new(mock_repo));
        let state = State(service);
        let pagination = Query(PaginationQuery {
            page: 1,
            per_page: 20,
            category: None,
        });
        let claims = create_mock_claims();

        let result = get_all_analisis(state, pagination, claims).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.0.data.len(), 1);
        assert_eq!(response.0.data[0].judul, "Analisis Server");
    }

    #[tokio::test]
    async fn test_handler_create_pengadaan() {
        let mut mock_repo = MockRepository::new();
        let user_id = Uuid::new_v4();

        mock_repo.expect_create_pengadaan()
            .with(mockall::predicate::always(), mockall::predicate::always())
            .times(1)
            .returning(move |req, uid| Ok(Pengadaan {
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
            }));

        let service = PerlengkapanService::new(Arc::new(mock_repo));
        let state = State(service);
        let mut claims = create_mock_claims();
        claims.user_id = user_id;

        let request = Json(CreatePengadaanRequest {
            judul: "Handler Test".to_string(),
            deskripsi: None,
            jenis: "TIK".to_string(),
            anggaran: None,
            target_selesai: None,
            pic_user_id: None,
        });

        let result = create_pengadaan(state, claims, request).await;

        assert!(result.is_ok());
        let (status, json) = result.unwrap();
        assert_eq!(status, axum::http::StatusCode::CREATED);
        assert_eq!(json.0.data.judul, "Handler Test");
    }

    #[tokio::test]
    async fn test_handler_create_analisis() {
        let mut mock_repo = MockRepository::new();
        let user_id = Uuid::new_v4();

        mock_repo.expect_create_analisis()
             .with(mockall::predicate::always(), mockall::predicate::always())
            .times(1)
            .returning(move |req, uid| Ok(AnalisisKebutuhan {
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
            }));

        let service = PerlengkapanService::new(Arc::new(mock_repo));
        let state = State(service);
        let mut claims = create_mock_claims();
        claims.user_id = user_id;

        let request = Json(CreateAnalisisRequest {
            judul: "Handler Analisis".to_string(),
            kategori: "UMUM".to_string(),
            deskripsi: None,
            prioritas: "sedang".to_string(),
            estimasi_biaya: None,
            justifikasi: None,
        });

        let result = create_analisis(state, claims, request).await;

        assert!(result.is_ok());
         let (status, json) = result.unwrap();
        assert_eq!(status, axum::http::StatusCode::CREATED);
        assert_eq!(json.0.data.judul, "Handler Analisis");
    }

    #[tokio::test]
    async fn test_handler_create_pemakaian() {
        let mut mock_repo = MockRepository::new();
        let user_id = Uuid::new_v4();
        let asset_id = Uuid::new_v4();

        // Expect get_asset_by_id to be called for validation
        mock_repo.expect_get_asset_by_id()
            .with(mockall::predicate::always())
            .times(1)
            .returning(move |_| Ok(Asset {
                id: asset_id,
                kategori_aset: "Tanah".to_string(),
                no_aset: "1".to_string(),
                nama_aset: Some("Found Asset".to_string()),
                kode_barang: Some("101".to_string()),
                merk: None,
                tipe: None,
                kondisi: Some("Baik".to_string()),
                lokasi: Some("Jakarta".to_string()),
                satker: Some("Pusat".to_string()),
                nilai_perolehan: Some(1000000.0),
                tgl_perolehan: Some("2023-01-01".to_string()),
                updated_at: Utc::now(),
            }));

        mock_repo.expect_create_pemakaian()
             .with(mockall::predicate::always(), mockall::predicate::always())
            .times(1)
            .returning(move |req, uid| Ok(Pemakaian {
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
            }));

        let service = PerlengkapanService::new(Arc::new(mock_repo));
        let state = State(service);
        let mut claims = create_mock_claims();
        claims.user_id = user_id;

        let request = Json(CreatePemakaianRequest {
            asset_id,
            piminjam_nama: "Jane Doe".to_string(),
            tanggal_mulai: chrono::NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            tanggal_selesai: None,
            keperluan: None,
        });

        let result = create_pemakaian(state, claims, request).await;

        assert!(result.is_ok());
         let (status, json) = result.unwrap();
        assert_eq!(status, axum::http::StatusCode::CREATED);
        assert_eq!(json.0.data.piminjam_nama, "Jane Doe");
    }

    #[tokio::test]
    async fn test_handler_create_hibah() {
        let mut mock_repo = MockRepository::new();
        let user_id = Uuid::new_v4();
        let asset_id = Uuid::new_v4();

        // Expect get_asset_by_id to be called for validation
        mock_repo.expect_get_asset_by_id()
            .with(mockall::predicate::always())
            .times(1)
            .returning(move |_| Ok(Asset {
                id: asset_id,
                kategori_aset: "Tanah".to_string(),
                no_aset: "1".to_string(),
                nama_aset: Some("Found Asset".to_string()),
                kode_barang: Some("101".to_string()),
                merk: None,
                tipe: None,
                kondisi: Some("Baik".to_string()),
                lokasi: Some("Jakarta".to_string()),
                satker: Some("Pusat".to_string()),
                nilai_perolehan: Some(1000000.0),
                tgl_perolehan: Some("2023-01-01".to_string()),
                updated_at: Utc::now(),
            }));

        mock_repo.expect_create_hibah()
             .with(mockall::predicate::always(), mockall::predicate::always())
            .times(1)
            .returning(move |req, uid| Ok(Hibah {
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
            }));

        let service = PerlengkapanService::new(Arc::new(mock_repo));
        let state = State(service);
        let mut claims = create_mock_claims();
        claims.user_id = user_id;

        let request = Json(CreateHibahRequest {
            asset_id,
            pemberi: "Donor Handler".to_string(),
            penerima: "Receiver Handler".to_string(),
            tanggal_hibah: chrono::NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            keterangan: None,
        });

        let result = create_hibah(state, claims, request).await;

        assert!(result.is_ok());
         let (status, json) = result.unwrap();
        assert_eq!(status, axum::http::StatusCode::CREATED);
        assert_eq!(json.0.data.pemberi, "Donor Handler");
    }

    #[tokio::test]
    async fn test_handler_create_mutasi() {
        let mut mock_repo = MockRepository::new();
        let user_id = Uuid::new_v4();
        let asset_id = Uuid::new_v4();

        // Expect get_asset_by_id to be called for validation
        mock_repo.expect_get_asset_by_id()
            .with(mockall::predicate::always())
            .times(1)
            .returning(move |_| Ok(Asset {
                id: asset_id,
                kategori_aset: "Tanah".to_string(),
                no_aset: "1".to_string(),
                nama_aset: Some("Found Asset".to_string()),
                kode_barang: Some("101".to_string()),
                merk: None,
                tipe: None,
                kondisi: Some("Baik".to_string()),
                lokasi: Some("Jakarta".to_string()),
                satker: Some("Pusat".to_string()),
                nilai_perolehan: Some(1000000.0),
                tgl_perolehan: Some("2023-01-01".to_string()),
                updated_at: Utc::now(),
            }));

        mock_repo.expect_create_mutasi()
             .with(mockall::predicate::always(), mockall::predicate::always())
            .times(1)
            .returning(move |req, uid| Ok(Mutasi {
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
            }));

        let service = PerlengkapanService::new(Arc::new(mock_repo));
        let state = State(service);
        let mut claims = create_mock_claims();
        claims.user_id = user_id;

        let request = Json(CreateMutasiRequest {
            asset_id,
            asal_satker: "Satker Handler A".to_string(),
            tujuan_satker: "Satker Handler B".to_string(),
            penanggung_jawab: "Handler Officer".to_string(),
            tanggal_mutasi: chrono::NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            keterangan: None,
        });

        let result = create_mutasi(state, claims, request).await;

        assert!(result.is_ok());
         let (status, json) = result.unwrap();
        assert_eq!(status, axum::http::StatusCode::CREATED);
        assert_eq!(json.0.data.asal_satker, "Satker Handler A");
    }

    #[tokio::test]
    async fn test_handler_create_penghapusan() {
        let mut mock_repo = MockRepository::new();
        let user_id = Uuid::new_v4();
        let asset_id = Uuid::new_v4();

        // Expect get_asset_by_id to be called for validation
        mock_repo.expect_get_asset_by_id()
            .with(mockall::predicate::always())
            .times(1)
            .returning(move |_| Ok(Asset {
                id: asset_id,
                kategori_aset: "Tanah".to_string(),
                no_aset: "1".to_string(),
                nama_aset: Some("Found Asset".to_string()),
                kode_barang: Some("101".to_string()),
                merk: None,
                tipe: None,
                kondisi: Some("Baik".to_string()),
                lokasi: Some("Jakarta".to_string()),
                satker: Some("Pusat".to_string()),
                nilai_perolehan: Some(1000000.0),
                tgl_perolehan: Some("2023-01-01".to_string()),
                updated_at: Utc::now(),
            }));

        mock_repo.expect_create_penghapusan()
             .with(mockall::predicate::always(), mockall::predicate::always())
            .times(1)
            .returning(move |req, uid| Ok(Penghapusan {
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
            }));

        let service = PerlengkapanService::new(Arc::new(mock_repo));
        let state = State(service);
        let mut claims = create_mock_claims();
        claims.user_id = user_id;

        let request = Json(CreatePenghapusanRequest {
            asset_id,
            tanggal_penghapusan: chrono::NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            alasan: "Handler Disposal".to_string(),
            metode_penghapusan: "Musnah".to_string(),
            nilai_residu: None,
        });

        let result = create_penghapusan(state, claims, request).await;

        assert!(result.is_ok());
         let (status, json) = result.unwrap();
        assert_eq!(status, axum::http::StatusCode::CREATED);
        assert_eq!(json.0.data.alasan, "Handler Disposal");
    }

    #[tokio::test]
    async fn test_handler_create_pengalihan() {
        let mut mock_repo = MockRepository::new();
        let user_id = Uuid::new_v4();
        let asset_id = Uuid::new_v4();

        // Expect get_asset_by_id to be called for validation
        mock_repo.expect_get_asset_by_id()
            .with(mockall::predicate::always())
            .times(1)
            .returning(move |_| Ok(Asset {
                id: asset_id,
                kategori_aset: "Tanah".to_string(),
                no_aset: "1".to_string(),
                nama_aset: Some("Found Asset".to_string()),
                kode_barang: Some("101".to_string()),
                merk: None,
                tipe: None,
                kondisi: Some("Baik".to_string()),
                lokasi: Some("Jakarta".to_string()),
                satker: Some("Pusat".to_string()),
                nilai_perolehan: Some(1000000.0),
                tgl_perolehan: Some("2023-01-01".to_string()),
                updated_at: Utc::now(),
            }));

        mock_repo.expect_create_pengalihan()
             .with(mockall::predicate::always(), mockall::predicate::always())
            .times(1)
            .returning(move |req, uid| Ok(Pengalihan {
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
            }));

        let service = PerlengkapanService::new(Arc::new(mock_repo));
        let state = State(service);
        let mut claims = create_mock_claims();
        claims.user_id = user_id;

        let request = Json(CreatePengalihanRequest {
            asset_id,
            pihak_lama: "Handler Lama".to_string(),
            pihak_baru: "Handler Baru".to_string(),
            tanggal_pengalihan: chrono::NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            dasar_pengalihan: None,
            keterangan: None,
        });

        let result = create_pengalihan(state, claims, request).await;

        assert!(result.is_ok());
         let (status, json) = result.unwrap();
        assert_eq!(status, axum::http::StatusCode::CREATED);
        assert_eq!(json.0.data.pihak_lama, "Handler Lama");
    }

    #[tokio::test]
    async fn test_handler_create_pemeliharaan() {
        let mut mock_repo = MockRepository::new();
        let user_id = Uuid::new_v4();
        let asset_id = Uuid::new_v4();

        // Expect get_asset_by_id to be called for validation
        mock_repo.expect_get_asset_by_id()
            .with(mockall::predicate::always())
            .times(1)
            .returning(move |_| Ok(Asset {
                id: asset_id,
                kategori_aset: "Tanah".to_string(),
                no_aset: "1".to_string(),
                nama_aset: Some("Found Asset".to_string()),
                kode_barang: Some("101".to_string()),
                merk: None,
                tipe: None,
                kondisi: Some("Baik".to_string()),
                lokasi: Some("Jakarta".to_string()),
                satker: Some("Pusat".to_string()),
                nilai_perolehan: Some(1000000.0),
                tgl_perolehan: Some("2023-01-01".to_string()),
                updated_at: Utc::now(),
            }));

        mock_repo.expect_create_pemeliharaan()
             .with(mockall::predicate::always(), mockall::predicate::always())
            .times(1)
            .returning(move |req, uid| Ok(Pemeliharaan {
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
            }));

        let service = PerlengkapanService::new(Arc::new(mock_repo));
        let state = State(service);
        let mut claims = create_mock_claims();
        claims.user_id = user_id;

        let request = Json(CreatePemeliharaanRequest {
            asset_id,
            jenis_pemeliharaan: "Handler Maintain".to_string(),
            biaya: None,
            tanggal_mulai: chrono::NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            tanggal_selesai: None,
            pelaksana: "Handler Tech".to_string(),
            keterangan: None,
        });

        let result = create_pemeliharaan(state, claims, request).await;

        assert!(result.is_ok());
         let (status, json) = result.unwrap();
        assert_eq!(status, axum::http::StatusCode::CREATED);
        assert_eq!(json.0.data.jenis_pemeliharaan, "Handler Maintain");
    }
}
