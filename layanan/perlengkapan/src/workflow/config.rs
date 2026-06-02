// ============================================================================
// Workflow Configuration Module
// Description: Defines workflow state machines, transition rules, and SLA configuration
// Requirements: REQ-W001, REQ-W002
// ============================================================================

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Workflow configuration defining state transitions and SLA requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConfig {
    /// Name of the workflow
    pub name: String,

    /// Description of the workflow
    pub description: String,

    /// Valid state transitions: from_state -> vec![to_states]
    pub transitions: HashMap<String, Vec<String>>,

    /// SLA in minutes for each state
    pub sla_minutes: HashMap<String, u32>,

    /// Required roles for each state transition
    pub required_roles: HashMap<String, String>,

    /// Whether this workflow supports parallel approvals
    pub supports_parallel_approval: bool,
}

impl WorkflowConfig {
    /// Default workflow configuration for Kebutuhan BMN
    ///
    /// States:
    /// - DRAFT (2000): Initial draft state
    /// - INPUT_BARANG (2001): Operator inputs goods list
    /// - SUBMIT_SATKER (2002): Submitted to validator
    /// - REVISI_SATKER (2003): Returned for revision
    /// - ANALISIS_KELAYAKAN (2004): Feasibility analysis
    /// - PENYUSUNAN_PRIORITAS (2005): Priority setting
    /// - APPROVED (2006): Approved
    /// - REJECTED (2007): Rejected
    /// - COMPLETED (2008): Completed
    /// - CANCELLED (2009): Cancelled
    pub fn default_kebutuhan_bmn() -> Self {
        // V1.1: Source-of-truth state names di-sync dgn
        // `KebutuhanBmnStatus::to_state_name`. Drift legacy yg dihapus:
        //   - SUBMIT_SATKER     → SUBMIT_WILAYAH
        //   - PENYUSUNAN_PRIORITAS → (di-merge ke ANALISIS_KELAYAKAN; tidak ada step terpisah)
        // Backward-compat dipertahankan di `KebutuhanBmnStatus::from_state_name`
        // (alias) — config baru tidak perlu menulis legacy names.
        let mut transitions = HashMap::new();

        // Draft → InputBarang | Cancelled
        transitions.insert(
            "DRAFT".to_string(),
            vec!["INPUT_BARANG".to_string(), "CANCELLED".to_string()],
        );

        // InputBarang → SubmitWilayah | Cancelled
        transitions.insert(
            "INPUT_BARANG".to_string(),
            vec!["SUBMIT_WILAYAH".to_string(), "CANCELLED".to_string()],
        );

        // SubmitWilayah → SubmitPusat (forward) | RevisiSatker (return)
        transitions.insert(
            "SUBMIT_WILAYAH".to_string(),
            vec!["SUBMIT_PUSAT".to_string(), "REVISI_SATKER".to_string()],
        );

        // RevisiSatker → SubmitWilayah (re-submit) | Cancelled
        transitions.insert(
            "REVISI_SATKER".to_string(),
            vec!["SUBMIT_WILAYAH".to_string(), "CANCELLED".to_string()],
        );

        // SubmitPusat → AnalisisKelayakan (sistem buka analisis)
        transitions.insert(
            "SUBMIT_PUSAT".to_string(),
            vec!["ANALISIS_KELAYAKAN".to_string()],
        );

        // AnalisisKelayakan → Approved | Rejected | RevisiWilayah
        transitions.insert(
            "ANALISIS_KELAYAKAN".to_string(),
            vec![
                "APPROVED".to_string(),
                "REJECTED".to_string(),
                "REVISI_WILAYAH".to_string(),
            ],
        );

        // RevisiWilayah → SubmitPusat (Validator Wilayah re-submit after fix)
        transitions.insert(
            "REVISI_WILAYAH".to_string(),
            vec!["SUBMIT_PUSAT".to_string()],
        );

        // Approved → Completed (auto-generate Laporan Hasil Analisis)
        transitions.insert("APPROVED".to_string(), vec!["COMPLETED".to_string()]);

        // Terminal states
        transitions.insert("REJECTED".to_string(), vec![]);
        transitions.insert("COMPLETED".to_string(), vec![]);
        transitions.insert("CANCELLED".to_string(), vec![]);

        // SLA configuration (in minutes)
        let mut sla_minutes = HashMap::new();
        sla_minutes.insert("SUBMIT_WILAYAH".to_string(), 2880); // 2 days
        sla_minutes.insert("SUBMIT_PUSAT".to_string(), 2880); // 2 days
        sla_minutes.insert("ANALISIS_KELAYAKAN".to_string(), 4320); // 3 days
        sla_minutes.insert("APPROVED".to_string(), 4320); // 3 days

        // Required roles for state transitions
        let mut required_roles = HashMap::new();
        required_roles.insert("DRAFT".to_string(), "validator_pusat".to_string()); // create periode
        required_roles.insert("INPUT_BARANG".to_string(), "operator_satker".to_string());
        required_roles.insert("SUBMIT_WILAYAH".to_string(), "operator_satker".to_string());
        required_roles.insert("REVISI_SATKER".to_string(), "operator_satker".to_string());
        required_roles.insert("SUBMIT_PUSAT".to_string(), "validator_wilayah".to_string());
        required_roles.insert(
            "ANALISIS_KELAYAKAN".to_string(),
            "validator_pusat".to_string(),
        );
        required_roles.insert("REVISI_WILAYAH".to_string(), "validator_pusat".to_string());
        required_roles.insert("APPROVED".to_string(), "validator_pusat".to_string());
        required_roles.insert("REJECTED".to_string(), "validator_pusat".to_string());
        required_roles.insert("COMPLETED".to_string(), "admin_pusat".to_string());

        Self {
            name: "kebutuhan_bmn".to_string(),
            description: "Workflow for BMN requirements approval process".to_string(),
            transitions,
            sla_minutes,
            required_roles,
            supports_parallel_approval: true,
        }
    }

