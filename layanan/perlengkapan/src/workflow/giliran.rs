//! Whose move it is while an entity sits in a state: the people to tell that work is waiting.
//!
//! An explicit table, not derived from `WorkflowConfig::required_roles`. That map is keyed by the
//! state a role moves INTO and is not reliable as "who is next": entering `SUBMIT_WILAYAH` lists the
//! operator, and entering a revision state lists the role that sends it back, so reading it as "the
//! role whose turn it is" tells an operator to act on the very submission they just made.
//!
//! The table is checked against the configurations (see the tests): a state added to a workflow
//! must be either here or in [`TANPA_GILIRAN`] with a reason, so a new state cannot ship without
//! somebody having decided who hears about it.

/// The roles whose move is pending while `entity_type` is in `state`. Empty when nobody's move is
/// pending (the person who asked is told of the outcome by the engine instead).
pub fn peran_giliran(entity_type: &str, state: &str) -> &'static [&'static str] {
    match (entity_type, state) {
        // Kebutuhan BMN: the campaign opens, the satker fills in and submits, wilayah checks, pusat
        // analyses and decides. A return goes back to whoever sent the work.
        ("kebutuhan_bmn", "INPUT_BARANG") => &["operator_satker"],
        ("kebutuhan_bmn", "SUBMIT_WILAYAH") => &["validator_wilayah"],
        ("kebutuhan_bmn", "REVISI_SATKER") => &["operator_satker"],
        ("kebutuhan_bmn", "SUBMIT_PUSAT") => &["validator_pusat"],
        ("kebutuhan_bmn", "REVISI_WILAYAH") => &["validator_wilayah"],
        ("kebutuhan_bmn", "ANALISIS_KELAYAKAN") => &["validator_pusat"],

        // Pemakaian BMN: the satker's own two-step approval.
        ("pemakaian_bmn", "SUBMITTED") => &["validator_satker"],
        ("pemakaian_bmn", "SUBMITTED_APPROVER_SATKER") => &["approver_satker"],
        ("pemakaian_bmn", "REVISI_OPERATOR") => &["operator_satker"],

        // Penghapusan BMN: two authorities (pusat, wilayah), each with its own SK.
        ("penghapusan_bmn", "SUBMIT_WILAYAH") => &["validator_wilayah"],
        ("penghapusan_bmn", "RETURNED_TO_OPERATOR") => &["operator_satker"],
        ("penghapusan_bmn", "SUBMIT_PUSAT") => &["validator_pusat"],
        ("penghapusan_bmn", "VERIFIKASI_PUSAT") => &["validator_pusat"],
        ("penghapusan_bmn", "KONSEP_SK_GENERATED") => &["validator_pusat"],
        ("penghapusan_bmn", "KONSEP_SK_WILAYAH_GENERATED") => &["validator_wilayah"],

        _ => &[],
    }
}

/// Roles that act on a satker's own record: telling them needs the satker's code.
pub fn peran_terikat_satker(peran: &str) -> bool {
    matches!(
        peran,
        "operator_satker" | "validator_satker" | "approver_satker" | "validator_wilayah"
    )
}

