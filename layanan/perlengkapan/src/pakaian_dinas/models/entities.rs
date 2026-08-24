use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;
use uuid::Uuid;

use super::*;

// ============ Master Data Models ============

/// Master table: Jenis Pakaian Dinas (Uniform Types)
/// Example: PDH, PDL, Toga Jaksa
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JenisPakaianDinas {
    pub id: Uuid,
    pub nama: String,
    pub deskripsi: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
impl JenisPakaianDinas {
    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            nama: row.get("nama"),
            deskripsi: row.try_get("deskripsi").ok(),
            is_active: row.try_get("is_active").unwrap_or(true),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}
/// Master table: Spesifikasi Pakaian Dinas (Uniform Specifications)
/// Example: Kemeja PDH, Celana PDH, Sepatu Dinas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpesifikasiPakaianDinas {
    pub id: Uuid,
    pub jenis_pakaian_dinas_id: Uuid,
    pub nama: String,
    pub gender: String,
    pub ukuran_group: String,
    pub deskripsi: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Joined fields
    pub jenis_pakaian_nama: Option<String>,
}
impl SpesifikasiPakaianDinas {
    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            jenis_pakaian_dinas_id: row.get("jenis_pakaian_dinas_id"),
            nama: row.get("nama"),
            gender: row.get("gender"),
            ukuran_group: row.get("ukuran_group"),
            deskripsi: row.try_get("deskripsi").ok(),
            is_active: row.try_get("is_active").unwrap_or(true),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            jenis_pakaian_nama: row.try_get("jenis_pakaian_nama").ok(),
        }
    }
}
/// Photo attachment for specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpesifikasiFoto {
    pub id: Uuid,
    pub spesifikasi_id: Uuid,
    pub path: String,
    pub filename: String,
    pub created_at: DateTime<Utc>,
}
impl SpesifikasiFoto {
    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            spesifikasi_id: row.get("spesifikasi_id"),
            path: row.get("path"),
            filename: row.get("filename"),
            created_at: row.get("created_at"),
        }
    }
}
/// Master table: SubSpesifikasi Pakaian Dinas
/// Example: Kemeja Lengan Panjang, Kemeja Lengan Pendek
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubSpesifikasiPakaianDinas {
    pub id: Uuid,
    pub spesifikasi_id: Uuid,
    pub nama: String,
    pub gender: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Joined fields
    pub spesifikasi_nama: Option<String>,
}
impl SubSpesifikasiPakaianDinas {
    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            spesifikasi_id: row.get("spesifikasi_id"),
            nama: row.get("nama"),
            gender: row.get("gender"),
            is_active: row.try_get("is_active").unwrap_or(true),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            spesifikasi_nama: row.try_get("spesifikasi_nama").ok(),
        }
    }
}
/// Master table: Ukuran (Sizes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ukuran {
    pub ukuran: String,
    pub group: String,
    pub urutan: i32,
}
impl Ukuran {
    pub fn from_row(row: &Row) -> Self {
        Self {
            ukuran: row.get("ukuran"),
            group: row.get("group"),
            urutan: row.try_get("urutan").unwrap_or(0),
        }
    }
}
// ============ Transaction Models ============

/// Main request header for uniform procurement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PengajuanPakaianDinas {
    pub id: Uuid,
    pub nama: String,
    pub deskripsi: Option<String>,
    pub tgl_mulai: Option<NaiveDate>,
    pub tgl_selesai: Option<NaiveDate>,
    pub is_reguler: bool,
    pub tahun: i32,
    pub pilihan_satker: String, // "all" or "sebagian"
    pub dengan_unit_kerja: bool,
    pub jenis_pakaian_dinas_id: Option<Uuid>,
    pub aktivitas_id: i32,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Joined/computed fields
    pub jenis_pakaian_nama: Option<String>,
    pub aktivitas_label: Option<String>,
    pub total_satker: Option<i64>,
    pub satker_selesai: Option<i64>,
    /// Derived from `is_reguler` + `tgl_selesai`, not stored. Serialised so the
    /// period rule stays here: the campaign list renders an open/closed badge,
    /// and the rule was previously only reachable through `is_open()`, which no
    /// response carried — so the frontend asked for a field that was never sent
    /// and failed to deserialise the entire list.
    pub is_open: bool,
}
impl PengajuanPakaianDinas {
    pub fn from_row(row: &Row) -> Self {
        let is_reguler = row.try_get("is_reguler").unwrap_or(true);
        let tgl_selesai: Option<NaiveDate> = row.try_get("tgl_selesai").ok();
        Self {
            id: row.get("id"),
            nama: row.get("nama"),
            deskripsi: row.try_get("deskripsi").ok(),
            tgl_mulai: row.try_get("tgl_mulai").ok(),
            tgl_selesai,
            is_reguler,
            tahun: row.get("tahun"),
            pilihan_satker: row.get("pilihan_satker"),
            dengan_unit_kerja: row.try_get("dengan_unit_kerja").unwrap_or(false),
            jenis_pakaian_dinas_id: row.try_get("jenis_pakaian_dinas_id").ok(),
            aktivitas_id: row.get("aktivitas_id"),
            created_by: row.try_get("created_by").ok(),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            jenis_pakaian_nama: row.try_get("jenis_pakaian_nama").ok(),
            aktivitas_label: row.try_get("aktivitas_label").ok(),
            total_satker: row.try_get("total_satker").ok(),
            satker_selesai: row.try_get("satker_selesai").ok(),
            is_open: Self::period_is_open(is_reguler, tgl_selesai),
        }
    }