    /// Default workflow configuration for Pemakaian BMN (Usage Permits)
    ///
    /// States:
    /// - DRAFT: Initial draft
    /// - SUBMITTED: Submitted for approval
    /// - APPROVED: Approved
    /// - REJECTED: Rejected
    /// - ACTIVE: Permit is active
    /// - EXPIRED: Permit has expired
    /// - REVOKED: Permit was revoked
    pub fn default_pemakaian_bmn() -> Self {
        let mut transitions = HashMap::new();

        // V035 (Fase 1.5): alur internal-satker 3-step.
        // DRAFT → SUBMITTED (Operator submit ke Validator Satker)
        transitions.insert(
            "DRAFT".to_string(),
            vec!["SUBMITTED".to_string(), "CANCELLED".to_string()],
        );

        // SUBMITTED (= menunggu Validator Satker) → ApproverSatker | RevisiOperator.
        // Legacy: SUBMITTED → APPROVED langsung tetap diizinkan utk record
        // lama / fallback override admin; tidak dipakai handler baru.
        transitions.insert(
            "SUBMITTED".to_string(),
            vec![
                "SUBMITTED_APPROVER_SATKER".to_string(),
                "REVISI_OPERATOR".to_string(),
                "APPROVED".to_string(),
                "REJECTED".to_string(),
            ],
        );

        // SUBMITTED_APPROVER_SATKER → APPROVED | RevisiOperator.
        transitions.insert(
            "SUBMITTED_APPROVER_SATKER".to_string(),
            vec!["APPROVED".to_string(), "REVISI_OPERATOR".to_string()],
        );

        // RevisiOperator → re-submit ke ValidatorSatker, atau cancel.
        transitions.insert(
            "REVISI_OPERATOR".to_string(),
            vec!["SUBMITTED".to_string(), "CANCELLED".to_string()],
        );

        // APPROVED → ACTIVE (sistem auto-generate SK Izin & nomor).
        transitions.insert("APPROVED".to_string(), vec!["ACTIVE".to_string()]);

        // ACTIVE → EXPIRED (scheduler) | REVOKED (Approver Satker; admin tidak boleh).
        transitions.insert(
            "ACTIVE".to_string(),
            vec!["EXPIRED".to_string(), "REVOKED".to_string()],
        );

        // Terminal states
        transitions.insert("REJECTED".to_string(), vec![]);
        transitions.insert("EXPIRED".to_string(), vec![]);
        transitions.insert("REVOKED".to_string(), vec![]);
        transitions.insert("CANCELLED".to_string(), vec![]);

        // SLA configuration (in minutes)
        let mut sla_minutes = HashMap::new();
        sla_minutes.insert("SUBMITTED".to_string(), 1440); // 1 hari (Validator Satker)
        sla_minutes.insert("SUBMITTED_APPROVER_SATKER".to_string(), 1440); // 1 hari
        sla_minutes.insert("APPROVED".to_string(), 480); // 8 jam aktivasi

        // Required roles (alur internal satker)
        let mut required_roles = HashMap::new();
        required_roles.insert("SUBMITTED".to_string(), "operator_satker".to_string());
        required_roles.insert(
            "SUBMITTED_APPROVER_SATKER".to_string(),
            "validator_satker".to_string(),
        );
        required_roles.insert(
            "REVISI_OPERATOR".to_string(),
            "validator_satker".to_string(),
        );
        required_roles.insert("APPROVED".to_string(), "approver_satker".to_string());
        required_roles.insert("REJECTED".to_string(), "approver_satker".to_string());
        required_roles.insert("REVOKED".to_string(), "approver_satker".to_string());

        Self {
            name: "pemakaian_bmn".to_string(),
            description: "Workflow for BMN usage permit approval process".to_string(),
            transitions,
            sla_minutes,
            required_roles,
            supports_parallel_approval: false,
        }
    }

