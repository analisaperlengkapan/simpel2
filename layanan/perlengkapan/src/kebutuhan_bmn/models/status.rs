use serde::{Deserialize, Serialize};

// ============================================================================
// Workflow Status Enum
// ============================================================================

/// Workflow status codes for BMN needs requests
///
/// Flow:
///   Draft(2000) → InputBarang(2001) → SubmitWilayah(2002) → SubmitPusat(2004) → AnalisisKelayakan(2005) → Approved(2006)/Rejected(2007) → Completed(2008)
///   Validator Wilayah can return to RevisiSatker(2003)
///   Validator Pusat approves or rejects (no revision back)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
#[derive(Default)]
pub enum KebutuhanBmnStatus {
    /// New request in draft state (Validator Pusat creates period)
    #[default]
    Draft = 2000,
    /// Operator Satker inputting goods + attachments
    InputBarang = 2001,
    /// Operator Satker submitted to Validator Wilayah
    SubmitWilayah = 2002,
    /// Returned to Operator Satker for revision (by Validator Wilayah)
    RevisiSatker = 2003,
    /// Validator Wilayah forwarded to Validator Pusat
    SubmitPusat = 2004,
    /// Validator Pusat analyzing feasibility (SIMAN + MySIMKARI data)
    AnalisisKelayakan = 2005,
    /// Request approved by Validator Pusat
    Approved = 2006,
    /// Request rejected by Validator Pusat
    Rejected = 2007,
    /// Process completed
    Completed = 2008,
    /// Request cancelled
    Cancelled = 2009,
    /// Returned to Validator Wilayah by Validator Pusat for corrections
    RevisiWilayah = 2010,
}
impl KebutuhanBmnStatus {
    /// Convert from database integer code
    pub fn from_code(code: i32) -> Option<Self> {
        match code {
            2000 => Some(Self::Draft),
            2001 => Some(Self::InputBarang),
            2002 => Some(Self::SubmitWilayah),
            2003 => Some(Self::RevisiSatker),
            2004 => Some(Self::SubmitPusat),
            2005 => Some(Self::AnalisisKelayakan),
            2006 => Some(Self::Approved),
            2007 => Some(Self::Rejected),
            2008 => Some(Self::Completed),
            2009 => Some(Self::Cancelled),
            2010 => Some(Self::RevisiWilayah),
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
            Self::InputBarang => "Input Barang",
            Self::SubmitWilayah => "Diajukan ke Validator Wilayah",
            Self::RevisiSatker => "Revisi Satker",
            Self::SubmitPusat => "Diajukan ke Validator Pusat",
            Self::AnalisisKelayakan => "Analisis Kelayakan",
            Self::Approved => "Disetujui",
            Self::Rejected => "Ditolak",
            Self::Completed => "Selesai",
            Self::Cancelled => "Dibatalkan",
            Self::RevisiWilayah => "Revisi Wilayah",
        }
    }

    /// Check if transition to target status is allowed
    ///
    /// Flow: Draft → InputBarang → SubmitWilayah → SubmitPusat → AnalisisKelayakan → Approved/Rejected
    /// Validator Wilayah: SubmitWilayah → SubmitPusat (forward) or RevisiSatker (return)
    /// Validator Pusat: AnalisisKelayakan → Approved or Rejected (NO revision back)
    pub fn can_transition_to(&self, target: Self) -> bool {
        use KebutuhanBmnStatus::*;
        match self {
            Draft => matches!(target, InputBarang | Cancelled),
            InputBarang => matches!(target, SubmitWilayah | Cancelled),
            SubmitWilayah => matches!(target, SubmitPusat | RevisiSatker),
            RevisiSatker => matches!(target, SubmitWilayah | Cancelled),
            SubmitPusat => matches!(target, AnalisisKelayakan),
            AnalisisKelayakan => matches!(target, Approved | Rejected | RevisiWilayah),
            RevisiWilayah => matches!(target, SubmitPusat),
            Approved => matches!(target, Completed),
            Rejected | Completed | Cancelled => false,
        }
    }

