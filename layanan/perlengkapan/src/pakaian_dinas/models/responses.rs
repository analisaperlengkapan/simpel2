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

/// Pre-flight "kesiapan" for the confirmation dialog shown before a workflow move.
///
/// Advisory, never a hard block: the person sees the numbers and may still
/// proceed. The counts are computed server-side so the dialog cannot disagree
/// with the data: the client only ever holds one page of the satker list.
///
/// **The warning follows the scope of the action**, and so does this shape:
///
/// * `tahap = "satker"`: ONE satker. Used by the operator submitting their own
///   satker and by a validator moving a single satker row on. Only `total` and
///   `pegawai_belum_diisi` are filled: how many of THAT satker's employees owe a
///   size and still have none. Nothing about any other satker, because moving
///   this one does not depend on them.
/// * `tahap = "wilayah" | "pusat"`: a bulk move of several rows (`terpilih`).
///   Two groups, never mixed:
///     - `fokus_*`: the rows being moved. How many are not complete and how many
///       employees are missing in them.
///     - the rest, the satker in the caller's scope that are NOT being moved
///       (`total` of them): `belum_mengajukan` (never submitted, still below the
///       wilayah) and, for `pusat`, `belum_diteruskan` (the wilayah has not
///       forwarded them yet).
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct KesiapanPakaianDinas {
    /// `satker` | `wilayah` | `pusat`: which gate this answers.
    pub tahap: String,
    /// Nothing outstanding for this action.
    pub lengkap: bool,
    /// `satker`: employees in the satker's roster. `wilayah`/`pusat`: the other
    /// satker in scope, the ones NOT being moved.
    pub total: i64,
    /// `satker` only: employees of that satker without a saved size.
    pub pegawai_belum_diisi: i64,
    /// Bulk only: rows being moved.
    #[serde(default)]
    pub fokus_satker: i64,
    /// Bulk only: of the rows being moved, how many have at least one employee
    /// without a size.
    #[serde(default)]
    pub fokus_belum_lengkap: i64,
    /// Bulk only: employees without a size in the rows being moved.
    #[serde(default)]
    pub fokus_pegawai_belum: i64,
    /// Bulk only: employees in the roster of the rows being moved.
    #[serde(default)]
    pub fokus_pegawai_total: i64,
    /// Bulk only, outside the moved rows: satker that have not submitted to the
    /// wilayah validator (1000/1003) and have employees to submit for.
    pub belum_mengajukan: i64,
    /// Bulk only, outside the moved rows: satker waiting at the wilayah step,
    /// not forwarded to pusat (1001/1007/1012).
    pub belum_diteruskan: i64,
}

/// Is a gate complete? Pure so the rule is testable without a database.
///
/// `satker`: every employee of the satker has a size. A bulk move is complete
/// when the rows being moved are all filled AND nobody is left below the level
/// that moves: for `wilayah` no satker has still to submit; for `pusat` none has
/// still to submit and none is still waiting at the wilayah. Satker already at
/// pusat waiting for their own final approval are pusat's own queue, not a
/// subordinate that failed to submit, so they never make the gate incomplete.
pub fn kesiapan_lengkap(
    tahap: &str,
    pegawai_belum_diisi: i64,
    fokus_belum_lengkap: i64,
    belum_mengajukan: i64,
    belum_diteruskan: i64,
) -> bool {
    match tahap {
        "satker" => pegawai_belum_diisi == 0,
        "wilayah" => fokus_belum_lengkap == 0 && belum_mengajukan == 0,
        "pusat" => fokus_belum_lengkap == 0 && belum_mengajukan == 0 && belum_diteruskan == 0,
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
        // satker: only that satker's unfilled employees matter, nothing else.
        assert!(kesiapan_lengkap("satker", 0, 9, 9, 9));
        assert!(!kesiapan_lengkap("satker", 1, 0, 0, 0));

        // wilayah (bulk): the rows being moved must be complete and no satker may
        // still be below. Satker waiting at the wilayah itself are its own queue.
        assert!(!kesiapan_lengkap("wilayah", 0, 1, 0, 0));
        assert!(!kesiapan_lengkap("wilayah", 0, 0, 1, 0));
        assert!(kesiapan_lengkap("wilayah", 0, 0, 0, 5));

        // pusat (bulk): also waits on what the wilayah has not forwarded yet.
        assert!(!kesiapan_lengkap("pusat", 0, 0, 0, 1));
        assert!(!kesiapan_lengkap("pusat", 0, 1, 0, 0));
        assert!(kesiapan_lengkap("pusat", 0, 0, 0, 0));

        // An unknown gate is never silently "complete".
        assert!(!kesiapan_lengkap("lainnya", 0, 0, 0, 0));
    }
}