    /// Default workflow configuration for Penghapusan BMN (BMN Disposal)
    ///
    /// States:
    /// - DRAFT: Initial draft
    /// - SUBMITTED: Submitted for review
    /// - REVIEWED: Under review
    /// - APPROVED: Approved for disposal
    /// - REJECTED: Rejected
    /// - CANCELLED: Cancelled
    pub fn default_penghapusan_bmn() -> Self {
        // V1.1: state names di-sync dgn `PenghapusanBmnStatus::to_state_name`.
        // Fase 1.9 dual-jalur: SubmitWilayah → KonsepSKWilayahGenerated
        // (kewenangan WILAYAH) ATAU → SubmitPusat (kewenangan PUSAT).
        let mut transitions = HashMap::new();

        // Draft → SubmitWilayah
        transitions.insert("DRAFT".to_string(), vec!["SUBMIT_WILAYAH".to_string()]);

        // SubmitWilayah → SubmitPusat (PUSAT) | KonsepSKWilayahGenerated (WILAYAH) | ReturnedToOperator
        transitions.insert(
            "SUBMIT_WILAYAH".to_string(),
            vec![
                "SUBMIT_PUSAT".to_string(),
                "KONSEP_SK_WILAYAH_GENERATED".to_string(),
                "RETURNED_TO_OPERATOR".to_string(),
            ],
        );

        // ReturnedToOperator → SubmitWilayah (re-submit)
        transitions.insert(
            "RETURNED_TO_OPERATOR".to_string(),
            vec!["SUBMIT_WILAYAH".to_string()],
        );

        // Jalur PUSAT
        transitions.insert(
            "SUBMIT_PUSAT".to_string(),
            vec!["VERIFIKASI_PUSAT".to_string()],
        );
        transitions.insert(
            "VERIFIKASI_PUSAT".to_string(),
            vec!["KONSEP_SK_GENERATED".to_string(), "REJECTED".to_string()],
        );
        transitions.insert(
            "KONSEP_SK_GENERATED".to_string(),
            vec!["SK_SIGNED".to_string()],
        );
        transitions.insert("SK_SIGNED".to_string(), vec!["COMPLETED".to_string()]);

        // Jalur WILAYAH (Fase 1.9)
        transitions.insert(
            "KONSEP_SK_WILAYAH_GENERATED".to_string(),
            vec!["SK_SIGNED_WILAYAH".to_string()],
        );
        transitions.insert(
            "SK_SIGNED_WILAYAH".to_string(),
            vec!["COMPLETED".to_string()],
        );

        // Terminal states
        transitions.insert("COMPLETED".to_string(), vec![]);
        transitions.insert("REJECTED".to_string(), vec![]);

        // SLA (minutes)
        let mut sla_minutes = HashMap::new();
        sla_minutes.insert("SUBMIT_WILAYAH".to_string(), 2880); // 2 days
        sla_minutes.insert("SUBMIT_PUSAT".to_string(), 2880);
        sla_minutes.insert("VERIFIKASI_PUSAT".to_string(), 4320); // 3 days
        sla_minutes.insert("KONSEP_SK_GENERATED".to_string(), 4320);
        sla_minutes.insert("KONSEP_SK_WILAYAH_GENERATED".to_string(), 4320);

        // Required roles
        let mut required_roles = HashMap::new();
        required_roles.insert("DRAFT".to_string(), "operator_satker".to_string());
        required_roles.insert("SUBMIT_WILAYAH".to_string(), "operator_satker".to_string());
        required_roles.insert(
            "RETURNED_TO_OPERATOR".to_string(),
            "validator_wilayah".to_string(),
        );
        required_roles.insert("SUBMIT_PUSAT".to_string(), "validator_wilayah".to_string());
        required_roles.insert(
            "VERIFIKASI_PUSAT".to_string(),
            "validator_pusat".to_string(),
        );
        required_roles.insert(
            "KONSEP_SK_GENERATED".to_string(),
            "validator_pusat".to_string(),
        );
        required_roles.insert("SK_SIGNED".to_string(), "validator_pusat".to_string());
        required_roles.insert(
            "KONSEP_SK_WILAYAH_GENERATED".to_string(),
            "validator_wilayah".to_string(),
        );
        required_roles.insert(
            "SK_SIGNED_WILAYAH".to_string(),
            "validator_wilayah".to_string(),
        );
        required_roles.insert("REJECTED".to_string(), "validator_pusat".to_string());
        required_roles.insert("COMPLETED".to_string(), "admin_pusat".to_string());

        Self {
            name: "penghapusan_bmn".to_string(),
            description: "Workflow for BMN disposal — dual kewenangan (PUSAT / WILAYAH)"
                .to_string(),
            transitions,
            sla_minutes,
            required_roles,
            supports_parallel_approval: false,
        }
    }