    /// The period rule, in one place. `from_row` stamps it onto the serialised
    /// field and [`Self::is_open`] reads it for callers holding an entity.
    fn period_is_open(is_reguler: bool, tgl_selesai: Option<NaiveDate>) -> bool {
        if !is_reguler {
            return true; // Non-regular requests are always open
        }
        match tgl_selesai {
            Some(end_date) => chrono::Local::now().date_naive() <= end_date,
            None => true,
        }
    }

    /// Check if the submission period is still open
    pub fn is_open(&self) -> bool {
        Self::period_is_open(self.is_reguler, self.tgl_selesai)
    }
}
/// Selected satkers for a pengajuan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PengajuanSatkerTerpilih {
    pub pengajuan_id: Uuid,
    /// MySIMKARI `kode_satker` — SoT `integrasi.mysimkari_satker.kode_satker`.
    /// NOT the bigint surrogate `id` (reassigned on re-sync). See V006 / #94.
    pub satker_id: String,
    pub satker_pusat_id: Option<String>,
    pub is_show_in_form: bool,
    // Joined fields
    pub satker_nama: Option<String>,
    pub satker_kode: Option<String>,
}
impl PengajuanSatkerTerpilih {
    pub fn from_row(row: &Row) -> Self {
        Self {
            pengajuan_id: row.get("pengajuan_id"),
            satker_id: row.get("satker_id"),
            satker_pusat_id: row.try_get("satker_pusat_id").ok(),
            is_show_in_form: row.try_get("is_show_in_form").unwrap_or(true),
            satker_nama: row.try_get("satker_nama").ok(),
            satker_kode: row.try_get("satker_kode").ok(),
        }
    }
}
/// Clothing items selected for a pengajuan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PengajuanPakaian {
    pub id: Uuid,
    pub pengajuan_id: Uuid,
    pub jenis_pakaian_id: Uuid,
    pub jenis_pakaian_nama: String,
    pub spesifikasi_id: Uuid,
    pub spesifikasi_nama: String,
    pub spesifikasi_ukuran_group: String,
    pub subspesifikasi_id: Option<Uuid>,
    pub subspesifikasi_nama: Option<String>,
    pub subspesifikasi_gender: Option<String>,
}
impl PengajuanPakaian {
    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            pengajuan_id: row.get("pengajuan_id"),
            jenis_pakaian_id: row.get("jenis_pakaian_id"),
            jenis_pakaian_nama: row.get("jenis_pakaian_nama"),
            spesifikasi_id: row.get("spesifikasi_id"),
            spesifikasi_nama: row.get("spesifikasi_nama"),
            spesifikasi_ukuran_group: row.get("spesifikasi_ukuran_group"),
            subspesifikasi_id: row.try_get("subspesifikasi_id").ok(),
            subspesifikasi_nama: row.try_get("subspesifikasi_nama").ok(),
            subspesifikasi_gender: row.try_get("subspesifikasi_gender").ok(),
        }
    }
}
/// Per-satker submission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PengajuanSatker {
    pub id: Uuid,
    pub pengajuan_id: Uuid,
    /// MySIMKARI `kode_satker` — SoT `integrasi.mysimkari_satker.kode_satker`.
    /// NOT the bigint surrogate `id` (reassigned on re-sync). See V006 / #94.
    pub satker_id: String,
    pub satker_pusat_id: Option<String>,
    pub aktivitas_id: i32,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Joined fields
    pub satker_nama: Option<String>,
    pub satker_kode: Option<String>,
    pub aktivitas_label: Option<String>,
    pub total_pegawai: Option<i64>,
}
impl PengajuanSatker {
    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            pengajuan_id: row.get("pengajuan_id"),
            satker_id: row.get("satker_id"),
            satker_pusat_id: row.try_get("satker_pusat_id").ok(),
            aktivitas_id: row.get("aktivitas_id"),
            created_by: row.try_get("created_by").ok(),
            updated_by: row.try_get("updated_by").ok(),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            satker_nama: row.try_get("satker_nama").ok(),
            satker_kode: row.try_get("satker_kode").ok(),
            aktivitas_label: row.try_get("aktivitas_label").ok(),
            total_pegawai: row.try_get("total_pegawai").ok(),
        }
    }
}
/// Employee data within a satker submission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PengajuanSatkerPegawai {
    pub id: Uuid,
    pub pengajuan_satker_id: Uuid,
    pub nip: String,
    pub nama: String,
    pub pangkat: Option<String>,
    pub jabatan: Option<String>,
    pub eselon: Option<String>,
    pub jenis_kelamin: String,
    pub gol_kd: Option<String>,
    pub jenis: Option<String>, // Jaksa/TU/etc
    pub with_hijab: bool,
    pub created_at: DateTime<Utc>,
}
impl PengajuanSatkerPegawai {
    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            pengajuan_satker_id: row.get("pengajuan_satker_id"),
            nip: row.get("nip"),
            nama: row.get("nama"),
            pangkat: row.try_get("pangkat").ok(),
            jabatan: row.try_get("jabatan").ok(),
            eselon: row.try_get("eselon").ok(),
            jenis_kelamin: row.get("jenis_kelamin"),
            gol_kd: row.try_get("gol_kd").ok(),
            jenis: row.try_get("jenis").ok(),
            with_hijab: row.try_get("with_hijab").unwrap_or(false),
            created_at: row.get("created_at"),
        }
    }
}
/// Employee's clothing size within a submission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PengajuanSatkerPegawaiUkuran {
    pub pengajuan_satker_id: Uuid,
    pub pegawai_id: Uuid,
    pub pakaian_id: Uuid,
    pub ukuran: String,
    // Joined fields
    pub pakaian_nama: Option<String>,
    pub ukuran_group: Option<String>,
}
impl PengajuanSatkerPegawaiUkuran {
    pub fn from_row(row: &Row) -> Self {
        Self {
            pengajuan_satker_id: row.get("pengajuan_satker_id"),
            pegawai_id: row.get("pegawai_id"),
            pakaian_id: row.get("pakaian_id"),
            ukuran: row.get("ukuran"),
            pakaian_nama: row.try_get("pakaian_nama").ok(),
            ukuran_group: row.try_get("ukuran_group").ok(),
        }
    }
}
/// Workflow activity history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PengajuanSatkerAktivitas {
    pub id: Uuid,
    pub pengajuan_satker_id: Uuid,
    pub aktivitas_id: i32,
    pub komentar: Option<String>,
    pub nip: Option<String>,
    pub nama: Option<String>,
    pub pangkat: Option<String>,
    pub jabatan: Option<String>,
    pub role: Option<String>,
    pub created_at: DateTime<Utc>,
}
impl PengajuanSatkerAktivitas {
    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            pengajuan_satker_id: row.get("pengajuan_satker_id"),
            aktivitas_id: row.get("aktivitas_id"),
            komentar: row.try_get("komentar").ok(),
            nip: row.try_get("nip").ok(),
            nama: row.try_get("nama").ok(),
            pangkat: row.try_get("pangkat").ok(),
            jabatan: row.try_get("jabatan").ok(),
            role: row.try_get("role").ok(),
            created_at: row.get("created_at"),
        }
    }
}
// ============ Persistent Employee Size Models ============

