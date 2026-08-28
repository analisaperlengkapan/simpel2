use super::KebutuhanBmnService;
use crate::kebutuhan_bmn::models::*;
use crate::kebutuhan_bmn::repository::KebutuhanBmnRepository;
use crate::shared::error::{AppError, AppResult};
use crate::shared::grpc::clients::IntegrasiClient;
use crate::shared::grpc::clients::integrasi::v1::{DataSource, SyncState};
use crate::shared::satker_scope::SatkerScope;
use std::collections::HashMap;
use tracing::warn;
use uuid::Uuid;

impl KebutuhanBmnService {
    /// Get feasibility analysis for a satker
    /// Compares requested goods against existing SIMAN inventory
    pub async fn get_analisis_kelayakan(
        &self,
        satker_id: Uuid,
        scope: &SatkerScope,
    ) -> AppResult<AnalisisKelayakanResponse> {
        // This is the whole of another satker's feasibility case: what they
        // asked for, what SIMAN says they already hold, and the gap between.
        // The laporan preview/download endpoints render exactly this into a
        // PDF, which is how an operator downloaded another satker's report.
        let satker = self.repository.get_satker_by_id(satker_id, scope).await?;
        let (barang_list, _) = self
            .repository
            .get_satker_barang(satker_id, 1, 1000, None)
            .await?;

        // V029 (#24): jika satker sudah melewati submit operator, baca snapshot
        // beku (di-freeze saat submit) alih-alih fetch SIMAN live — sehingga
        // Validator Wilayah & Pusat melihat angka yg PERSIS sama dgn operator.
        let snapshot = if satker.status.is_post_operator_submit() {
            self.repository
                .get_analisis_snapshot(satker_id)
                .await
                .ok()
                .flatten()
                .and_then(|v| serde_json::from_value::<AnalisisSnapshot>(v).ok())
        } else {
            None
        };

        let (barang_with_inventory, summary, is_snapshot, snapshot_at) = match snapshot {
            Some(snap) => {
                let at = snap.snapshot_at.clone();
                let (b, s) = Self::build_analisis_from_snapshot(barang_list, &snap);
                (b, s, true, Some(at))
            }
            None => {
                let (b, s, _) = self
                    .compute_live_analisis(barang_list, &satker.satker_id)
                    .await;
                (b, s, false, None)
            }
        };

        // Fetch pegawai data from MySIMKARI for final analysis by Validator Pusat
        let data_pegawai = self
            .get_mysimkari_pegawai_data(&satker.satker_id)
            .await
            .ok();

        let integrasi_sync = self.get_integrasi_sync_metadata().await;

        Ok(AnalisisKelayakanResponse {
            satker,
            barang_list: barang_with_inventory,
            summary,
            data_pegawai,
            integrasi_sync,
            is_snapshot,
            snapshot_at,
        })
    }

