//! Common types and enums for Perlengkapan domain

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Status for various perlengkapan entities
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PerlengkapanStatus {
    Draft,
    Pending,
    Approved,
    Rejected,
    InProgress,
    Completed,
    Cancelled,
}

impl std::fmt::Display for PerlengkapanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Draft => write!(f, "draft"),
            Self::Pending => write!(f, "pending"),
            Self::Approved => write!(f, "approved"),
            Self::Rejected => write!(f, "rejected"),
            Self::InProgress => write!(f, "in_progress"),
            Self::Completed => write!(f, "completed"),
            Self::Cancelled => write!(f, "cancelled"),
        }
    }
}

/// Asset condition categories
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum AssetCondition {
    Baik,
    KurangBaik,
    RusakBerat,
    RusakRingan,
}

impl std::fmt::Display for AssetCondition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Baik => write!(f, "Baik"),
            Self::KurangBaik => write!(f, "Kurang Baik"),
            Self::RusakBerat => write!(f, "Rusak Berat"),
            Self::RusakRingan => write!(f, "Rusak Ringan"),
        }
    }
}

/// Priority levels
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

/// Asset categories (from SIMAN)
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum AssetCategory {
    Tanah,
    GedungBangunan,
    PeralatanMesin,
    JalanIrigasiJaringan,
    AsetTetapLainnya,
    KonstruksiDalamPengerjaan,
    AsetTakBerwujud,
    Persediaan,
}

impl AssetCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Tanah => "tanah",
            Self::GedungBangunan => "gedung_bangunan",
            Self::PeralatanMesin => "peralatan_mesin",
            Self::JalanIrigasiJaringan => "jalan_irigasi_jaringan",
            Self::AsetTetapLainnya => "aset_tetap_lainnya",
            Self::KonstruksiDalamPengerjaan => "konstruksi_dalam_pengerjaan",
            Self::AsetTakBerwujud => "aset_tak_berwujud",
            Self::Persediaan => "persediaan",
        }
    }
}

impl std::fmt::Display for AssetCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
