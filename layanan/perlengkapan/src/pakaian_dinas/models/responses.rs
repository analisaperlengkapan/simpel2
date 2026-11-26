use serde::{Deserialize, Serialize};
use tokio_postgres::Row;
use uuid::Uuid;

use super::*;

// ============ Bulk action ============

/// What came of approving several satker at once. A partial result is a normal
/// answer (200): some rows may be at the wrong stage or outside the caller's
/// scope, and each says why.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkActionResult {
    pub berhasil: Vec<BulkActionBerhasil>,
    pub gagal: Vec<BulkActionGagal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkActionBerhasil {
    pub pengajuan_satker_id: Uuid,
    pub satker_id: String,
    pub aktivitas_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkActionGagal {
    pub pengajuan_satker_id: Uuid,
    pub alasan: String,
}

// ============ Waiting count ============

/// How much pakaian dinas work is waiting on the caller, for the dashboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenungguPakaianDinas {
    pub jumlah: i64,
    /// The role this count is for; empty when the caller's role has no turn in
    /// the pakaian dinas flow.
    pub peran: String,
}

// ============ Workflow readiness ============

/// Pre-flight "kesiapan" for one workflow gate (satker / wilayah / pusat).
///
/// Advisory, never a hard block: the UI shows these numbers in a confirmation
/// dialog and the person may still proceed. The counts are computed server-side
/// so the dialog cannot disagree with the data — the client only ever holds one
/// page of the satker list, not the whole campaign.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct KesiapanPakaianDinas {
    /// `satker` | `wilayah` | `pusat` — which gate this answers.
    pub tahap: String,
    /// Nothing outstanding at this gate.
    pub lengkap: bool,
    /// Units the gate covers: pegawai for `satker`, satker for the others.
    pub total: i64,
    /// Pegawai without a saved size (`satker` only).
    pub pegawai_belum_diisi: i64,
    /// Satker that have not submitted to the wilayah validator (1000/1003).
    pub belum_mengajukan: i64,
    /// Submitted satker whose roster is not fully filled (not 1000/1003/1008).
    pub belum_lengkap: i64,
    /// Satker already at the wilayah step but not forwarded to pusat
    /// (1001/1007/1012).
    pub belum_diteruskan: i64,
    /// Satker already at the pusat step but not finalised (1004/1010).
    pub belum_selesai: i64,
    /// Satker finalised (1008).
    pub selesai: i64,
}

/// Is a gate complete? Pure so the rule is testable without a database.
///
/// `wilayah` waits on the satker it is about to forward: nobody may still be at
/// Input/Revisi and no already-submitted satker may be half-filled. `pusat`
/// additionally waits on the satker wilayah has not forwarded yet — that is the
/// "wilayah belum mengajukan ke pusat" the confirmation names. Satker at pusat
/// awaiting their own final approval (`belum_selesai`) are pusat's own queue,
/// not a subordinate that failed to submit, so they do not make the gate
/// "incomplete".
pub fn kesiapan_lengkap(
    tahap: &str,
    belum_mengajukan: i64,
    belum_lengkap: i64,
    belum_diteruskan: i64,
) -> bool {
    match tahap {
        "satker" => belum_lengkap == 0,
        "wilayah" => belum_mengajukan == 0 && belum_lengkap == 0,
        "pusat" => belum_mengajukan == 0 && belum_lengkap == 0 && belum_diteruskan == 0,
        _ => false,
    }
}

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
/// Satu ukuran pegawai pada satu kolom pakaian pengajuan (nama kolom = `spesifikasi_nama`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LaporanUkuranPegawai {
    pub pakaian_nama: String,
    pub ukuran: String,
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
    /// Ukuran per kolom pakaian, bukan per grup: baju dan kaos sama-sama grup BAJU, dan
    /// mengumpulkannya per grup membuat salah satunya menimpa yang lain. Kolom yang tak
    /// diisi pegawai ini tidak ada di sini.
    pub ukuran: Vec<LaporanUkuranPegawai>,
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
            // Diisi oleh repositori dengan satu kueri untuk seluruh halaman.
            ukuran: Vec::new(),
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

#[cfg(test)]
mod kesiapan_tests {
    use super::kesiapan_lengkap;

    #[test]
    fn each_gate_only_waits_on_its_own_outstanding_work() {
        // satker: only unfilled employees matter.
        assert!(kesiapan_lengkap("satker", 9, 0, 9));
        assert!(!kesiapan_lengkap("satker", 0, 1, 0));

        // wilayah: nobody may still be at Input/Revisi and no already-submitted
        // satker may be half-filled. Satker not yet forwarded are the wilayah's
        // own queue, so they do not make the gate incomplete.
        assert!(!kesiapan_lengkap("wilayah", 1, 0, 5));
        assert!(!kesiapan_lengkap("wilayah", 0, 1, 0));
        assert!(kesiapan_lengkap("wilayah", 0, 0, 5));

        // pusat: also waits on what wilayah has not forwarded yet.
        assert!(!kesiapan_lengkap("pusat", 0, 0, 1));
        assert!(kesiapan_lengkap("pusat", 0, 0, 0));

        // An unknown gate is never silently "complete".
        assert!(!kesiapan_lengkap("lainnya", 0, 0, 0));
    }
}
