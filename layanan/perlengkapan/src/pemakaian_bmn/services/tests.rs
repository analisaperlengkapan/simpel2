use crate::pemakaian_bmn::models::*;

#[test]
fn test_status_transitions() {
    let draft = PemakaianBmnStatus::Draft;
    assert!(draft.can_transition_to(PemakaianBmnStatus::Submitted));
    assert!(!draft.can_transition_to(PemakaianBmnStatus::Active));
}