    /// Get allowed next statuses for workflow UI
    pub fn allowed_transitions(&self) -> Vec<Self> {
        use KebutuhanBmnStatus::*;
        match self {
            Draft => vec![InputBarang, Cancelled],
            InputBarang => vec![SubmitWilayah, Cancelled],
            SubmitWilayah => vec![SubmitPusat, RevisiSatker],
            RevisiSatker => vec![SubmitWilayah, Cancelled],
            SubmitPusat => vec![AnalisisKelayakan],
            AnalisisKelayakan => vec![Approved, Rejected, RevisiWilayah],
            RevisiWilayah => vec![SubmitPusat],
            Approved => vec![Completed],
            Rejected | Completed | Cancelled => vec![],
        }
    }

    /// V029 (#24): apakah satker sudah MELEWATI tahap submit operator —
    /// yaitu operator tidak lagi mengedit barang. Pada state ini analisis
    /// kelayakan dibaca dari snapshot beku (bukan SIMAN live) agar Validator
    /// Wilayah & Pusat melihat data konsisten dgn operator. State editing
    /// (Draft/InputBarang/RevisiSatker) tetap memakai analisis live.
    pub fn is_post_operator_submit(&self) -> bool {
        matches!(
            self,
            Self::SubmitWilayah
                | Self::SubmitPusat
                | Self::AnalisisKelayakan
                | Self::RevisiWilayah
                | Self::Approved
                | Self::Rejected
                | Self::Completed
        )
    }

    /// Convert status to workflow engine state name
    pub fn to_state_name(&self) -> &'static str {
        match self {
            Self::Draft => "DRAFT",
            Self::InputBarang => "INPUT_BARANG",
            Self::SubmitWilayah => "SUBMIT_WILAYAH",
            Self::RevisiSatker => "REVISI_SATKER",
            Self::SubmitPusat => "SUBMIT_PUSAT",
            Self::AnalisisKelayakan => "ANALISIS_KELAYAKAN",
            Self::Approved => "APPROVED",
            Self::Rejected => "REJECTED",
            Self::Completed => "COMPLETED",
            Self::Cancelled => "CANCELLED",
            Self::RevisiWilayah => "REVISI_WILAYAH",
        }
    }

    /// Convert workflow engine state name to status
    pub fn from_state_name(name: &str) -> Option<Self> {
        match name {
            "DRAFT" => Some(Self::Draft),
            "INPUT_BARANG" => Some(Self::InputBarang),
            "SUBMIT_WILAYAH" => Some(Self::SubmitWilayah),
            "SUBMIT_SATKER" => Some(Self::SubmitWilayah), // backward compat
            "REVISI_SATKER" => Some(Self::RevisiSatker),
            "SUBMIT_PUSAT" => Some(Self::SubmitPusat),
            "ANALISIS_KELAYAKAN" => Some(Self::AnalisisKelayakan),
            "PENYUSUNAN_PRIORITAS" => Some(Self::AnalisisKelayakan), // backward compat
            "APPROVED" => Some(Self::Approved),
            "REJECTED" => Some(Self::Rejected),
            "COMPLETED" => Some(Self::Completed),
            "CANCELLED" => Some(Self::Cancelled),
            "REVISI_WILAYAH" => Some(Self::RevisiWilayah),
            _ => None,
        }
    }
}
// ============================================================================
// Satker Selection Type
// ============================================================================

/// Options for satker selection in a request
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum PilihanSatker {
    #[default]
    Semua,
    Sebagian,
    /// Cakupan satu wilayah Kejaksaan Tinggi — sistem otomatis resolve
    /// semua satker di bawah Kejati tsb dari `integrasi.v_satker_wilayah`
    /// (V029, Fase 1.7).
    Wilayah,
}
impl PilihanSatker {
    // Infallible parse (defaults on unknown) → named ctor, not FromStr.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "sebagian" => Self::Sebagian,
            "wilayah" => Self::Wilayah,
            _ => Self::Semua,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Semua => "semua",
            Self::Sebagian => "sebagian",
            Self::Wilayah => "wilayah",
        }
    }
}
