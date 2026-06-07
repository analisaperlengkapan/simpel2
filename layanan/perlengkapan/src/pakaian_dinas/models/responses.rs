use serde::{Deserialize, Serialize};
use tokio_postgres::Row;

use super::*;

// ============ Report Models ============

/// Summary report by size
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaporanRekapUkuran {
    pub pakaian_nama: String,
    pub ukuran_group: String,
    pub ukuran: String,
    pub jumlah_laki: i64,
    pub jumlah_perempuan: i64,
    pub jumlah_total: i64,
}
impl LaporanRekapUkuran {
    pub fn from_row(row: &Row) -> Self {
        Self {
            pakaian_nama: row.get("pakaian_nama"),
            ukuran_group: row.get("ukuran_group"),
            ukuran: row.get("ukuran"),
            jumlah_laki: row.get("jumlah_laki"),
            jumlah_perempuan: row.get("jumlah_perempuan"),
            jumlah_total: row.get("jumlah_total"),
        }
    }
}
/// Detail list report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaporanDaftarPegawai {
    pub nip: String,
    pub nama: String,
    pub satker_nama: String,
    pub jabatan: Option<String>,
    pub pangkat: Option<String>,
    pub jenis_kelamin: String,
    pub gol_kd: Option<String>,
    pub jenis: Option<String>,
    pub eselon: Option<String>,
    pub ukuran_baju: Option<String>,
    pub ukuran_celana: Option<String>,
    pub ukuran_sepatu: Option<String>,
    pub with_hijab: bool,
}
impl LaporanDaftarPegawai {
    pub fn from_row(row: &Row) -> Self {
        Self {
            nip: row.get("nip"),
            nama: row.get("nama"),
            satker_nama: row.get("satker_nama"),
            jabatan: row.try_get("jabatan").ok(),
            pangkat: row.try_get("pangkat").ok(),
            jenis_kelamin: row.get("jenis_kelamin"),
            gol_kd: row.try_get("gol_kd").ok(),
            jenis: row.try_get("jenis").ok(),
            eselon: row.try_get("eselon").ok(),
            ukuran_baju: row.try_get("ukuran_baju").ok(),
            ukuran_celana: row.try_get("ukuran_celana").ok(),
            ukuran_sepatu: row.try_get("ukuran_sepatu").ok(),
            with_hijab: row.try_get("with_hijab").unwrap_or(false),
        }
    }
}
/// Info kesegaran sinkronisasi MySIMKARI (Fase 2.4) untuk banner wizard ukuran.
#[derive(Debug, Clone, Serialize)]
pub struct PegawaiSyncInfo {
    pub sumber: String,
    /// Nama state sinkronisasi (SYNC_STATE_COMPLETED / FAILED / RUNNING / ...).
    pub state: String,
    pub last_sync_at: Option<String>,
    /// `true` jika sync terakhir COMPLETED dan ada timestamp.
    pub segar: bool,
    pub records_synced: i64,
    pub error_message: Option<String>,
}
/// Roster pegawai satker + info kesegaran sinkronisasi (Fase 2.4).
#[derive(Debug, Clone, Serialize)]
pub struct PegawaiRosterWithSync {
    pub pegawai: Vec<MysimkariPegawai>,
    pub total: i64,
    /// `None` bila integrasi tidak tersedia / probe gagal (tidak diketahui).
    pub sync: Option<PegawaiSyncInfo>,
}