    /// Default workflow configuration for Pakaian Dinas (Official Uniforms)
    ///
    /// States:
    /// - DRAFT (1000): Initial draft
    /// - SUBMIT_WILAYAH (1001): Submitted to Validator Wilayah
    /// - REVISI_PELAKSANA (1003): Returned to operator for revision
    /// - SUBMIT_PUSAT (1004): Submitted to Validator Pusat
    /// - REVISI_WILAYAH (1007): Returned to Validator Wilayah by Validator Pusat
    /// - SELESAI (1008): Completed
    /// - DITOLAK (1006): Rejected
    /// - CANCELLED: Cancelled
    pub fn default_pakaian_dinas() -> Self {
        let mut transitions = HashMap::new();

        // DRAFT can transition to SUBMIT_WILAYAH or CANCELLED
        transitions.insert(
            "DRAFT".to_string(),
            vec!["SUBMIT_WILAYAH".to_string(), "CANCELLED".to_string()],
        );

        // SUBMIT_WILAYAH can transition to SUBMIT_PUSAT or REVISI_PELAKSANA
        transitions.insert(
            "SUBMIT_WILAYAH".to_string(),
            vec!["SUBMIT_PUSAT".to_string(), "REVISI_PELAKSANA".to_string()],
        );

        // REVISI_PELAKSANA returns to SUBMIT_WILAYAH
        transitions.insert(
            "REVISI_PELAKSANA".to_string(),
            vec!["SUBMIT_WILAYAH".to_string()],
        );

        // SUBMIT_PUSAT can transition to SELESAI, REVISI_WILAYAH, or DITOLAK
        transitions.insert(
            "SUBMIT_PUSAT".to_string(),
            vec![
                "SELESAI".to_string(),
                "REVISI_WILAYAH".to_string(),
                "DITOLAK".to_string(),
            ],
        );

        // REVISI_WILAYAH returns to SUBMIT_PUSAT
        transitions.insert(
            "REVISI_WILAYAH".to_string(),
            vec!["SUBMIT_PUSAT".to_string()],
        );

        // Terminal states
        transitions.insert("SELESAI".to_string(), vec![]);
        transitions.insert("DITOLAK".to_string(), vec![]);
        transitions.insert("CANCELLED".to_string(), vec![]);

        // SLA configuration (in minutes)
        let mut sla_minutes = HashMap::new();
        sla_minutes.insert("SUBMIT_WILAYAH".to_string(), 2880); // 2 days
        sla_minutes.insert("SUBMIT_PUSAT".to_string(), 4320); // 3 days
        sla_minutes.insert("REVISI_WILAYAH".to_string(), 2880); // 2 days

        // Required roles
        let mut required_roles = HashMap::new();
        required_roles.insert("DRAFT".to_string(), "operator_satker".to_string());
        required_roles.insert("SUBMIT_WILAYAH".to_string(), "operator_satker".to_string());
        required_roles.insert(
            "REVISI_PELAKSANA".to_string(),
            "validator_wilayah".to_string(),
        );
        required_roles.insert("SUBMIT_PUSAT".to_string(), "validator_wilayah".to_string());
        required_roles.insert("REVISI_WILAYAH".to_string(), "validator_pusat".to_string());
        required_roles.insert("SELESAI".to_string(), "validator_pusat".to_string());
        required_roles.insert("DITOLAK".to_string(), "validator_pusat".to_string());

        Self {
            name: "pakaian_dinas".to_string(),
            description: "Workflow for official uniform distribution approval process".to_string(),
            transitions,
            sla_minutes,
            required_roles,
            supports_parallel_approval: false,
        }
    }