/// Non-terminal states where nobody's move is pending, and why. Anything else a workflow can sit
/// in must have an entry in [`peran_giliran`].
#[cfg(test)]
pub(crate) const TANPA_GILIRAN: &[(&str, &str, &str)] = &[
    (
        "kebutuhan_bmn",
        "DRAFT",
        "pusat is still composing the campaign",
    ),
    (
        "kebutuhan_bmn",
        "APPROVED",
        "pusat approved it and completes it; the requester is told of the approval",
    ),
    (
        "pemakaian_bmn",
        "DRAFT",
        "the operator is still composing it",
    ),
    (
        "pemakaian_bmn",
        "APPROVED",
        "activation is the system's step, the requester is told of the approval",
    ),
    (
        "pemakaian_bmn",
        "ACTIVE",
        "in force; expiry is announced by the expiry reminder",
    ),
    (
        "penghapusan_bmn",
        "DRAFT",
        "the operator is still composing it",
    ),
    (
        "penghapusan_bmn",
        "SK_SIGNED",
        "uploading the signed SK completes the usulan in the same request, as the system",
    ),
    (
        "penghapusan_bmn",
        "SK_SIGNED_WILAYAH",
        "uploading the signed SK completes the usulan in the same request, as the system",
    ),
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::config::WorkflowConfig;

    fn configs() -> Vec<(&'static str, WorkflowConfig)> {
        vec![
            ("kebutuhan_bmn", WorkflowConfig::default_kebutuhan_bmn()),
            ("pemakaian_bmn", WorkflowConfig::default_pemakaian_bmn()),
            ("penghapusan_bmn", WorkflowConfig::default_penghapusan_bmn()),
        ]
    }

    /// Derived from the configurations, not enumerated here: a state a workflow gains must be
    /// either given a turn or excused, so it cannot ship with nobody told.
    #[test]
    fn every_state_a_workflow_can_wait_in_has_a_turn_or_a_reason() {
        let mut unaccounted = Vec::new();
        for (name, config) in configs() {
            for (state, next) in &config.transitions {
                if next.is_empty() {
                    continue; // terminal
                }
                let excused = TANPA_GILIRAN
                    .iter()
                    .any(|(n, s, _)| *n == name && *s == state.as_str());
                if peran_giliran(name, state).is_empty() && !excused {
                    unaccounted.push(format!("{name}/{state}"));
                }
            }
        }
        unaccounted.sort();
        assert!(
            unaccounted.is_empty(),
            "states with no role to tell and no excuse in TANPA_GILIRAN: {unaccounted:?}"
        );
    }

    /// And the table cannot name a state or a role that does not exist.
    #[test]
    fn the_table_names_only_real_states_and_real_roles() {
        let known_roles = [
            "operator_satker",
            "validator_satker",
            "approver_satker",
            "validator_wilayah",
            "validator_pusat",
        ];
        for (name, config) in configs() {
            for state in config.transitions.keys() {
                for role in peran_giliran(name, state) {
                    assert!(
                        known_roles.contains(role),
                        "{name}/{state}: unknown role {role}"
                    );
                }
            }
        }
        // A state in the table but not in its workflow is a typo that would notify nobody.
        for (name, state) in [
            ("kebutuhan_bmn", "INPUT_BARANG"),
            ("kebutuhan_bmn", "SUBMIT_WILAYAH"),
            ("kebutuhan_bmn", "REVISI_SATKER"),
            ("kebutuhan_bmn", "SUBMIT_PUSAT"),
            ("kebutuhan_bmn", "REVISI_WILAYAH"),
            ("kebutuhan_bmn", "ANALISIS_KELAYAKAN"),
            ("pemakaian_bmn", "SUBMITTED"),
            ("pemakaian_bmn", "SUBMITTED_APPROVER_SATKER"),
            ("pemakaian_bmn", "REVISI_OPERATOR"),
            ("penghapusan_bmn", "SUBMIT_WILAYAH"),
            ("penghapusan_bmn", "RETURNED_TO_OPERATOR"),
            ("penghapusan_bmn", "SUBMIT_PUSAT"),
            ("penghapusan_bmn", "VERIFIKASI_PUSAT"),
            ("penghapusan_bmn", "KONSEP_SK_GENERATED"),
            ("penghapusan_bmn", "KONSEP_SK_WILAYAH_GENERATED"),
        ] {
            let (_, config) = configs().into_iter().find(|(n, _)| *n == name).unwrap();
            assert!(
                config.transitions.contains_key(state),
                "{name} has no state {state}"
            );
            assert!(!peran_giliran(name, state).is_empty(), "{name}/{state}");
        }
    }

    #[test]
    fn a_submission_is_told_to_the_next_role_never_back_to_the_sender() {
        assert_eq!(
            peran_giliran("penghapusan_bmn", "SUBMIT_WILAYAH"),
            &["validator_wilayah"]
        );
        assert_eq!(
            peran_giliran("kebutuhan_bmn", "SUBMIT_WILAYAH"),
            &["validator_wilayah"]
        );
        assert_eq!(
            peran_giliran("penghapusan_bmn", "RETURNED_TO_OPERATOR"),
            &["operator_satker"]
        );
        assert!(peran_giliran("penghapusan_bmn", "COMPLETED").is_empty());
        assert!(peran_giliran("modul_yang_belum_ada", "APA_SAJA").is_empty());
    }

    #[test]
    fn roles_bound_to_a_satker_need_its_code_and_pusat_does_not() {
        for r in [
            "operator_satker",
            "validator_satker",
            "approver_satker",
            "validator_wilayah",
        ] {
            assert!(peran_terikat_satker(r), "{r}");
        }
        assert!(!peran_terikat_satker("validator_pusat"));
        assert!(!peran_terikat_satker("admin"));
    }
}