/// Persistent per-employee profile for pakaian dinas. Stores the uniform
/// sizes *and* the reporting-relevant attributes (eselon, jenis kelamin,
/// jenis pegawai, mapped unit kerja, kode satker) that the MySIMKARI API
/// does not expose. Updated by the pakaian dinas wizard and by approved
/// pengajuan, not by MySIMKARI sync.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PegawaiPakaianDinas {
    pub nip: String,
    pub nama: Option<String>,
    pub ukuran_baju: Option<String>,
    pub ukuran_celana: Option<String>,
    pub ukuran_sepatu: Option<String>,
    pub with_hijab: bool,
    pub pangkat: Option<String>,
    pub jabatan: Option<String>,
    pub status: Option<String>,
    pub eselon: Option<String>,
    pub jenis_kelamin: Option<String>,
    pub jenis_pegawai: Option<String>,
    pub mapped_unit_kerja: Option<String>,
    pub kode_satker: Option<String>,
    pub last_pengajuan_satker_pegawai_id: Option<Uuid>,
    pub updated_at: DateTime<Utc>,
}
impl PegawaiPakaianDinas {
    pub fn from_row(row: &Row) -> Self {
        Self {
            nip: row.get("nip"),
            nama: row.try_get("nama").ok(),
            ukuran_baju: row.try_get("ukuran_baju").ok(),
            ukuran_celana: row.try_get("ukuran_celana").ok(),
            ukuran_sepatu: row.try_get("ukuran_sepatu").ok(),
            with_hijab: row.try_get("with_hijab").unwrap_or(false),
            pangkat: row.try_get("pangkat").ok(),
            jabatan: row.try_get("jabatan").ok(),
            status: row.try_get("status").ok(),
            eselon: row.try_get("eselon").ok(),
            jenis_kelamin: row.try_get("jenis_kelamin").ok(),
            jenis_pegawai: row.try_get("jenis_pegawai").ok(),
            mapped_unit_kerja: row.try_get("mapped_unit_kerja").ok(),
            kode_satker: row.try_get("kode_satker").ok(),
            last_pengajuan_satker_pegawai_id: row.try_get("last_pengajuan_satker_pegawai_id").ok(),
            updated_at: row.get("updated_at"),
        }
    }
}
// ============ MySIMKARI Integration Models ============