    /// Validate if a transition from one state to another is allowed
    pub fn is_valid_transition(&self, from_state: &str, to_state: &str) -> bool {
        if let Some(allowed_transitions) = self.transitions.get(from_state) {
            allowed_transitions.contains(&to_state.to_string())
        } else {
            false
        }
    }

    /// Get SLA in minutes for a given state
    pub fn get_sla_minutes(&self, state: &str) -> Option<u32> {
        self.sla_minutes.get(state).copied()
    }

    /// Get required role for a state transition
    pub fn get_required_role(&self, to_state: &str) -> Option<&str> {
        self.required_roles.get(to_state).map(|s| s.as_str())
    }

    /// Get all valid next states from current state
    pub fn get_next_states(&self, current_state: &str) -> Vec<String> {
        self.transitions
            .get(current_state)
            .cloned()
            .unwrap_or_default()
    }

    /// Check if a state is terminal (no outgoing transitions)
    pub fn is_terminal_state(&self, state: &str) -> bool {
        self.transitions
            .get(state)
            .map(|transitions| transitions.is_empty())
            .unwrap_or(true)
    }

    /// Validate the workflow configuration
    pub fn validate(&self) -> Result<(), String> {
        // Check that all states in transitions have SLA or are terminal
        for (state, next_states) in &self.transitions {
            if !next_states.is_empty() && !self.sla_minutes.contains_key(state) {
                // Non-terminal states should have SLA
                // (This is a warning, not an error)
            }
        }

        // Check that all required roles are defined for non-terminal states
        for (state, next_states) in &self.transitions {
            if !next_states.is_empty() && !self.required_roles.contains_key(state) {
                return Err(format!(
                    "Non-terminal state '{}' missing required role definition",
                    state
                ));
            }
        }

        Ok(())
    }
}

