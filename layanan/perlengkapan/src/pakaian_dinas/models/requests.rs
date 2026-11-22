use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Request DTO for creating/updating Jenis Pakaian Dinas
#[derive(Debug, Deserialize, Validate)]
pub struct CreateJenisPakaianDinasRequest {
    #[validate(length(
        min = 1,
        max = 255,
        message = "Nama harus diisi dan maksimal 255 karakter"
    ))]
    pub nama: String,
    pub deskripsi: Option<String>,
    #[serde(default = "default_true")]
    pub is_active: bool,
}
/// Request DTO for creating/updating Spesifikasi
#[derive(Debug, Deserialize, Validate)]
pub struct CreateSpesifikasiRequest {
    pub jenis_pakaian_dinas_id: Uuid,
    #[validate(length(min = 1, max = 255, message = "Nama harus diisi"))]
    pub nama: String,
    #[validate(length(min = 1, max = 10, message = "Gender harus diisi"))]
    pub gender: String, // L, P, or SEMUA
    #[validate(length(min = 1, max = 50, message = "Ukuran group harus diisi"))]
    pub ukuran_group: String, // BAJU, CELANA, or SEPATU
    pub deskripsi: Option<String>,
    #[serde(default = "default_true")]
    pub is_active: bool,
}
/// Request DTO for creating SubSpesifikasi
#[derive(Debug, Deserialize, Validate)]
pub struct CreateSubSpesifikasiRequest {
    pub spesifikasi_id: Uuid,
    #[validate(length(min = 1, max = 255, message = "Nama harus diisi"))]
    pub nama: String,
    #[validate(length(min = 1, max = 10, message = "Gender harus diisi"))]
    pub gender: String,
    #[serde(default = "default_true")]
    pub is_active: bool,
}
/// Request DTO for creating Pengajuan
#[derive(Debug, Deserialize, Validate)]
pub struct CreatePengajuanRequest {
    #[validate(length(min = 1, max = 255, message = "Nama pengajuan harus diisi"))]
    pub nama: String,
    pub deskripsi: Option<String>,
    pub tgl_mulai: Option<NaiveDate>,
    pub tgl_selesai: Option<NaiveDate>,
    #[serde(default = "default_true")]
    pub is_reguler: bool,
    pub tahun: Option<i32>,
    #[validate(length(min = 1, message = "Pilihan satker harus diisi"))]
    pub pilihan_satker: String,
    #[serde(default)]
    pub dengan_unit_kerja: bool,
    pub jenis_pakaian_dinas_id: Option<Uuid>,
    pub spesifikasi_ids: Vec<Uuid>, // Selected specifications
    /// Selected satkers (if `pilihan_satker = "sebagian"`), as MySIMKARI
    /// `kode_satker` — not the bigint surrogate id (V006/#94).
    pub satker_ids: Option<Vec<String>>,
    /// Wilayah Kejaksaan Tinggi (#19) — wajib jika `pilihan_satker = "wilayah"`.
    /// Satker di-resolve otomatis dari `integrasi.v_satker_wilayah`: nilainya
    /// adalah `kode_satker` Kejaksaan Tinggi penaung.
    #[serde(default)]
    pub wilayah_id: Option<String>,
}
/// Request DTO for editing a Pengajuan. Every field is optional: what is absent
/// stays as it is. Mainly how a closed campaign is extended, since the dates are
/// enforced.
#[derive(Debug, Deserialize, Validate)]
pub struct UpdatePengajuanRequest {
    #[validate(length(min = 1, max = 255, message = "Nama pengajuan harus diisi"))]
    pub nama: Option<String>,
    pub deskripsi: Option<String>,
    pub tgl_mulai: Option<NaiveDate>,
    pub tgl_selesai: Option<NaiveDate>,
}
/// Request DTO for creating/updating employee with sizes
#[derive(Debug, Deserialize, Validate)]
pub struct CreatePegawaiUkuranRequest {
    #[validate(length(min = 1, message = "NIP harus diisi"))]
    pub nip: String,
    #[validate(length(min = 1, message = "Nama harus diisi"))]
    pub nama: String,
    pub pangkat: Option<String>,
    pub jabatan: Option<String>,
    pub eselon: Option<String>,
    #[validate(length(min = 1, max = 1, message = "Jenis kelamin harus L atau P"))]
    pub jenis_kelamin: String,
    pub gol_kd: Option<String>,
    pub jenis: Option<String>,
    #[serde(default)]
    pub with_hijab: bool,
    pub ukuran: Vec<PegawaiUkuranItem>,
}
/// Size entry for a specific clothing item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PegawaiUkuranItem {
    pub pakaian_id: Uuid,
    pub ukuran: String,
}
/// Request DTO for workflow action (approve/reject)
#[derive(Debug, Deserialize, Validate)]
pub struct ValidatorActionRequest {
    pub pengajuan_satker_id: Uuid,
    pub aksi: String, // "approve" or "reject"
    pub komentar: Option<String>,
}
/// Request DTO for approving several satker at once.
///
/// Approval only: a return needs its own note per satker, so it stays one at a
/// time. Each id goes through exactly the path a single approval takes.
#[derive(Debug, Deserialize)]
pub struct BulkValidatorActionRequest {
    pub pengajuan_satker_ids: Vec<Uuid>,
    pub aksi: String,
    pub komentar: Option<String>,
}
/// Request DTO for personal uniform sizes (self-service)
#[derive(Debug, Deserialize, Validate)]
pub struct UpdatePersonalUkuranRequest {
    #[validate(length(min = 1, message = "Ukuran baju harus diisi"))]
    pub ukuran_baju: String,
    #[validate(length(min = 1, message = "Ukuran celana harus diisi"))]
    pub ukuran_celana: String,
    #[validate(length(min = 1, message = "Ukuran sepatu harus diisi"))]
    pub ukuran_sepatu: String,
    /// `Option`, not a `#[serde(default)] bool`: this is the self-service form,
    /// which only owns the three sizes. A plain default meant every save posted
    /// `false` and the upsert wrote it over whatever the operator wizard
    /// (`UpsertPegawaiProfileRequest`) had set — silent data loss in a field the
    /// uniform reports read. `None` now means "leave it alone"; see the COALESCE
    /// in `repository/pegawai.rs::upsert_pegawai_pakaian_dinas`.
    #[serde(default)]
    pub with_hijab: Option<bool>,
}
/// Full profile upsert — used by satker operators to set the reporting
/// fields that MySIMKARI does not carry (eselon, gender, jenis pegawai,
/// hijab flag, mapped unit kerja) alongside sizes. Called during the
/// pakaian dinas wizard and by the admin bulk-import.
#[derive(Debug, Deserialize, Validate)]
pub struct UpsertPegawaiProfileRequest {
    #[validate(length(min = 1, max = 30, message = "NIP tidak valid"))]
    pub nip: String,
    pub nama: Option<String>,
    pub pangkat: Option<String>,
    pub jabatan: Option<String>,
    pub eselon: Option<String>,
    /// "L" or "P"
    pub jenis_kelamin: Option<String>,
    /// "TU", "Jaksa", etc.
    pub jenis_pegawai: Option<String>,
    #[serde(default)]
    pub with_hijab: bool,
    pub mapped_unit_kerja: Option<String>,
    pub kode_satker: Option<String>,
    pub ukuran_baju: Option<String>,
    pub ukuran_celana: Option<String>,
    pub ukuran_sepatu: Option<String>,
}
/// Report filter options
#[derive(Debug, Clone, Deserialize, Default)]
pub struct LaporanFilter {
    pub pengajuan_id: Option<Uuid>,
    pub tahun: Option<i32>,
    /// MySIMKARI `kode_satker` (V006/#94).
    pub satker_id: Option<String>,
    /// `ms_jenis_pakaian_dinas.id`, denormalised onto
    /// `pengajuan_pakaian_dinas_pakaian.jenis_pakaian_id`.
    ///
    /// Not to be confused with `jenis` below, which is the *employee* category
    /// (Jaksa / Tata Usaha) stored on `..._satker_pegawai.jenis`.
    pub jenis_pakaian_id: Option<Uuid>,
    pub jenis_kelamin: Option<String>,
    pub eselon: Option<String>,
    pub jenis: Option<String>,
}
// ============ Utility Functions ============