/// Employee data from MySIMKARI integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MysimkariPegawai {
    /// `integrasi.mysimkari_pegawai.id` is BIGSERIAL (int8). This was `i32`,
    /// and `row.get::<_, i32>` on an int8 column PANICS — so every roster read
    /// 500'd on the first row even once the query itself was fixed (#94).
    pub id: i64,
    pub nama: String,
    pub nip: String,
    pub no_hp: Option<String>,
    pub email_dinas: Option<String>,
    pub bidang: Option<String>,
    pub foto: Option<String>,
    pub jk: String,
    pub agama: Option<String>,
    pub nrp: Option<String>,
    pub jabatan: Option<String>,
    pub golpang: Option<String>,
    pub jenis_jabatan_terakhir: Option<String>,
    pub jabat_tmt: Option<String>,
    pub eselon: Option<String>,
    pub nama_satker: Option<String>,
    pub gol_kd: Option<String>,
    /// MySIMKARI `kode_satker` — the column is TEXT. This was `Option<Uuid>`
    /// read via `try_get(..).ok()`, so the type error was swallowed and the
    /// field silently deserialized to `None` on EVERY row (#94).
    pub satker_id: Option<String>,
}
impl MysimkariPegawai {
    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            nama: row.get("nama"),
            nip: row.get("nip"),
            no_hp: row.try_get("no_hp").ok(),
            email_dinas: row.try_get("email_dinas").ok(),
            bidang: row.try_get("bidang").ok(),
            foto: row.try_get("foto").ok(),
            jk: row.try_get("jk").unwrap_or_else(|_| "L".to_string()),
            agama: row.try_get("agama").ok(),
            nrp: row.try_get("nrp").ok(),
            jabatan: row.try_get("jabatan").ok(),
            golpang: row.try_get("golpang").ok(),
            jenis_jabatan_terakhir: row.try_get("jenis_jabatan_terakhir").ok(),
            jabat_tmt: row.try_get("jabat_tmt").ok(),
            eselon: row.try_get("eselon").ok(),
            nama_satker: row.try_get("nama_satker").ok(),
            gol_kd: row.try_get("gol_kd").ok(),
            satker_id: row.try_get("satker_id").ok(),
        }
    }

    /// Convert to pegawai for pengajuan submission
    pub fn to_pengajuan_pegawai(
        &self,
        existing_sizes: Option<&PegawaiPakaianDinas>,
    ) -> CreatePegawaiUkuranRequest {
        CreatePegawaiUkuranRequest {
            nip: self.nip.clone(),
            nama: self.nama.clone(),
            pangkat: self.golpang.clone(),
            jabatan: self.jabatan.clone(),
            eselon: self.eselon.clone(),
            jenis_kelamin: self.jk.clone(),
            gol_kd: self.gol_kd.clone(),
            jenis: self.jenis_jabatan_terakhir.clone(),
            with_hijab: existing_sizes.map(|s| s.with_hijab).unwrap_or(false),
            ukuran: vec![], // Will be filled from existing sizes or user input
        }
    }
}
