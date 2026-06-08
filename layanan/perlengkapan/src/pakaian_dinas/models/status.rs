use serde::{Deserialize, Serialize};

// ============ Enums ============

/// Gender options for uniform specifications
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Gender {
    L,     // Laki-laki (Male)
    P,     // Perempuan (Female)
    Semua, // All genders
}
impl Gender {
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "L" => Gender::L,
            "P" => Gender::P,
            _ => Gender::Semua,
        }
    }

    pub fn to_db_string(&self) -> &'static str {
        match self {
            Gender::L => "L",
            Gender::P => "P",
            Gender::Semua => "SEMUA",
        }
    }
}
/// Ukuran group for clothing categories
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum UkuranGroup {
    Baju,
    Celana,
    Sepatu,
}
impl UkuranGroup {
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "BAJU" => UkuranGroup::Baju,
            "CELANA" => UkuranGroup::Celana,
            "SEPATU" => UkuranGroup::Sepatu,
            _ => UkuranGroup::Baju,
        }
    }

    pub fn to_db_string(&self) -> &'static str {
        match self {
            UkuranGroup::Baju => "BAJU",
            UkuranGroup::Celana => "CELANA",
            UkuranGroup::Sepatu => "SEPATU",
        }
    }
}
/// Workflow status codes for pakaian dinas requests
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AktivitasStatus {
    Input = 1000,
    SubmitToValidator = 1001,
    SubmitToValidatorWilayah = 1012,
    RevisiPelaksana = 1003,
    RevisiSatker = 1005,
    Ditolak = 1006,
    RevisiWilayah = 1007,
    SubmitToPusat = 1004,
    SubmitToPusatFromWilayah = 1010,
    Selesai = 1008,
    StartKejagung = 1009,
    StartNonKejagung = 1011,
}
impl AktivitasStatus {
    pub fn from_i32(code: i32) -> Option<Self> {
        match code {
            1000 => Some(AktivitasStatus::Input),
            1001 => Some(AktivitasStatus::SubmitToValidator),
            1003 => Some(AktivitasStatus::RevisiPelaksana),
            1004 => Some(AktivitasStatus::SubmitToPusat),
            1005 => Some(AktivitasStatus::RevisiSatker),
            1006 => Some(AktivitasStatus::Ditolak),
            1007 => Some(AktivitasStatus::RevisiWilayah),
            1008 => Some(AktivitasStatus::Selesai),
            1009 => Some(AktivitasStatus::StartKejagung),
            1010 => Some(AktivitasStatus::SubmitToPusatFromWilayah),
            1011 => Some(AktivitasStatus::StartNonKejagung),
            1012 => Some(AktivitasStatus::SubmitToValidatorWilayah),
            _ => None,
        }
    }

    pub fn to_i32(&self) -> i32 {
        match self {
            AktivitasStatus::Input => 1000,
            AktivitasStatus::SubmitToValidator => 1001,
            AktivitasStatus::SubmitToValidatorWilayah => 1012,
            AktivitasStatus::RevisiPelaksana => 1003,
            AktivitasStatus::RevisiSatker => 1005,
            AktivitasStatus::Ditolak => 1006,
            AktivitasStatus::RevisiWilayah => 1007,
            AktivitasStatus::SubmitToPusat => 1004,
            AktivitasStatus::SubmitToPusatFromWilayah => 1010,
            AktivitasStatus::Selesai => 1008,
            AktivitasStatus::StartKejagung => 1009,
            AktivitasStatus::StartNonKejagung => 1011,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            AktivitasStatus::Input => "Input",
            AktivitasStatus::SubmitToValidator => "Diajukan ke Validator",
            AktivitasStatus::SubmitToValidatorWilayah => "Diajukan ke Validator Wilayah",
            AktivitasStatus::RevisiPelaksana => "Revisi Pelaksana",
            AktivitasStatus::RevisiSatker => "Revisi Satker",
            AktivitasStatus::Ditolak => "Ditolak",
            AktivitasStatus::RevisiWilayah => "Revisi Wilayah",
            AktivitasStatus::SubmitToPusat => "Diajukan ke Pusat",
            AktivitasStatus::SubmitToPusatFromWilayah => "Diajukan ke Pusat dari Wilayah",
            AktivitasStatus::Selesai => "Selesai",
            AktivitasStatus::StartKejagung => "Mulai (Kejagung)",
            AktivitasStatus::StartNonKejagung => "Mulai (Non-Kejagung)",
        }
    }
}