fn default_true() -> bool {
    true
}

/// Menyimpan ukuran satu pegawai pada satu pengajuan.
///
/// Identitas pegawai TIDAK dikirim klien: ia diambil dari
/// `integrasi.mysimkari_pegawai` saat menyimpan, lalu dibekukan sebagai bukti
/// historis pada baris pengajuan. Jabatan dan pangkat berubah karena mutasi
/// dan promosi, jadi yang perlu tercatat adalah keadaan saat pengajuan dibuat
/// — dan membiarkan klien mengirimnya berarti mengizinkan nama seseorang
/// diubah dari luar.
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct SimpanUkuranPegawaiRequest {
    #[serde(default)]
    pub with_hijab: bool,
    /// Kosong berarti pegawai tercatat ikut serta tetapi ukurannya belum ada;
    /// itu keadaan yang sah dan berbeda dari tidak diikutkan sama sekali.
    #[serde(default)]
    pub ukuran: Vec<UkuranPegawaiInput>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UkuranPegawaiInput {
    pub pakaian_id: Uuid,
    pub ukuran: String,
}

/// Request DTO untuk membuat grup ukuran (V019). `kode` dinormalkan menjadi HURUF BESAR
/// tanpa spasi; `tipe` bawaannya `PILIHAN`.
#[derive(Debug, Deserialize, Validate)]
pub struct CreateUkuranGrupRequest {
    #[validate(length(
        min = 1,
        max = 50,
        message = "Kode harus diisi dan maksimal 50 karakter"
    ))]
    pub kode: String,
    #[validate(length(
        min = 1,
        max = 100,
        message = "Nama harus diisi dan maksimal 100 karakter"
    ))]
    pub nama: String,
    /// `PILIHAN` (bawaan) atau `TEKS`.
    pub tipe: Option<String>,
    pub urutan: Option<i32>,
}