    /// V029 (#24): hitung analisis kelayakan LIVE dari SIMAN per barang.
    /// Mengembalikan (barang+inventory, summary, snapshot-beku siap simpan).
    /// Dipakai saat operator masih mengedit (live) dan saat submit (untuk
    /// membekukan snapshot).
    pub(crate) async fn compute_live_analisis(
        &self,
        barang_list: Vec<PengajuanKebutuhanBmnBarang>,
        // MySIMKARI `kode_satker` of the satker this analysis belongs to. The
        // comparison is "what does THIS satker already hold", so it cannot be
        // derived from the barang rows — it has to be handed in.
        satker_code: &str,
    ) -> (
        Vec<BarangWithExistingInventory>,
        AnalisisSummary,
        AnalisisSnapshot,
    ) {
        let mut total_diminta: i64 = 0;
        let mut total_existing: i64 = 0;
        let mut barang_with_inventory = Vec::new();
        let mut snap_barang = Vec::new();

        for barang in barang_list {
            total_diminta += barang.jumlah as i64;

            // What this satker already holds under the same barang code, read
            // from `integrasi.siman_aset` — the live source of truth the rest
            // of the system reads.
            //
            // This branch used to hang off an Option<SimanIntegration> that was
            // never constructed anywhere in the application, so it always fell
            // through to the stored `existing_count` column and an empty asset
            // list. The screen said "dibandingkan dengan SIMAN" and nothing had
            // been compared.
            //
            // A SIMAN read that fails must not fail the whole analysis: fall
            // back to the stored column and say so in the log, rather than
            // turning a degraded comparison into a 500 on a page the operator
            // needs to keep working.
            let kode = barang.kode_barang.clone().unwrap_or_default();
            let (existing_count, existing_assets) = match self
                .repository
                .count_siman_assets_for(satker_code, &kode, 20)
                .await
            {
                Ok(found) => found,
                Err(e) => {
                    warn!(
                        "SIMAN lookup failed for satker {} barang {}: {}. \
                         Falling back to the stored existing_count.",
                        satker_code, kode, e
                    );
                    (barang.existing_count, vec![])
                }
            };

            total_existing += existing_count as i64;

            let gap = barang.jumlah - existing_count;
            let recommendation = self.generate_recommendation(gap, barang.jumlah);

            snap_barang.push(AnalisisSnapshotBarang {
                barang_id: barang.id,
                existing_count,
                gap,
                recommendation: recommendation.clone(),
                existing_assets: existing_assets
                    .iter()
                    .map(|a| AnalisisSnapshotAsset {
                        no_aset: a.no_aset.clone(),
                        nama_aset: a.nama_aset.clone(),
                        kondisi: a.kondisi.clone(),
                        lokasi: a.lokasi.clone(),
                    })
                    .collect(),
            });

            // The response flattens the barang row, so `existing_count` on the
            // wire is the row's STORED column — not the number the gap beside
            // it was computed from. Left alone the payload contradicts itself:
            // "existing_count 0, gap 3", with two matching assets listed under
            // it. Carry the computed value onto the row that gets serialised.
            let mut barang = barang;
            barang.existing_count = existing_count;

            barang_with_inventory.push(BarangWithExistingInventory {
                barang,
                existing_assets,
                gap,
                recommendation,
            });
        }

        let total_gap = total_diminta - total_existing;
        let kelayakan_persen = if total_diminta > 0 {
            (total_existing as f64 / total_diminta as f64) * 100.0
        } else {
            100.0
        };

        let summary = AnalisisSummary {
            total_diminta,
            total_existing,
            total_gap,
            kelayakan_persen,
        };
        let snapshot = AnalisisSnapshot {
            snapshot_at: chrono::Utc::now().to_rfc3339(),
            summary: AnalisisSnapshotSummary {
                total_diminta,
                total_existing,
                total_gap,
                kelayakan_persen,
            },
            barang: snap_barang,
        };

        (barang_with_inventory, summary, snapshot)
    }

    /// V029 (#24): rekonstruksi analisis dari snapshot beku — fungsi MURNI
    /// (testable tanpa SIMAN/DB). Baris barang tetap dari DB (beku setelah
    /// submit), tetapi `existing_count`/`gap`/`recommendation`/`existing_assets`
    /// diambil dari snapshot agar konsisten dgn yg dilihat operator. Barang
    /// yg tak ada di snapshot (mis. ditambahkan setelah submit saat revisi)
    /// jatuh ke `existing_count` tersimpan sebagai fallback.
    pub(crate) fn build_analisis_from_snapshot(
        barang_list: Vec<PengajuanKebutuhanBmnBarang>,
        snapshot: &AnalisisSnapshot,
    ) -> (Vec<BarangWithExistingInventory>, AnalisisSummary) {
        let mut out = Vec::new();
        for barang in barang_list {
            match snapshot.barang.iter().find(|b| b.barang_id == barang.id) {
                Some(snap) => {
                    let existing_assets = snap
                        .existing_assets
                        .iter()
                        .map(|a| ExistingAssetInfo {
                            no_aset: a.no_aset.clone(),
                            nama_aset: a.nama_aset.clone(),
                            kondisi: a.kondisi.clone(),
                            lokasi: a.lokasi.clone(),
                        })
                        .collect();
                    out.push(BarangWithExistingInventory {
                        existing_assets,
                        gap: snap.gap,
                        recommendation: snap.recommendation.clone(),
                        barang,
                    });
                }
                None => {
                    let gap = barang.jumlah - barang.existing_count;
                    out.push(BarangWithExistingInventory {
                        existing_assets: vec![],
                        gap,
                        recommendation: String::new(),
                        barang,
                    });
                }
            }
        }

        let summary = AnalisisSummary {
            total_diminta: snapshot.summary.total_diminta,
            total_existing: snapshot.summary.total_existing,
            total_gap: snapshot.summary.total_gap,
            kelayakan_persen: snapshot.summary.kelayakan_persen,
        };
        (out, summary)
    }

