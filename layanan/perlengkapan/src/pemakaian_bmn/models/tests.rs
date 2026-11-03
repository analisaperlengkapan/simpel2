use super::*;
use uuid::Uuid;

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

/// Setiap status yang bisa tersimpan di kolom harus punya label sendiri.
///
/// Ini yang dulu tidak dijaga: salinan kosakata di frontend melewatkan
/// `SUBMITTED_APPROVER_SATKER` dan `REVISI_OPERATOR`, jadi izin yang sedang
/// menunggu Approver Satker — langkah tengah dari seluruh alur V035 — tampil
/// sebagai "Lainnya".
#[test]
fn every_status_names_itself_and_no_two_share_a_label() {
    let all = [
        PemakaianBmnStatus::Draft,
        PemakaianBmnStatus::Submitted,
        PemakaianBmnStatus::SubmittedApproverSatker,
        PemakaianBmnStatus::RevisiOperator,
        PemakaianBmnStatus::Approved,
        PemakaianBmnStatus::Rejected,
        PemakaianBmnStatus::Active,
        PemakaianBmnStatus::Expired,
        PemakaianBmnStatus::Revoked,
        PemakaianBmnStatus::Cancelled,
    ];

    let mut labels: Vec<&str> = Vec::new();
    for status in all {
        let (label, tone) = PemakaianBmnStatus::describe(status.to_state_name());
        assert_eq!(
            label,
            status.label(),
            "describe() harus memakai label enum untuk {:?}",
            status
        );
        assert_ne!(
            label,
            status.to_state_name(),
            "{:?} masih menampilkan token mesin, bukan label",
            status
        );
        assert_eq!(tone, status.tone());
        labels.push(status.label());
    }

    labels.sort_unstable();
    let unique = {
        let mut l = labels.clone();
        l.dedup();
        l.len()
    };
    assert_eq!(unique, labels.len(), "dua status berbagi label: {labels:?}");
}

/// Kanari arah sebaliknya: state di luar enum tidak boleh diberi nama karangan.
/// Menampilkan nilainya apa adanya jauh lebih menolong daripada "Lainnya",
/// yang menyembunyikan justru hal yang perlu dilihat.
#[test]
fn an_unknown_state_is_shown_verbatim_not_renamed() {
    let (label, tone) = PemakaianBmnStatus::describe("BELUM_ADA_DI_ENUM");
    assert_eq!(label, "BELUM_ADA_DI_ENUM");
    assert_eq!(tone, crate::shared::status_tone::StatusTone::Neutral);
}

// ── The holder of a permit ───────────────────────────────────────────────

fn rekam() -> PegawaiRekam {
    PegawaiRekam {
        info: PegawaiInfo {
            nip: "19800101000000001".into(),
            nama: Some("  Pegawai Uji A ".into()),
            jabatan: Some("Staf".into()),
            pangkat: Some("Jaksa Madya / (IV/a)".into()),
            satker_id: Some("SKR001".into()),
            nama_satker: Some("KEJAKSAAN NEGERI UJI A".into()),
            foto: Some("foto.jpg".into()),
        },
        satker_uuid: Some(Uuid::from_u128(7)),
    }
}

#[test]
fn a_snapshot_copies_what_the_lookup_found() {
    let s = PegawaiSnapshot::dari_rekam(rekam()).unwrap();
    assert_eq!(s.nama, "Pegawai Uji A", "name is trimmed");
    assert_eq!(s.jabatan.as_deref(), Some("Staf"));
    assert_eq!(s.satker_id, Uuid::from_u128(7));
    assert_eq!(s.satker_nama, "KEJAKSAAN NEGERI UJI A");
    assert_eq!(s.unit_kerja.as_deref(), Some("KEJAKSAAN NEGERI UJI A"));
    assert_eq!(s.foto.as_deref(), Some("foto.jpg"));
}

/// `golpang` holds rank and grade in one inconsistently-shaped string (187 of
/// 21 328 are reversed), so it is stored whole as pangkat and never split.
#[test]
fn rank_and_grade_are_not_guessed_apart() {
    let s = PegawaiSnapshot::dari_rekam(rekam()).unwrap();
    assert_eq!(s.pangkat.as_deref(), Some("Jaksa Madya / (IV/a)"));
    assert_eq!(s.golongan, None);
}

#[test]
fn an_employee_with_no_name_cannot_hold_a_permit() {
    for nama in [None, Some(String::new()), Some("   ".to_string())] {
        let mut r = rekam();
        r.info.nama = nama;
        assert!(PegawaiSnapshot::dari_rekam(r).is_err());
    }
}

#[test]
fn an_employee_whose_satker_did_not_resolve_cannot_hold_a_permit() {
    let mut r = rekam();
    r.satker_uuid = None;
    assert!(PegawaiSnapshot::dari_rekam(r).is_err());
}

/// An absent value stays absent; a blank one must not become an empty string
/// that prints as nothing on the SK instead of the SK's "-".
#[test]
fn blank_optional_fields_become_absent_not_empty() {
    let mut r = rekam();
    r.info.jabatan = Some("  ".into());
    r.info.pangkat = None;
    r.info.foto = Some(String::new());
    let s = PegawaiSnapshot::dari_rekam(r).unwrap();
    assert_eq!(s.jabatan, None);
    assert_eq!(s.pangkat, None);
    assert_eq!(s.foto, None);
}