/// Request DTO untuk mengubah grup ukuran. `tipe` TIDAK bisa diubah: mengubah grup
/// berdaftar menjadi teks (atau sebaliknya) membuat isian yang sudah tersimpan tidak
/// lagi sesuai dengan aturan grupnya.
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUkuranGrupRequest {
    #[validate(length(
        min = 1,
        max = 100,
        message = "Nama harus diisi dan maksimal 100 karakter"
    ))]
    pub nama: String,
    pub urutan: Option<i32>,
    #[serde(default = "default_true")]
    pub is_active: bool,
}

/// Request DTO untuk menambah satu ukuran ke daftar sebuah grup `PILIHAN`.
#[derive(Debug, Deserialize, Validate)]
pub struct CreateUkuranRequest {
    #[validate(length(
        min = 1,
        max = 20,
        message = "Ukuran harus diisi dan maksimal 20 karakter"
    ))]
    pub ukuran: String,
    #[validate(length(min = 1, max = 50, message = "Grup harus diisi"))]
    pub group: String,
    /// Kosong = ditaruh di akhir daftar.
    pub urutan: Option<i32>,
}

/// Request DTO untuk mengubah urutan sebuah ukuran. Nilainya sendiri tidak diganti:
/// mengganti nama berarti membuat yang baru dan menghapus yang lama, supaya isian yang
/// memakainya tidak berubah diam-diam.
#[derive(Debug, Deserialize)]
pub struct UpdateUkuranUrutanRequest {
    pub urutan: i32,
}