    /// Fetch MySIMKARI pegawai data for satker analysis
    async fn get_mysimkari_pegawai_data(&self, satker_id: &str) -> AppResult<DataPegawaiRekap> {
        let integrasi_client = match &self.integrasi_client {
            Some(client) => client,
            None => {
                warn!(
                    "Integrasi client not configured, returning empty MySIMKARI rekap for satker {}",
                    satker_id
                );
                return Ok(DataPegawaiRekap {
                    total_pegawai: 0,
                    rekap_eselon: vec![],
                    rekap_non_eselon: vec![],
                });
            }
        };

        let mut current_page = 1;
        let per_page = 200;
        let mut all_items = Vec::new();
        let mut total_pegawai = 0_i64;

        loop {
            let response = integrasi_client
                .get_mysimkari_pegawai(satker_id, current_page, per_page)
                .await
                .map_err(|e| {
                    AppError::Internal(format!(
                        "Failed to fetch MySIMKARI pegawai data for satker {}: {}",
                        satker_id, e
                    ))
                })?;

            if total_pegawai == 0 {
                total_pegawai = response
                    .pagination
                    .as_ref()
                    .map(|p| p.total_items)
                    .unwrap_or(response.items.len() as i64);
            }

            let total_pages = response
                .pagination
                .as_ref()
                .map(|p| p.total_pages)
                .unwrap_or(current_page);

            if response.items.is_empty() {
                break;
            }

            all_items.extend(response.items);

            if current_page >= total_pages {
                break;
            }
            current_page += 1;
        }

        let mut eselon_counts: HashMap<String, i64> = HashMap::new();
        let mut non_eselon_counts: HashMap<(String, String, bool), i64> = HashMap::new();

        for pegawai in all_items {
            if let Some(tingkat_eselon) =
                Self::extract_eselon_level(&pegawai.jabatan, &pegawai.extra_fields)
            {
                *eselon_counts.entry(tingkat_eselon).or_insert(0) += 1;
                continue;
            }

            let golongan = if pegawai.golongan.trim().is_empty() {
                "Tidak Diketahui".to_string()
            } else {
                pegawai.golongan.clone()
            };

            let pangkat = if pegawai.pangkat.trim().is_empty() {
                "Tidak Diketahui".to_string()
            } else {
                pegawai.pangkat.clone()
            };

            let is_jaksa = pegawai.jabatan.to_lowercase().contains("jaksa");
            *non_eselon_counts
                .entry((golongan, pangkat, is_jaksa))
                .or_insert(0) += 1;
        }

        let mut rekap_eselon: Vec<RekapEselonItem> = eselon_counts
            .into_iter()
            .map(|(tingkat_eselon, jumlah)| RekapEselonItem {
                tingkat_eselon,
                jumlah,
            })
            .collect();
        rekap_eselon.sort_by(|a, b| a.tingkat_eselon.cmp(&b.tingkat_eselon));

        let mut rekap_non_eselon: Vec<RekapNonEselonItem> = non_eselon_counts
            .into_iter()
            .map(
                |((golongan, pangkat, is_jaksa), jumlah)| RekapNonEselonItem {
                    golongan,
                    pangkat,
                    is_jaksa,
                    jumlah,
                },
            )
            .collect();
        rekap_non_eselon.sort_by(|a, b| {
            a.golongan
                .cmp(&b.golongan)
                .then(a.pangkat.cmp(&b.pangkat))
                .then(a.is_jaksa.cmp(&b.is_jaksa))
        });

        Ok(DataPegawaiRekap {
            total_pegawai,
            rekap_eselon,
            rekap_non_eselon,
        })
    }

    fn extract_eselon_level(
        jabatan: &str,
        extra_fields: &HashMap<String, String>,
    ) -> Option<String> {
        let from_extra = extra_fields
            .get("tingkat_eselon")
            .or_else(|| extra_fields.get("eselon"))
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        if from_extra.is_some() {
            return from_extra;
        }

        let jabatan_lc = jabatan.to_lowercase();
        let patterns = [
            ("eselon i", "Eselon I"),
            ("eselon ii", "Eselon II"),
            ("eselon iii", "Eselon III"),
            ("eselon iv", "Eselon IV"),
            ("eselon v", "Eselon V"),
        ];

        for (needle, label) in patterns {
            if jabatan_lc.contains(needle) {
                return Some(label.to_string());
            }
        }

        None
    }

    /// Hard ceiling for the whole sync-status probe.
    ///
    /// `fetch_sync_status` already treats an unreachable integrasi as "unknown"
    /// rather than "risky" — the probe is explicitly NOT allowed to block a
    /// decision. But not blocking on the RESULT is not the same as not blocking
    /// in TIME, and the two probes used to run one after the other with no
    /// ceiling of their own. When integrasi was unreachable (a NetworkPolicy
    /// opened port 50053 while the service listens on 50051), that cost 25s then
    /// 30s, serially, on `keputusan-pusat`:
    ///
    /// ```text
    /// 07:34:11.717  request masuk
    /// 07:34:36.900  mysimkari menyerah        (+25,2s)
    /// 07:35:06.935  siman menyerah            (+30,0s)
    /// 07:35:06.960  SELURUH kerja DB selesai  (+0,025s)
    /// ```
    ///
    /// The gateway cuts clients off at 30s, so every Validator Pusat approval
    /// returned 504 while the transition itself succeeded ~20s later via the
    /// cancel-safe middleware. The netpol is fixed, but nothing stopped this
    /// from recurring the next time integrasi is slow or down.
    ///
    /// 3s is far below the gateway's 30s and far above a healthy round trip
    /// (measured 0,02s in-cluster once the policy was correct).
    const SYNC_PROBE_BUDGET: std::time::Duration = std::time::Duration::from_secs(3);