/// Workflow state codes mapping to database ms_aktivitas_bmn.kode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WorkflowStateCode {
    Draft = 2000,
    InputBarang = 2001,
    SubmitSatker = 2002,
    RevisiSatker = 2003,
    AnalisisKelayakan = 2004,
    PenyusunanPrioritas = 2005,
    Approved = 2006,
    Rejected = 2007,
    Completed = 2008,
    Cancelled = 2009,
    RevisiWilayah = 2010,
}

impl WorkflowStateCode {
    /// Convert state code to state name
    pub fn to_state_name(self) -> &'static str {
        match self {
            WorkflowStateCode::Draft => "DRAFT",
            WorkflowStateCode::InputBarang => "INPUT_BARANG",
            WorkflowStateCode::SubmitSatker => "SUBMIT_SATKER",
            WorkflowStateCode::RevisiSatker => "REVISI_SATKER",
            WorkflowStateCode::AnalisisKelayakan => "ANALISIS_KELAYAKAN",
            WorkflowStateCode::PenyusunanPrioritas => "PENYUSUNAN_PRIORITAS",
            WorkflowStateCode::Approved => "APPROVED",
            WorkflowStateCode::Rejected => "REJECTED",
            WorkflowStateCode::Completed => "COMPLETED",
            WorkflowStateCode::Cancelled => "CANCELLED",
            WorkflowStateCode::RevisiWilayah => "REVISI_WILAYAH",
        }
    }

    /// Convert state name to state code
    pub fn from_state_name(name: &str) -> Option<Self> {
        match name {
            "DRAFT" => Some(WorkflowStateCode::Draft),
            "INPUT_BARANG" => Some(WorkflowStateCode::InputBarang),
            "SUBMIT_SATKER" => Some(WorkflowStateCode::SubmitSatker),
            "REVISI_SATKER" => Some(WorkflowStateCode::RevisiSatker),
            "ANALISIS_KELAYAKAN" => Some(WorkflowStateCode::AnalisisKelayakan),
            "PENYUSUNAN_PRIORITAS" => Some(WorkflowStateCode::PenyusunanPrioritas),
            "APPROVED" => Some(WorkflowStateCode::Approved),
            "REJECTED" => Some(WorkflowStateCode::Rejected),
            "COMPLETED" => Some(WorkflowStateCode::Completed),
            "CANCELLED" => Some(WorkflowStateCode::Cancelled),
            "REVISI_WILAYAH" => Some(WorkflowStateCode::RevisiWilayah),
            _ => None,
        }
    }

    /// Get the integer code value
    pub fn code(self) -> i32 {
        self as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_kebutuhan_bmn_workflow() {
        let config = WorkflowConfig::default_kebutuhan_bmn();

        // V1.1: state names di-sync dgn enum baru.
        assert!(config.is_valid_transition("DRAFT", "INPUT_BARANG"));
        assert!(config.is_valid_transition("INPUT_BARANG", "SUBMIT_WILAYAH"));
        assert!(config.is_valid_transition("SUBMIT_WILAYAH", "SUBMIT_PUSAT"));
        assert!(config.is_valid_transition("SUBMIT_WILAYAH", "REVISI_SATKER"));
        assert!(config.is_valid_transition("SUBMIT_PUSAT", "ANALISIS_KELAYAKAN"));
        assert!(config.is_valid_transition("ANALISIS_KELAYAKAN", "APPROVED"));
        assert!(config.is_valid_transition("APPROVED", "COMPLETED"));

        // Invalid skip / dari terminal
        assert!(!config.is_valid_transition("DRAFT", "APPROVED"));
        assert!(!config.is_valid_transition("DRAFT", "ANALISIS_KELAYAKAN"));
        assert!(!config.is_valid_transition("REJECTED", "APPROVED"));
        assert!(!config.is_valid_transition("COMPLETED", "DRAFT"));

        // SLA
        assert_eq!(config.get_sla_minutes("SUBMIT_WILAYAH"), Some(2880));
        assert_eq!(config.get_sla_minutes("ANALISIS_KELAYAKAN"), Some(4320));

        // Required roles
        assert_eq!(
            config.get_required_role("SUBMIT_WILAYAH"),
            Some("operator_satker")
        );
        assert_eq!(
            config.get_required_role("ANALISIS_KELAYAKAN"),
            Some("validator_pusat")
        );

        // Terminal
        assert!(config.is_terminal_state("REJECTED"));
        assert!(config.is_terminal_state("CANCELLED"));
        assert!(config.is_terminal_state("COMPLETED"));
        assert!(!config.is_terminal_state("DRAFT"));
    }

    /// V1.1 (Fase 1.1) Convergence test: untuk setiap pasangan state
    /// di enum `KebutuhanBmnStatus`, hasil `enum.can_transition_to` HARUS
    /// match `config.is_valid_transition`. Mencegah drift di masa depan.
    #[test]
    fn convergence_kebutuhan_bmn_enum_vs_config() {
        use crate::kebutuhan_bmn::models::KebutuhanBmnStatus as S;
        let config = WorkflowConfig::default_kebutuhan_bmn();
        let states = [
            S::Draft,
            S::InputBarang,
            S::SubmitWilayah,
            S::RevisiSatker,
            S::SubmitPusat,
            S::AnalisisKelayakan,
            S::Approved,
            S::Rejected,
            S::Completed,
            S::Cancelled,
            S::RevisiWilayah,
        ];
        for from in states.iter() {
            for to in states.iter() {
                let enum_says = from.can_transition_to(*to);
                let config_says =
                    config.is_valid_transition(from.to_state_name(), to.to_state_name());
                assert_eq!(
                    enum_says,
                    config_says,
                    "Drift! enum {}->{}: enum.can_transition_to={}, config.is_valid_transition={}",
                    from.to_state_name(),
                    to.to_state_name(),
                    enum_says,
                    config_says
                );
            }
        }
    }

    /// V1.1 Convergence test untuk Pemakaian BMN.
    #[test]
    fn convergence_pemakaian_bmn_enum_vs_config() {
        use crate::pemakaian_bmn::models::PemakaianBmnStatus as S;
        let config = WorkflowConfig::default_pemakaian_bmn();
        let states = [
            S::Draft,
            S::Submitted,
            S::SubmittedApproverSatker,
            S::RevisiOperator,
            S::Approved,
            S::Rejected,
            S::Active,
            S::Expired,
            S::Revoked,
            S::Cancelled,
        ];
        for from in states.iter() {
            for to in states.iter() {
                let enum_says = from.can_transition_to(*to);
                let config_says =
                    config.is_valid_transition(from.to_state_name(), to.to_state_name());
                assert_eq!(
                    enum_says,
                    config_says,
                    "Drift! enum {}->{}: enum.can_transition_to={}, config.is_valid_transition={}",
                    from.to_state_name(),
                    to.to_state_name(),
                    enum_says,
                    config_says
                );
            }
        }
    }

    /// V1.1 Convergence test untuk Penghapusan BMN (dual jalur SK).
    #[test]
    fn convergence_penghapusan_bmn_enum_vs_config() {
        use crate::penghapusan_bmn::models::PenghapusanBmnStatus as S;
        let config = WorkflowConfig::default_penghapusan_bmn();
        let states = [
            S::Draft,
            S::SubmitWilayah,
            S::ReturnedToOperator,
            S::SubmitPusat,
            S::VerifikasiPusat,
            S::KonsepSKGenerated,
            S::SKSigned,
            S::Completed,
            S::Rejected,
            S::KonsepSKWilayahGenerated,
            S::SKSignedWilayah,
        ];
        for from in states.iter() {
            for to in states.iter() {
                let enum_says = from.can_transition_to(*to);
                let config_says =
                    config.is_valid_transition(from.to_state_name(), to.to_state_name());
                assert_eq!(
                    enum_says,
                    config_says,
                    "Drift! enum {}->{}: enum.can_transition_to={}, config.is_valid_transition={}",
                    from.to_state_name(),
                    to.to_state_name(),
                    enum_says,
                    config_says
                );
            }
        }
    }

    #[test]
    fn test_default_pemakaian_bmn_workflow() {
        let config = WorkflowConfig::default_pemakaian_bmn();

        // Alur baru (V035 Fase 1.5): Operator → ValidatorSatker → ApproverSatker.
        assert!(config.is_valid_transition("DRAFT", "SUBMITTED"));
        assert!(config.is_valid_transition("SUBMITTED", "SUBMITTED_APPROVER_SATKER"));
        assert!(config.is_valid_transition("SUBMITTED", "REVISI_OPERATOR"));
        assert!(config.is_valid_transition("SUBMITTED_APPROVER_SATKER", "APPROVED"));
        assert!(config.is_valid_transition("SUBMITTED_APPROVER_SATKER", "REVISI_OPERATOR"));
        assert!(config.is_valid_transition("REVISI_OPERATOR", "SUBMITTED"));
        assert!(config.is_valid_transition("APPROVED", "ACTIVE"));
        // Legacy direct path tetap valid utk back-compat record lama.
        assert!(config.is_valid_transition("SUBMITTED", "APPROVED"));

        // Invalid: skip step / dari terminal
        assert!(!config.is_valid_transition("DRAFT", "ACTIVE"));
        assert!(!config.is_valid_transition("DRAFT", "SUBMITTED_APPROVER_SATKER"));
        assert!(!config.is_valid_transition("EXPIRED", "ACTIVE"));
        assert!(!config.is_valid_transition("SUBMITTED_APPROVER_SATKER", "ACTIVE"));
    }

    #[test]
    fn test_workflow_state_code_conversion() {
        assert_eq!(WorkflowStateCode::Draft.code(), 2000);
        assert_eq!(WorkflowStateCode::Approved.code(), 2006);

        assert_eq!(WorkflowStateCode::Draft.to_state_name(), "DRAFT");
        assert_eq!(WorkflowStateCode::Approved.to_state_name(), "APPROVED");

        assert_eq!(
            WorkflowStateCode::from_state_name("DRAFT"),
            Some(WorkflowStateCode::Draft)
        );
        assert_eq!(WorkflowStateCode::from_state_name("INVALID"), None);
    }

    #[test]
    fn test_get_next_states() {
        let config = WorkflowConfig::default_kebutuhan_bmn();

        let next_states = config.get_next_states("DRAFT");
        assert_eq!(next_states.len(), 2);
        assert!(next_states.contains(&"INPUT_BARANG".to_string()));
        assert!(next_states.contains(&"CANCELLED".to_string()));

        let terminal_states = config.get_next_states("REJECTED");
        assert_eq!(terminal_states.len(), 0);
    }

    #[test]
    fn test_workflow_validation() {
        let config = WorkflowConfig::default_kebutuhan_bmn();
        assert!(config.validate().is_ok());
    }
}