/// Request DTO untuk menyusun ulang SELURUH daftar ukuran sebuah grup dalam satu permintaan.
/// Satu klik panah di layar mengubah urutan beberapa ukuran sekaligus; mengirimnya satu per
/// satu membiarkan separuhnya tersimpan bila salah satu gagal.
#[derive(Debug, Deserialize)]
pub struct UrutkanUkuranRequest {
    /// Seluruh ukuran grup itu dalam urutan yang diinginkan: harus persis himpunan yang ada
    /// (tanpa ganda, tanpa kurang, tanpa lebih), supaya daftar yang sudah berubah di server
    /// tak ditimpa susunan dari layar yang basi.
    pub ukuran: Vec<String>,
}

/// Satu isian ukuran pada profil pegawai (V019).
#[derive(Debug, Clone, Deserialize)]
pub struct UkuranProfilInput {
    pub spesifikasi_id: Uuid,
    /// Kosong = hapus isian untuk spesifikasi ini.
    pub ukuran: String,
}

/// Request DTO untuk menyimpan profil ukuran per spesifikasi.
#[derive(Debug, Deserialize)]
pub struct SimpanUkuranProfilRequest {
    #[serde(default)]
    pub ukuran: Vec<UkuranProfilInput>,
    /// Pakaian muslimah. `None` = tidak mengubah yang tercatat: halaman Ukuran lama menyimpan
    /// bendera ini bersama tiga ukuran, jadi halaman baru yang tak bisa memakai endpoint itu
    /// (menimpa ukuran per spesifikasi) membutuhkannya di sini.
    #[serde(default)]
    pub with_hijab: Option<bool>,
}

/// What narrows the per-satker list of one pengajuan: a free-text search and a set of
/// workflow statuses. A nationwide campaign carries every MySIMKARI satker (545 on
/// staging), and the list used to be one page of the first 100 by name — nothing beyond
/// them could be reached by a validator pusat or an admin, and there was no way to ask
/// "which ones are waiting on me".
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SatkerListFilter {
    /// Already a `LIKE` pattern with the user's wildcards escaped: `%` and `_` in what
    /// was typed must match themselves, not "anything".
    pub q_pattern: Option<String>,
    /// `aktivitas_id` values the row may stand at. Empty = every status.
    pub statuses: Vec<i32>,
}

impl SatkerListFilter {
    /// Longest search text honoured. Anything past it is not a name or a code anyone typed.
    pub const MAX_Q: usize = 100;
    /// More status codes than the workflow has is a malformed request, not a wide filter.
    pub const MAX_STATUSES: usize = 16;

    /// `q` is the raw `?q=`; `status` the raw `?status=1001,1007`. Blank means "no filter".
    pub fn parse(
        q: Option<&str>,
        status: Option<&str>,
    ) -> Result<Self, crate::shared::error::AppError> {
        use crate::shared::error::bad_request;

        let q_pattern = match q.map(str::trim).filter(|s| !s.is_empty()) {
            None => None,
            Some(s) if s.chars().count() > Self::MAX_Q => {
                return Err(bad_request(&format!(
                    "Kata pencarian maksimal {} karakter",
                    Self::MAX_Q
                )));
            }
            Some(s) => {
                let mut escaped = String::with_capacity(s.len() + 2);
                escaped.push('%');
                for c in s.chars() {
                    if matches!(c, '\\' | '%' | '_') {
                        escaped.push('\\');
                    }
                    escaped.push(c);
                }
                escaped.push('%');
                Some(escaped)
            }
        };

        let mut statuses = Vec::new();
        for token in status
            .unwrap_or_default()
            .split(',')
            .map(str::trim)
            .filter(|t| !t.is_empty())
        {
            let code: i32 = token
                .parse()
                .map_err(|_| bad_request(&format!("Status tidak dikenal: {token}")))?;
            if !statuses.contains(&code) {
                statuses.push(code);
            }
        }
        if statuses.len() > Self::MAX_STATUSES {
            return Err(bad_request("Terlalu banyak status pada satu permintaan"));
        }

        Ok(Self {
            q_pattern,
            statuses,
        })
    }
}
