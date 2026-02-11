// ============================================================================
// Workflow Configuration Module
// Description: Defines workflow state machines, transition rules, and SLA configuration
// Requirements: REQ-W001, REQ-W002
// ============================================================================

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

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
        let mut transitions = HashMap::new();

        // DRAFT can transition to INPUT_BARANG or CANCELLED
        transitions.insert("DRAFT".to_string(), vec![
            "INPUT_BARANG".to_string(),
            "CANCELLED".to_string(),
        ]);

        // INPUT_BARANG can transition to SUBMIT_SATKER or back to DRAFT
        transitions.insert("INPUT_BARANG".to_string(), vec![
            "SUBMIT_SATKER".to_string(),
            "DRAFT".to_string(),
        ]);

        // SUBMIT_SATKER can transition to ANALISIS_KELAYAKAN or REVISI_SATKER
        transitions.insert("SUBMIT_SATKER".to_string(), vec![
            "ANALISIS_KELAYAKAN".to_string(),
            "REVISI_SATKER".to_string(),
            "REJECTED".to_string(),
        ]);

        // REVISI_SATKER can transition back to INPUT_BARANG
        transitions.insert("REVISI_SATKER".to_string(), vec![
            "INPUT_BARANG".to_string(),
        ]);

        // ANALISIS_KELAYAKAN can transition to PENYUSUNAN_PRIORITAS, REVISI_SATKER, or REJECTED
        transitions.insert("ANALISIS_KELAYAKAN".to_string(), vec![
            "PENYUSUNAN_PRIORITAS".to_string(),
            "REVISI_SATKER".to_string(),
            "REJECTED".to_string(),
        ]);

        // PENYUSUNAN_PRIORITAS can transition to APPROVED or REJECTED
        transitions.insert("PENYUSUNAN_PRIORITAS".to_string(), vec![
            "APPROVED".to_string(),
            "REJECTED".to_string(),
        ]);

        // APPROVED can transition to COMPLETED
        transitions.insert("APPROVED".to_string(), vec![
            "COMPLETED".to_string(),
        ]);

        // COMPLETED can transition to CANCELLED (for rollback scenarios)
        transitions.insert("COMPLETED".to_string(), vec![
            "CANCELLED".to_string(),
        ]);

        // REJECTED is terminal (no transitions)
        transitions.insert("REJECTED".to_string(), vec![]);

        // CANCELLED is terminal (no transitions)
        transitions.insert("CANCELLED".to_string(), vec![]);

        // SLA configuration (in minutes)
        let mut sla_minutes = HashMap::new();
        sla_minutes.insert("SUBMIT_SATKER".to_string(), 2880);           // 2 days (48 hours)
        sla_minutes.insert("ANALISIS_KELAYAKAN".to_string(), 4320);      // 3 days (72 hours)
        sla_minutes.insert("PENYUSUNAN_PRIORITAS".to_string(), 1440);    // 1 day (24 hours)
        sla_minutes.insert("APPROVED".to_string(), 4320);                 // 3 days (72 hours)

        // Required roles for state transitions
        let mut required_roles = HashMap::new();
        required_roles.insert("DRAFT".to_string(), "operator_satker".to_string());
        required_roles.insert("INPUT_BARANG".to_string(), "operator_satker".to_string());
        required_roles.insert("SUBMIT_SATKER".to_string(), "operator_satker".to_string());
        required_roles.insert("ANALISIS_KELAYAKAN".to_string(), "validator_pusat".to_string());
        required_roles.insert("PENYUSUNAN_PRIORITAS".to_string(), "validator_pusat".to_string());
        required_roles.insert("APPROVED".to_string(), "admin_pusat".to_string());
        required_roles.insert("REJECTED".to_string(), "admin_pusat".to_string());
        required_roles.insert("REVISI_SATKER".to_string(), "validator_pusat".to_string());
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

        // DRAFT can transition to SUBMITTED or CANCELLED
        transitions.insert("DRAFT".to_string(), vec![
            "SUBMITTED".to_string(),
            "CANCELLED".to_string(),
        ]);

        // SUBMITTED can transition to APPROVED or REJECTED
        transitions.insert("SUBMITTED".to_string(), vec![
            "APPROVED".to_string(),
            "REJECTED".to_string(),
        ]);

        // APPROVED can transition to ACTIVE
        transitions.insert("APPROVED".to_string(), vec![
            "ACTIVE".to_string(),
        ]);

        // ACTIVE can transition to EXPIRED or REVOKED
        transitions.insert("ACTIVE".to_string(), vec![
            "EXPIRED".to_string(),
            "REVOKED".to_string(),
        ]);

        // Terminal states
        transitions.insert("REJECTED".to_string(), vec![]);
        transitions.insert("EXPIRED".to_string(), vec![]);
        transitions.insert("REVOKED".to_string(), vec![]);
        transitions.insert("CANCELLED".to_string(), vec![]);

        // SLA configuration (in minutes)
        let mut sla_minutes = HashMap::new();
        sla_minutes.insert("SUBMITTED".to_string(), 1440);  // 1 day
        sla_minutes.insert("APPROVED".to_string(), 480);    // 8 hours

        // Required roles
        let mut required_roles = HashMap::new();
        required_roles.insert("SUBMITTED".to_string(), "pegawai".to_string());
        required_roles.insert("APPROVED".to_string(), "pimpinan_satker".to_string());
        required_roles.insert("REJECTED".to_string(), "pimpinan_satker".to_string());
        required_roles.insert("REVOKED".to_string(), "pimpinan_satker".to_string());

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
        let mut transitions = HashMap::new();

        // DRAFT can transition to SUBMITTED or CANCELLED
        transitions.insert("DRAFT".to_string(), vec![
            "SUBMITTED".to_string(),
            "CANCELLED".to_string(),
        ]);

        // SUBMITTED can transition to REVIEWED or REJECTED
        transitions.insert("SUBMITTED".to_string(), vec![
            "REVIEWED".to_string(),
            "REJECTED".to_string(),
        ]);

        // REVIEWED can transition to APPROVED or REJECTED
        transitions.insert("REVIEWED".to_string(), vec![
            "APPROVED".to_string(),
            "REJECTED".to_string(),
        ]);

        // Terminal states
        transitions.insert("APPROVED".to_string(), vec![]);
        transitions.insert("REJECTED".to_string(), vec![]);
        transitions.insert("CANCELLED".to_string(), vec![]);

        // SLA configuration (in minutes)
        let mut sla_minutes = HashMap::new();
        sla_minutes.insert("SUBMITTED".to_string(), 2880);  // 2 days (48 hours)
        sla_minutes.insert("REVIEWED".to_string(), 4320);   // 3 days (72 hours)

        // Required roles
        let mut required_roles = HashMap::new();
        required_roles.insert("DRAFT".to_string(), "operator_satker".to_string());
        required_roles.insert("SUBMITTED".to_string(), "operator_satker".to_string());
        required_roles.insert("REVIEWED".to_string(), "verifikator".to_string());
        required_roles.insert("APPROVED".to_string(), "pimpinan".to_string());
        required_roles.insert("REJECTED".to_string(), "pimpinan".to_string());

        Self {
            name: "penghapusan_bmn".to_string(),
            description: "Workflow for BMN disposal approval process".to_string(),
            transitions,
            sla_minutes,
            required_roles,
            supports_parallel_approval: false,
        }
    }

    /// Default workflow configuration for Pakaian Dinas (Official Uniforms)
    ///
    /// States:
    /// - DRAFT: Initial draft
    /// - SUBMITTED: Submitted for approval
    /// - APPROVED: Approved
    /// - REJECTED: Rejected
    /// - COMPLETED: Rekapitulasi generated and distributed
    /// - CANCELLED: Cancelled
    pub fn default_pakaian_dinas() -> Self {
        let mut transitions = HashMap::new();

        // DRAFT can transition to SUBMITTED or CANCELLED
        transitions.insert("DRAFT".to_string(), vec![
            "SUBMITTED".to_string(),
            "CANCELLED".to_string(),
        ]);

        // SUBMITTED can transition to APPROVED or REJECTED
        transitions.insert("SUBMITTED".to_string(), vec![
            "APPROVED".to_string(),
            "REJECTED".to_string(),
        ]);

        // APPROVED can transition to COMPLETED
        transitions.insert("APPROVED".to_string(), vec![
            "COMPLETED".to_string(),
        ]);

        // Terminal states
        transitions.insert("COMPLETED".to_string(), vec![]);
        transitions.insert("REJECTED".to_string(), vec![]);
        transitions.insert("CANCELLED".to_string(), vec![]);

        // SLA configuration (in minutes)
        let mut sla_minutes = HashMap::new();
        sla_minutes.insert("SUBMITTED".to_string(), 2880);  // 2 days (48 hours)
        sla_minutes.insert("APPROVED".to_string(), 1440);   // 1 day (24 hours)

        // Required roles
        let mut required_roles = HashMap::new();
        required_roles.insert("DRAFT".to_string(), "operator_satker".to_string());
        required_roles.insert("SUBMITTED".to_string(), "operator_satker".to_string());
        required_roles.insert("APPROVED".to_string(), "validator_pusat".to_string());
        required_roles.insert("REJECTED".to_string(), "validator_pusat".to_string());
        required_roles.insert("COMPLETED".to_string(), "admin_pusat".to_string());

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

        // Test valid transitions
        assert!(config.is_valid_transition("DRAFT", "INPUT_BARANG"));
        assert!(config.is_valid_transition("INPUT_BARANG", "SUBMIT_SATKER"));
        assert!(config.is_valid_transition("SUBMIT_SATKER", "ANALISIS_KELAYAKAN"));

        // Test invalid transitions
        assert!(!config.is_valid_transition("DRAFT", "APPROVED"));
        assert!(!config.is_valid_transition("REJECTED", "APPROVED"));

        // Test SLA
        assert_eq!(config.get_sla_minutes("SUBMIT_SATKER"), Some(2880));
        assert_eq!(config.get_sla_minutes("ANALISIS_KELAYAKAN"), Some(4320));

        // Test required roles
        assert_eq!(config.get_required_role("SUBMIT_SATKER"), Some("operator_satker"));
        assert_eq!(config.get_required_role("APPROVED"), Some("admin_pusat"));

        // Test terminal states
        assert!(config.is_terminal_state("REJECTED"));
        assert!(config.is_terminal_state("CANCELLED"));
        assert!(!config.is_terminal_state("DRAFT"));
    }

    #[test]
    fn test_default_pemakaian_bmn_workflow() {
        let config = WorkflowConfig::default_pemakaian_bmn();

        // Test valid transitions
        assert!(config.is_valid_transition("DRAFT", "SUBMITTED"));
        assert!(config.is_valid_transition("SUBMITTED", "APPROVED"));
        assert!(config.is_valid_transition("APPROVED", "ACTIVE"));

        // Test invalid transitions
        assert!(!config.is_valid_transition("DRAFT", "ACTIVE"));
        assert!(!config.is_valid_transition("EXPIRED", "ACTIVE"));
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
        assert_eq!(
            WorkflowStateCode::from_state_name("INVALID"),
            None
        );
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