    pub(crate) async fn get_integrasi_sync_metadata(&self) -> Option<IntegrasiSyncMetadata> {
        let integrasi_client = self.integrasi_client.as_ref()?;

        // Concurrent, so the ceiling covers both probes rather than each one,
        // and a healthy pair costs one round trip instead of two.
        let probes = async {
            tokio::join!(
                Self::fetch_sync_status(integrasi_client, DataSource::Mysimkari, "mysimkari"),
                Self::fetch_sync_status(integrasi_client, DataSource::Siman, "siman"),
            )
        };

        let (mysimkari, siman) = match tokio::time::timeout(Self::SYNC_PROBE_BUDGET, probes).await {
            Ok(pair) => pair,
            Err(_) => {
                warn!(
                    "Integrasi sync-status probe exceeded {:?}; treating both sources as unknown \
                     rather than holding the caller's request open",
                    Self::SYNC_PROBE_BUDGET
                );
                (
                    Self::unknown_sync_status("mysimkari"),
                    Self::unknown_sync_status("siman"),
                )
            }
        };

        Some(IntegrasiSyncMetadata { mysimkari, siman })
    }

    /// "We could not find out", which `is_integrasi_sync_risky` reads as
    /// not-risky — the same value the per-probe error path already produces.
    fn unknown_sync_status(source_name: &str) -> IntegrasiSyncStatus {
        IntegrasiSyncStatus {
            source: source_name.to_string(),
            state: "SYNC_STATE_UNSPECIFIED".to_string(),
            last_sync_at: None,
            next_sync_at: None,
            records_synced: 0,
            error_message: None,
        }
    }

    async fn fetch_sync_status(
        integrasi_client: &IntegrasiClient,
        source: DataSource,
        source_name: &str,
    ) -> IntegrasiSyncStatus {
        match integrasi_client.get_sync_status(source).await {
            Ok(status) => {
                let state = SyncState::try_from(status.state)
                    .map(|s| s.as_str_name().to_string())
                    .unwrap_or_else(|_| "SYNC_STATE_UNSPECIFIED".to_string());

                IntegrasiSyncStatus {
                    source: source_name.to_string(),
                    state,
                    last_sync_at: if status.last_sync_at.trim().is_empty() {
                        None
                    } else {
                        Some(status.last_sync_at)
                    },
                    next_sync_at: if status.next_sync_at.trim().is_empty() {
                        None
                    } else {
                        Some(status.next_sync_at)
                    },
                    records_synced: status.records_synced,
                    error_message: if status.error_message.trim().is_empty() {
                        None
                    } else {
                        Some(status.error_message)
                    },
                }
            }
            Err(e) => {
                // NOTE: A failed gRPC probe (network timeout, service restart, DNS)
                // must NOT block Validator Pusat business decisions. We only set
                // error_message when the sync *itself* reports a failure (handled
                // in the Ok branch above). Probe failures are logged but treated
                // as "unknown" rather than "risky".
                warn!(
                    "Failed to fetch sync status for {} from Integrasi (probe failure, not blocking decisions): {}",
                    source_name, e
                );
                IntegrasiSyncStatus {
                    source: source_name.to_string(),
                    state: "SYNC_STATE_UNSPECIFIED".to_string(),
                    last_sync_at: None,
                    next_sync_at: None,
                    records_synced: 0,
                    error_message: None,
                }
            }
        }
    }

    /// Generate recommendation based on gap analysis
    fn generate_recommendation(&self, gap: i32, jumlah_diminta: i32) -> String {
        if gap <= 0 {
            "Tidak perlu pengadaan - stok mencukupi".to_string()
        } else if gap < jumlah_diminta / 2 {
            format!("Perlu pengadaan {} unit (sebagian)", gap)
        } else if gap == jumlah_diminta {
            format!("Perlu pengadaan {} unit (tidak ada stok)", gap)
        } else {
            format!("Perlu pengadaan {} unit (prioritas tinggi)", gap)
        }
    }
}
