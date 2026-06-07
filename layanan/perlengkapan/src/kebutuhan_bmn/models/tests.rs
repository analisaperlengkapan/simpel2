use super::*;

#[test]
fn test_status_from_code() {
    assert_eq!(
        KebutuhanBmnStatus::from_code(2000),
        Some(KebutuhanBmnStatus::Draft)
    );
    assert_eq!(
        KebutuhanBmnStatus::from_code(2008),
        Some(KebutuhanBmnStatus::Completed)
    );
    assert_eq!(KebutuhanBmnStatus::from_code(9999), None);
}

#[test]
fn test_status_to_code() {
    assert_eq!(KebutuhanBmnStatus::Draft.to_code(), 2000);
    assert_eq!(KebutuhanBmnStatus::Completed.to_code(), 2008);
}

#[test]
fn test_status_transitions() {
    let draft = KebutuhanBmnStatus::Draft;
    assert!(draft.can_transition_to(KebutuhanBmnStatus::InputBarang));
    assert!(draft.can_transition_to(KebutuhanBmnStatus::Cancelled));
    assert!(!draft.can_transition_to(KebutuhanBmnStatus::Approved));

    // Operator satker submits to wilayah, not directly to pusat
    let input = KebutuhanBmnStatus::InputBarang;
    assert!(input.can_transition_to(KebutuhanBmnStatus::SubmitWilayah));
    assert!(!input.can_transition_to(KebutuhanBmnStatus::SubmitPusat));

    // Validator wilayah can forward or return
    let submit_wil = KebutuhanBmnStatus::SubmitWilayah;
    assert!(submit_wil.can_transition_to(KebutuhanBmnStatus::SubmitPusat));
    assert!(submit_wil.can_transition_to(KebutuhanBmnStatus::RevisiSatker));

    // Validator pusat approves or rejects only
    let analisis = KebutuhanBmnStatus::AnalisisKelayakan;
    assert!(analisis.can_transition_to(KebutuhanBmnStatus::Approved));
    assert!(analisis.can_transition_to(KebutuhanBmnStatus::Rejected));
    assert!(!analisis.can_transition_to(KebutuhanBmnStatus::RevisiSatker));

    let completed = KebutuhanBmnStatus::Completed;
    assert!(!completed.can_transition_to(KebutuhanBmnStatus::Draft));
    assert!(completed.allowed_transitions().is_empty());
}

#[test]
fn test_pilihan_satker() {
    assert_eq!(PilihanSatker::from_str("semua"), PilihanSatker::Semua);
    assert_eq!(PilihanSatker::from_str("sebagian"), PilihanSatker::Sebagian);
    // V029 (Fase 1.7): scope baru "wilayah".
    assert_eq!(PilihanSatker::from_str("wilayah"), PilihanSatker::Wilayah);
    assert_eq!(PilihanSatker::from_str("WILAYAH"), PilihanSatker::Wilayah);
    // Default fallback masih Semua.
    assert_eq!(PilihanSatker::from_str("unknown"), PilihanSatker::Semua);
    assert_eq!(PilihanSatker::Sebagian.as_str(), "sebagian");
    assert_eq!(PilihanSatker::Wilayah.as_str(), "wilayah");
}

#[test]
fn test_status_labels() {
    assert_eq!(KebutuhanBmnStatus::Draft.label(), "Draft");
    assert_eq!(
        KebutuhanBmnStatus::AnalisisKelayakan.label(),
        "Analisis Kelayakan"
    );
}
