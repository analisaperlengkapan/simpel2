use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

// ============================================================================
// Workflow Status Enum
// ============================================================================

/// Workflow status codes for BMN usage permits
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
#[derive(Default)]
pub enum PemakaianBmnStatus {
    /// New permit request in draft state
    #[default]
    Draft = 3000,
    /// Submitted to Validator Satker (internal satker validation step)
    Submitted = 3001,
    /// Approved by Approver Satker (Pengguna Barang Satker)
    Approved = 3002,
    /// Rejected (legacy terminal — alur baru pakai RevisiOperator)
    Rejected = 3003,
    /// Permit is active
    Active = 3004,
    /// Permit has expired
    Expired = 3005,
    /// Permit was revoked
    Revoked = 3006,
    /// Request cancelled
    Cancelled = 3007,
    /// V035 (Fase 1.5): forwarded by Validator Satker, awaiting Approver Satker
    SubmittedApproverSatker = 3010,
    /// V035 (Fase 1.5): returned to Operator for revision (catatan wajib)
    RevisiOperator = 3011,
}
impl PemakaianBmnStatus {
    /// Convert from database integer code
    pub fn from_code(code: i32) -> Option<Self> {
        match code {
            3000 => Some(Self::Draft),
            3001 => Some(Self::Submitted),
            3002 => Some(Self::Approved),
            3003 => Some(Self::Rejected),
            3004 => Some(Self::Active),
            3005 => Some(Self::Expired),
            3006 => Some(Self::Revoked),
            3007 => Some(Self::Cancelled),
            3010 => Some(Self::SubmittedApproverSatker),
            3011 => Some(Self::RevisiOperator),
            _ => None,
        }
    }

    /// Convert to database integer code
    pub fn to_code(self) -> i32 {
        self as i32
    }

    /// Get human-readable label
    pub fn label(&self) -> &'static str {
        match self {
            Self::Draft => "Draft",
            Self::Submitted => "Menunggu Validator Satker",
            Self::SubmittedApproverSatker => "Menunggu Approver Satker",
            Self::RevisiOperator => "Revisi Operator",
            Self::Approved => "Disetujui",
            Self::Rejected => "Ditolak",
            Self::Active => "Aktif",
            Self::Expired => "Kadaluarsa",
            Self::Revoked => "Dicabut",
            Self::Cancelled => "Dibatalkan",
        }
    }

    /// Convert status to workflow engine state name
    pub fn to_state_name(&self) -> &'static str {
        match self {
            Self::Draft => "DRAFT",
            Self::Submitted => "SUBMITTED",
            Self::SubmittedApproverSatker => "SUBMITTED_APPROVER_SATKER",
            Self::RevisiOperator => "REVISI_OPERATOR",
            Self::Approved => "APPROVED",
            Self::Rejected => "REJECTED",
            Self::Active => "ACTIVE",
            Self::Expired => "EXPIRED",
            Self::Revoked => "REVOKED",
            Self::Cancelled => "CANCELLED",
        }
    }

    /// Convert workflow engine state name to status
    pub fn from_state_name(name: &str) -> Option<Self> {
        match name {
            "DRAFT" => Some(Self::Draft),
            "SUBMITTED" => Some(Self::Submitted),
            "SUBMITTED_APPROVER_SATKER" => Some(Self::SubmittedApproverSatker),
            "REVISI_OPERATOR" => Some(Self::RevisiOperator),
            "APPROVED" => Some(Self::Approved),
            "REJECTED" => Some(Self::Rejected),
            "ACTIVE" => Some(Self::Active),
            "EXPIRED" => Some(Self::Expired),
            "REVOKED" => Some(Self::Revoked),
            "CANCELLED" => Some(Self::Cancelled),
            _ => None,
        }
    }

    /// Check if transition to target status is allowed.
    ///
    /// Alur: `Draft → Submitted (ValidatorSatker) → SubmittedApproverSatker
    /// → Approved → Active`. `Submitted/SubmittedApproverSatker → RevisiOperator`
    /// (catatan wajib). `RevisiOperator → Submitted` (re-submit).
    pub fn can_transition_to(&self, target: Self) -> bool {
        use PemakaianBmnStatus::*;
        match self {
            Draft => matches!(target, Submitted | Cancelled),
            // ValidatorSatker → ApproverSatker | RevisiOperator | Rejected.
            Submitted => matches!(target, SubmittedApproverSatker | RevisiOperator | Rejected),
            SubmittedApproverSatker => matches!(target, Approved | RevisiOperator),
            RevisiOperator => matches!(target, Submitted | Cancelled),
            Approved => matches!(target, Active),
            Active => matches!(target, Expired | Revoked),
            Rejected | Expired | Revoked | Cancelled => false,
        }
    }
}
// ============================================================================
// BMN Type Enum
// ============================================================================

/// Types of BMN that can be requested for usage
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum JenisBmn {
    /// Motor vehicle (car, motorcycle)
    KendaraanBermotor,
    /// Government housing
    RumahNegara,
    /// Laptop/computer
    Laptop,
    /// Other equipment
    Lainnya,
}
impl JenisBmn {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::KendaraanBermotor => "KENDARAAN_BERMOTOR",
            Self::RumahNegara => "RUMAH_NEGARA",
            Self::Laptop => "LAPTOP",
            Self::Lainnya => "LAINNYA",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "KENDARAAN_BERMOTOR" => Some(Self::KendaraanBermotor),
            "RUMAH_NEGARA" => Some(Self::RumahNegara),
            "LAPTOP" => Some(Self::Laptop),
            "LAINNYA" => Some(Self::Lainnya),
            _ => None,
        }
    }

    /// Get required fields for this BMN type
    pub fn required_fields(&self) -> Vec<&'static str> {
        match self {
            Self::KendaraanBermotor => vec!["no_polisi", "merk", "tahun"],
            Self::RumahNegara => vec!["alamat", "luas_tanah", "luas_bangunan"],
            Self::Laptop => vec!["merk", "serial_number"],
            Self::Lainnya => vec![],
        }
    }
}
/// Status cek ketersediaan BMN per periode (Fase 1.11).
///
/// `Available`: BMN bebas utk periode yg diminta.
/// `Sequential`: Ada izin aktif tapi berakhir sebelum periode usulan
/// dimulai → diizinkan (operator dpt submit).
/// `Overlap`: Ada izin aktif yg overlap dgn periode usulan → tolak.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BmnCheckStatus {
    Available,
    PemakaianBerurutan {
        existing_holder: String,
        existing_sampai_tgl: NaiveDate,
    },
    Overlap {
        existing_holder: String,
        existing_sampai_tgl: NaiveDate,
    },
}
