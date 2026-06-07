use super::*;

#[test]
fn test_status_from_code() {
    assert_eq!(
        PemakaianBmnStatus::from_code(3000),
        Some(PemakaianBmnStatus::Draft)
    );
    assert_eq!(
        PemakaianBmnStatus::from_code(3004),
        Some(PemakaianBmnStatus::Active)
    );
    assert_eq!(PemakaianBmnStatus::from_code(9999), None);
}

#[test]
fn test_status_transitions() {
    let draft = PemakaianBmnStatus::Draft;
    assert!(draft.can_transition_to(PemakaianBmnStatus::Submitted));
    assert!(!draft.can_transition_to(PemakaianBmnStatus::Active));

    let active = PemakaianBmnStatus::Active;
    assert!(active.can_transition_to(PemakaianBmnStatus::Expired));
    assert!(active.can_transition_to(PemakaianBmnStatus::Revoked));
    assert!(!active.can_transition_to(PemakaianBmnStatus::Draft));
}

/// V035 (Fase 1.5): alur internal-satker 3-step.
/// Draft → Submitted (Operator) → SubmittedApproverSatker (Validator)
/// → Approved (Approver Satker) → Active.
/// RevisiOperator pintu balik dari Validator atau Approver.
#[test]
fn test_status_transitions_satker_3step() {
    use PemakaianBmnStatus::*;
    // Operator submit
    assert!(Draft.can_transition_to(Submitted));
    // Validator Satker forward → ApproverSatker
    assert!(Submitted.can_transition_to(SubmittedApproverSatker));
    // Validator Satker return → RevisiOperator
    assert!(Submitted.can_transition_to(RevisiOperator));
    // Approver Satker approve
    assert!(SubmittedApproverSatker.can_transition_to(Approved));
    // Approver Satker return ke Operator
    assert!(SubmittedApproverSatker.can_transition_to(RevisiOperator));
    // Operator re-submit
    assert!(RevisiOperator.can_transition_to(Submitted));
    // RevisiOperator dapat di-cancel
    assert!(RevisiOperator.can_transition_to(Cancelled));
    // Skip illegal: Draft langsung ke Approved tidak boleh
    assert!(!Draft.can_transition_to(Approved));
    assert!(!Draft.can_transition_to(SubmittedApproverSatker));
    // SubmittedApproverSatker tidak boleh langsung Active (harus via Approved → activate)
    assert!(!SubmittedApproverSatker.can_transition_to(Active));
    // RevisiOperator tidak boleh langsung lompat ke ApproverSatker
    assert!(!RevisiOperator.can_transition_to(SubmittedApproverSatker));
}

/// Helper utk test logic overlap di repository — direkstrak agar
/// dapat di-unit-test tanpa DB. Match dgn implementasi di
/// `check_bmn_availability_for_period` (repository.rs).
fn classify(
    existing: &[(chrono::NaiveDate, chrono::NaiveDate, &str)],
    new_start: chrono::NaiveDate,
    new_end: chrono::NaiveDate,
) -> &'static str {
    let mut latest_end: Option<chrono::NaiveDate> = None;
    for (s, e, _) in existing {
        let overlap = !(*e < new_start || *s > new_end);
        if overlap {
            return "Overlap";
        }
        if latest_end.map(|prev| *e > prev).unwrap_or(true) {
            latest_end = Some(*e);
        }
    }
    if latest_end.is_some() {
        "PemakaianBerurutan"
    } else {
        "Available"
    }
}

#[test]
fn bmn_check_status_available_when_no_existing() {
    let s = classify(
        &[],
        chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
        chrono::NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
    );
    assert_eq!(s, "Available");
}

#[test]
fn bmn_check_status_sequential_when_existing_ends_before_new_start() {
    // Existing 2026-01-01 .. 2026-06-02, new starts 2026-06-03 → sequential OK.
    let s = classify(
        &[(
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2026, 6, 2).unwrap(),
            "pegawai_lain",
        )],
        chrono::NaiveDate::from_ymd_opt(2026, 6, 3).unwrap(),
        chrono::NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
    );
    assert_eq!(s, "PemakaianBerurutan");
}

#[test]
fn bmn_check_status_overlap_when_periods_intersect() {
    // Existing 2026-01-01 .. 2026-06-30, new 2026-06-15 .. 2026-12-31 → overlap.
    let s = classify(
        &[(
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap(),
            "pegawai_lain",
        )],
        chrono::NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
        chrono::NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
    );
    assert_eq!(s, "Overlap");
}

#[test]
fn bmn_check_status_overlap_when_new_inside_existing() {
    // Existing 2026-01-01 .. 2026-12-31, new 2026-05-01 .. 2026-06-30
    // (entirely inside) → overlap.
    let s = classify(
        &[(
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
            "pegawai_lain",
        )],
        chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(),
        chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap(),
    );
    assert_eq!(s, "Overlap");
}

#[test]
fn bmn_check_status_overlap_when_edge_equal() {
    // Existing ends 2026-06-02, new starts 2026-06-02 — same day = overlap
    // (sequential berarti new_start STRICTLY > existing_end).
    let s = classify(
        &[(
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2026, 6, 2).unwrap(),
            "pegawai_lain",
        )],
        chrono::NaiveDate::from_ymd_opt(2026, 6, 2).unwrap(),
        chrono::NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
    );
    assert_eq!(s, "Overlap");
}

#[test]
fn test_state_name_conversion() {
    assert_eq!(PemakaianBmnStatus::Draft.to_state_name(), "DRAFT");
    assert_eq!(PemakaianBmnStatus::Active.to_state_name(), "ACTIVE");

    assert_eq!(
        PemakaianBmnStatus::from_state_name("DRAFT"),
        Some(PemakaianBmnStatus::Draft)
    );
    assert_eq!(PemakaianBmnStatus::from_state_name("INVALID"), None);
}

#[test]
fn test_jenis_bmn_required_fields() {
    let kendaraan = JenisBmn::KendaraanBermotor;
    assert!(kendaraan.required_fields().contains(&"no_polisi"));

    let rumah = JenisBmn::RumahNegara;
    assert!(rumah.required_fields().contains(&"alamat"));

    let laptop = JenisBmn::Laptop;
    assert!(laptop.required_fields().contains(&"serial_number"));
}
