// ============================================================================
// shared/policy.rs — Workflow Policy / Authorization Layer (Fase 1.2)
// ============================================================================
//
// Fase 0.3 menambahkan `Claims::require_any_role` / `require_role` —
// digunakan inline di handler. Cocok untuk RBAC sederhana, TAPI:
//   • Role yg boleh berbeda per current_state (mis. forward dari SUBMITTED
//     hanya Validator Satker; forward dari SUBMITTED_APPROVER_SATKER hanya
//     Approver Satker). require_any_role tidak tahu state.
//   • Matriks role × action × state tersebar di puluhan handler — sulit
//     diaudit (BPK) & sulit diuji.
//
// Solusi: `WorkflowPolicy` trait + impl per modul. Handler memanggil
// `policy.authorize(&claims, action, current_state)?` dan policy
// menentukan role yg sah berdasar (action, state).
//
// TIDAK ADA bypass admin. Sebelumnya `admin`/`admin_pusat`/`superadmin` lolos
// setiap policy, sehingga administrator IT bisa menyetujui/menolak/menandatangani
// hal yg oleh proses bisnis dialamatkan ke pejabat (Approver Satker, Validator
// Pusat) — melanggar segregation of duties (NIST INCITS 359 SSD; OWASP
// Authorization: least privilege) dan membuat jejak audit tak bermakna
// ("siapa yg menyetujui?" → "admin", selalu). Intervensi darurat lewat jalur
// break-glass yg terpisah, wajib beralasan, dan diaudit (lihat
// `shared::break_glass`).
//
// Migrasi inkremental: handler lama boleh tetap pakai `claims.require_*`;
// handler baru / refactor pindah ke policy. Begitu coverage cukup, kita
// dapat hapus require_* dari handler dan jadikan policy single source.
// ============================================================================

use crate::shared::error::AppError;
use crate::shared::middleware::Claims;

/// Trait untuk policy workflow-spesifik. Setiap modul men-define enum
/// `Action`-nya sendiri (mis. `PemakaianBmnAction`) dan men-implement
/// trait ini untuk memetakan (action, state) → daftar role yg sah.
pub trait WorkflowPolicy {
    /// Enum action modul. Harus `Copy + Debug` agar gampang di-log.
    type Action: Copy + std::fmt::Debug;

    /// Nama modul (dipakai utk pesan error & audit log).
    fn module_name(&self) -> &'static str;

    /// Map (action, current_state) → daftar role yg diizinkan.
    /// `current_state = None` artinya action tidak peduli state
    /// (mis. create draft — belum punya state).
    ///
    /// Return `None` artinya kombinasi action × state tidak valid
    /// secara workflow (mis. approve dari state DRAFT) — handler akan
    /// mendapat error 403 dgn pesan eksplisit.
    fn allowed_roles(
        &self,
        action: Self::Action,
        current_state: Option<&str>,
    ) -> Option<&'static [&'static str]>;

    /// Authorize satu aksi: caller harus memegang SALAH SATU role yg diizinkan
    /// `allowed_roles` untuk (action, state) ini. Tidak ada bypass admin.
    fn authorize(
        &self,
        claims: &Claims,
        action: Self::Action,
        current_state: Option<&str>,
    ) -> Result<(), AppError> {
        self.authorize_as(claims, action, current_state).map(|_| ())
    }

    /// Seperti [`authorize`](Self::authorize), tetapi mengembalikan **role
    /// yang mengotorisasi** aksi itu (lihat [`Claims::acting_role`]) — role
    /// inilah yang harus diteruskan ke workflow engine & dicatat di audit,
    /// bukan role primer caller.
    fn authorize_as(
        &self,
        claims: &Claims,
        action: Self::Action,
        current_state: Option<&str>,
    ) -> Result<String, AppError> {
        let allowed = self.allowed_roles(action, current_state).ok_or_else(|| {
            AppError::Authorization(format!(
                "Aksi {:?} tidak valid pada state '{}' utk modul {}",
                action,
                current_state.unwrap_or("<none>"),
                self.module_name()
            ))
        })?;

        if claims.holds_any_role(allowed) {
            let acting = claims.acting_role(allowed);
            tracing::debug!(
                module = self.module_name(),
                action = ?action,
                state = current_state.unwrap_or("<none>"),
                user_id = %claims.user_id,
                role = %acting,
                "policy: allow"
            );
            Ok(acting)
        } else {
            tracing::info!(
                module = self.module_name(),
                action = ?action,
                state = current_state.unwrap_or("<none>"),
                user_id = %claims.user_id,
                roles = ?claims.roles,
                allowed = ?allowed,
                "policy: deny"
            );
            Err(AppError::Authorization(format!(
                "Akses ditolak: role '{}' tidak diizinkan utk aksi {:?} pada state '{}' (perlu: {})",
                claims.role,
                action,
                current_state.unwrap_or("<none>"),
                allowed.join(", ")
            )))
        }
    }
}

// ============================================================================
// Pemakaian BMN Policy (Fase 1.5 + 1.2 integration)
// ============================================================================

/// Action enum untuk workflow Pemakaian BMN.
///
/// State sumber utk pengecekan diambil dari `IzinPemakaianBmn::status`
/// (string state-name, mis. "SUBMITTED", "SUBMITTED_APPROVER_SATKER").
#[derive(Debug, Clone, Copy)]
pub enum PemakaianBmnAction {
    /// Operator buat draft (state belum ada).
    Create,
    /// Operator submit draft ke Validator Satker.
    Submit,
    /// Validator Satker meneruskan ke Approver Satker.
    ValidatorSatkerForward,
    /// Validator Satker mengembalikan ke Operator utk revisi.
    ValidatorSatkerReturn,
    /// Approver Satker menyetujui (auto-activate).
    ApproverSatkerApprove,
    /// Approver Satker mengembalikan ke Operator utk revisi.
    ApproverSatkerReturn,
    /// Operator re-submit setelah revisi.
    Resubmit,
    /// Approver Satker mencabut izin aktif (Admin TIDAK boleh — sesuai stakeholder).
    Revoke,
    /// Operator update draft permit.
    UpdateDraft,
    /// Operator membatalkan draft / usulan yang sedang direvisi miliknya.
    Cancel,
    /// Menerbitkan nomor izin + SK untuk izin yang sudah disetujui. Normalnya
    /// otomatis saat approve; endpoint manual hanya untuk mengulang aktivasi
    /// yang gagal (mis. SIMAN sedang tidak tersedia).
    Activate,
    /// Membuat konsep surat izin (DOCX/PDF).
    GenerateDocument,
    /// Mengunggah PDF izin yang sudah ditandatangani.
    UploadSigned,
}

impl PemakaianBmnAction {
    /// The action that moves a permit `from` one state `to` another.
    ///
    /// Exists so the detail endpoint can narrow `allowed_transitions` — which
    /// the workflow engine computes from the STATE alone — down to the moves
    /// the CALLER may actually make, by handing each candidate back to
    /// [`WorkflowPolicy::allowed_roles`]. Without it the API would advertise a
    /// button that answers 403, and the FE would have to keep its own copy of
    /// the RBAC table to avoid showing it.
    ///
    /// `None` means no single action performs that move (so nothing is
    /// offered), NOT that it is permitted.
    ///
    /// The two return paths are distinguished by their SOURCE state: only
    /// Validator Satker can be the one returning from `SUBMITTED`, only
    /// Approver Satker from `SUBMITTED_APPROVER_SATKER`.
    pub fn for_transition(from: &str, to: &str) -> Option<Self> {
        use PemakaianBmnAction::*;
        match (from, to) {
            ("DRAFT", "SUBMITTED") => Some(Submit),
            ("SUBMITTED", "SUBMITTED_APPROVER_SATKER") => Some(ValidatorSatkerForward),
            ("SUBMITTED", "REVISI_OPERATOR") => Some(ValidatorSatkerReturn),
            ("SUBMITTED_APPROVER_SATKER", "APPROVED") => Some(ApproverSatkerApprove),
            ("SUBMITTED_APPROVER_SATKER", "REVISI_OPERATOR") => Some(ApproverSatkerReturn),
            ("REVISI_OPERATOR", "SUBMITTED") => Some(Resubmit),
            ("ACTIVE", "REVOKED") => Some(Revoke),
            ("DRAFT" | "REVISI_OPERATOR", "CANCELLED") => Some(Cancel),
            _ => None,
        }
    }

    /// Imperative label for the control that performs this action.
    ///
    /// Lives beside the action rather than in the frontend so the workflow
    /// vocabulary has one home; the FE renders it verbatim.
    pub fn action_label(&self) -> &'static str {
        use PemakaianBmnAction::*;
        match self {
            Create => "Buat Draft",
            Submit => "Ajukan ke Validator Satker",
            ValidatorSatkerForward => "Teruskan ke Approver Satker",
            ValidatorSatkerReturn => "Kembalikan untuk Revisi",
            ApproverSatkerApprove => "Setujui Izin",
            ApproverSatkerReturn => "Kembalikan untuk Revisi",
            Resubmit => "Ajukan Ulang",
            Revoke => "Cabut Izin",
            UpdateDraft => "Simpan Draft",
            Cancel => "Batalkan Usulan",
            Activate => "Terbitkan Izin",
            GenerateDocument => "Buat Konsep Surat",
            UploadSigned => "Unggah PDF Bertanda Tangan",
        }
    }
}

pub struct PemakaianBmnPolicy;

impl WorkflowPolicy for PemakaianBmnPolicy {
    type Action = PemakaianBmnAction;

    fn module_name(&self) -> &'static str {
        "pemakaian_bmn"
    }

    fn allowed_roles(
        &self,
        action: Self::Action,
        current_state: Option<&str>,
    ) -> Option<&'static [&'static str]> {
        use PemakaianBmnAction::*;
        match (action, current_state) {
            // Create & UpdateDraft: tidak peduli state (draft baru / draft yg sama).
            (Create, _) => Some(&["operator_satker"]),
            (UpdateDraft, Some("DRAFT")) => Some(&["operator_satker"]),
            (UpdateDraft, Some("REVISI_OPERATOR")) => Some(&["operator_satker"]),

            // Submit: hanya dari DRAFT.
            (Submit, Some("DRAFT")) => Some(&["operator_satker"]),

            // Validator Satker actions: hanya dari SUBMITTED.
            (ValidatorSatkerForward, Some("SUBMITTED")) => Some(&["validator_satker"]),
            (ValidatorSatkerReturn, Some("SUBMITTED")) => Some(&["validator_satker"]),

            // Approver Satker actions: hanya dari SUBMITTED_APPROVER_SATKER.
            (ApproverSatkerApprove, Some("SUBMITTED_APPROVER_SATKER")) => {
                Some(&["approver_satker"])
            }
            (ApproverSatkerReturn, Some("SUBMITTED_APPROVER_SATKER")) => Some(&["approver_satker"]),

            // Resubmit: hanya dari REVISI_OPERATOR.
            (Resubmit, Some("REVISI_OPERATOR")) => Some(&["operator_satker"]),

            // Revoke: hanya Approver Satker, dan hanya saat ACTIVE.
            // Stakeholder eksplisit: Admin TIDAK boleh revoke. Karena
            // `authorize()` tidak lagi punya bypass admin, cukup tidak
            // mencantumkan admin di sini — dan `allowed_roles` inilah yg
            // dipakai endpoint detail untuk menyaring tombol, sehingga admin
            // pun tidak lagi ditawari "Cabut Izin" yg berujung 403.
            (Revoke, Some("ACTIVE")) => Some(&["approver_satker"]),

            // Operator boleh membatalkan usulannya sendiri selama belum
            // berada di tangan validator/approver.
            (Cancel, Some("DRAFT" | "REVISI_OPERATOR")) => Some(&["operator_satker"]),

            // Aktivasi manual = mengulang penerbitan izin yg sudah disetujui;
            // penerbitan nomor izin adalah tindakan pejabat (Approver Satker).
            (Activate, Some("APPROVED")) => Some(&["approver_satker"]),

            // Dokumen izin dikelola sisi satker yg mengajukan/menerbitkan.
            // Validator Wilayah/Pusat hanya memantau (read-only, mandat
            // stakeholder) — tidak boleh membuat/mengunggah dokumen.
            (GenerateDocument, _) => Some(&["operator_satker", "approver_satker"]),
            (UploadSigned, _) => Some(&["operator_satker", "approver_satker"]),

            // Kombinasi lain → tidak valid (None → 403).
            _ => None,
        }
    }
}

/// Maker-checker (four-eyes) untuk rantai persetujuan internal-satker.
///
/// Orang yang **mengusulkan** tidak boleh **memvalidasi** usulannya sendiri,
/// dan orang yang **menyetujui** tidak boleh sekaligus pengusul atau
/// validatornya. Role yang berbeda tidak cukup: satu akun yang memegang
/// `operator_satker` + `validator_satker` (atau + `approver_satker`) tetap bisa
/// menjalankan seluruh rantai sendirian, dan itu persis kontrol yang dibuat
/// rantai ini untuk dicegah (segregation of duties; OWASP Authorization).
///
/// `created_by` / `validator_satker_id` bernilai `None` untuk record legacy
/// (sebelum V035) — tidak ada pihak untuk dibandingkan, jadi lolos.
pub fn enforce_maker_checker(
    action: PemakaianBmnAction,
    actor: uuid::Uuid,
    created_by: Option<uuid::Uuid>,
    validator_satker_id: Option<uuid::Uuid>,
) -> Result<(), AppError> {
    use PemakaianBmnAction::*;
    let conflict = |who: &str| {
        Err(AppError::Authorization(format!(
            "Pemisahan tugas: {who} tidak boleh {} usulan yang sama (maker-checker)",
            action.action_label().to_lowercase()
        )))
    };
    match action {
        ValidatorSatkerForward | ValidatorSatkerReturn if created_by == Some(actor) => {
            conflict("pengusul")
        }
        ApproverSatkerApprove | ApproverSatkerReturn if created_by == Some(actor) => {
            conflict("pengusul")
        }
        ApproverSatkerApprove | ApproverSatkerReturn if validator_satker_id == Some(actor) => {
            conflict("validator")
        }
        _ => Ok(()),
    }
}

/// Role yg boleh MEMBACA dashboard monitoring Pemakaian BMN.
///
/// Stakeholder (Fase 2.6): Validator Wilayah & Validator Pusat **hanya
/// memantau** pemakaian BMN — read-only, tidak ada aksi approve/revoke.
/// `PemakaianBmnPolicy` sudah memastikan mereka tidak punya satu pun
/// write-action (semua kombinasi → `None` → 403), jadi sisi tulis aman.
/// Konstanta ini melengkapi sisi BACA: hanya audiens monitoring yg sah +
/// peran internal-satker (yg wajar melihat aktivitas satkernya sendiri).
const PEMAKAIAN_MONITORING_ROLES: &[&str] = &[
    // Audiens monitoring lintas-satker (read-only, mandat stakeholder).
    "validator_wilayah",
    "validator_pusat",
    "pusat",
    "analis_pusat",
    // Peran internal-satker — melihat aktivitas pemakaian satkernya sendiri.
    "operator_satker",
    "validator_satker",
    "approver_satker",
];

/// Guard BACA untuk endpoint monitoring Pemakaian BMN (Fase 2.6).
///
/// Endpoint monitoring bersifat agregat (tidak terikat state satu entitas),
/// sehingga tidak lewat `WorkflowPolicy::authorize`. Guard tipis ini cukup:
/// pastikan caller adalah audiens monitoring yg sah. Administrator aplikasi
/// boleh MEMBACA (pemantauan operasional, `Capability::ViewAudit`) — tetapi
/// membaca tidak berarti bisa menyetujui: sisi tulis tidak punya bypass admin.
pub fn enforce_monitoring_read(claims: &Claims) -> Result<(), AppError> {
    claims.require_any_role_or_admin(PEMAKAIAN_MONITORING_ROLES)
}

/// Stakeholder mandate: Admin TIDAK boleh revoke izin pemakaian BMN.
///
/// `PemakaianBmnPolicy::allowed_roles(Revoke, ACTIVE)` sudah tidak memuat admin,
/// jadi seorang admin murni ditolak oleh policy. Guard ini menutup kasus yg
/// tidak bisa ditangkap policy: pemegang **dua** role, `approver_satker` +
/// `admin`, yang lolos policy lewat role approver-nya. Kewenangan ini eksklusif
/// pejabat satker; orang yg juga mengadministrasi sistem tidak boleh
/// menggunakannya. (Dengan `AUTHENC_ACTIVE_ROLE_ENFORCEMENT` aktif, pemegang dua
/// role bertindak dgn SATU role aktif dan kasus ini hilang dgn sendirinya.)
pub fn enforce_no_admin_revoke(claims: &Claims) -> Result<(), AppError> {
    if claims.is_admin() {
        Err(AppError::Authorization(
            "Admin tidak diizinkan mencabut izin pemakaian BMN — kewenangan ini eksklusif milik Approver Satker (sesuai PMK & arahan stakeholder)".into(),
        ))
    } else {
        Ok(())
    }
}

// ============================================================================
// Kebutuhan BMN Policy
// ============================================================================

/// State machine Kebutuhan BMN (kode 2000+):
///   DRAFT → INPUT_BARANG → SUBMIT_WILAYAH → SUBMIT_PUSAT
///   → ANALISIS_KELAYAKAN → APPROVED → COMPLETED
#[derive(Debug, Clone, Copy)]
pub enum KebutuhanBmnAction {
    /// Validator Pusat buat periode RKBMN.
    CreatePeriode,
    /// Operator Satker submit usulan dari satker ke Wilayah.
    SubmitWilayah,
    /// Validator Wilayah forward ke Pusat.
    ForwardPusat,
    /// Validator Wilayah / Pusat kembalikan utk revisi.
    Return,
    /// Validator Pusat approve / reject.
    Decide,
    /// Generate / download Laporan Hasil Analisis.
    GenerateLaporan,
}

pub struct KebutuhanBmnPolicy;

impl WorkflowPolicy for KebutuhanBmnPolicy {
    type Action = KebutuhanBmnAction;
    fn module_name(&self) -> &'static str {
        "kebutuhan_bmn"
    }
    fn allowed_roles(
        &self,
        action: Self::Action,
        current_state: Option<&str>,
    ) -> Option<&'static [&'static str]> {
        use KebutuhanBmnAction::*;
        match (action, current_state) {
            (CreatePeriode, _) => Some(&["validator_pusat"]),
            (SubmitWilayah, Some("INPUT_BARANG")) => Some(&["operator_satker"]),
            (SubmitWilayah, Some("REVISI_SATKER")) => Some(&["operator_satker"]),
            (ForwardPusat, Some("SUBMIT_WILAYAH")) => Some(&["validator_wilayah"]),
            (Return, Some("SUBMIT_WILAYAH")) => Some(&["validator_wilayah"]),
            (Return, Some("REVISI_WILAYAH")) => Some(&["validator_wilayah"]),
            (Decide, Some("ANALISIS_KELAYAKAN")) => Some(&["validator_pusat"]),
            (Decide, Some("SUBMIT_PUSAT")) => Some(&["validator_pusat"]),
            (GenerateLaporan, _) => {
                Some(&["validator_pusat", "validator_wilayah", "operator_satker"])
            }
            _ => None,
        }
    }
}

// ============================================================================
// Penghapusan BMN Policy
// ============================================================================

#[derive(Debug, Clone, Copy)]
pub enum PenghapusanBmnAction {
    Create,
    SubmitWilayah,
    /// Validator Wilayah generate konsep SK utk kewenangan WILAYAH
    /// (mewakili Kepala Kejaksaan Tinggi).
    GenerateKonsepSKWilayah,
    /// Validator Wilayah upload signed SK utk kewenangan WILAYAH.
    UploadSignedSKWilayah,
    /// Validator Wilayah forward ke Pusat (untuk kewenangan PUSAT).
    ForwardPusat,
    /// Validator Pusat generate konsep SK / upload signed SK
    /// (mewakili Jaksa Agung Muda Pembinaan).
    GenerateKonsepSKPusat,
    UploadSignedSKPusat,
    Reject,
}

pub struct PenghapusanBmnPolicy;

impl WorkflowPolicy for PenghapusanBmnPolicy {
    type Action = PenghapusanBmnAction;
    fn module_name(&self) -> &'static str {
        "penghapusan_bmn"
    }
    fn allowed_roles(
        &self,
        action: Self::Action,
        current_state: Option<&str>,
    ) -> Option<&'static [&'static str]> {
        use PenghapusanBmnAction::*;
        match (action, current_state) {
            (Create, _) => Some(&["operator_satker"]),
            (SubmitWilayah, Some("DRAFT")) => Some(&["operator_satker"]),
            (SubmitWilayah, Some("RETURNED_TO_OPERATOR")) => Some(&["operator_satker"]),
            (GenerateKonsepSKWilayah, Some("SUBMIT_WILAYAH")) => Some(&["validator_wilayah"]),
            (UploadSignedSKWilayah, Some("KONSEP_SK_WILAYAH_GENERATED")) => {
                Some(&["validator_wilayah"])
            }
            (ForwardPusat, Some("SUBMIT_WILAYAH")) => Some(&["validator_wilayah"]),
            (GenerateKonsepSKPusat, Some("VERIFIKASI_PUSAT")) => Some(&["validator_pusat"]),
            (UploadSignedSKPusat, Some("KONSEP_SK_PUSAT_GENERATED")) => Some(&["validator_pusat"]),
            (Reject, Some("VERIFIKASI_PUSAT")) => Some(&["validator_pusat"]),
            _ => None,
        }
    }
}

// ============================================================================
// Pakaian Dinas Policy
// ============================================================================

#[derive(Debug, Clone, Copy)]
pub enum PakaianDinasAction {
    /// Operator/pelaksana buat pengajuan satker.
    Create,
    /// Pelaksana submit ke Validator Wilayah (non-Kejagung) atau langsung Pusat (Kejagung).
    Submit,
    /// Validator Wilayah forward.
    ForwardPusat,
    /// Validator Wilayah / Pusat kembalikan.
    Return,
    /// Validator Pusat approve / reject.
    Decide,
    /// Admin CRUD master Jenis Pakaian.
    AdminMaster,
}

pub struct PakaianDinasPolicy;

impl WorkflowPolicy for PakaianDinasPolicy {
    type Action = PakaianDinasAction;
    fn module_name(&self) -> &'static str {
        "pakaian_dinas"
    }
    fn allowed_roles(
        &self,
        action: Self::Action,
        current_state: Option<&str>,
    ) -> Option<&'static [&'static str]> {
        use PakaianDinasAction::*;
        match (action, current_state) {
            (Create, _) => Some(&["operator_satker"]),
            (Submit, Some("INPUT")) => Some(&["operator_satker"]),
            (ForwardPusat, Some("SUBMIT_VALIDATOR")) => Some(&["validator_wilayah"]),
            (Return, Some("SUBMIT_VALIDATOR")) => Some(&["validator_wilayah"]),
            (Return, Some("REVISI_WILAYAH")) => Some(&["validator_wilayah"]),
            (Decide, Some("SUBMIT_PUSAT")) => Some(&["validator_pusat"]),
            // Admin master: tidak peduli state; hanya administrator aplikasi.
            (AdminMaster, _) => Some(lib_core::authz::ADMIN_ROLES),
            _ => None,
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn claims_with(role: &str) -> Claims {
        Claims::with_roles(Uuid::nil(), "test", [role])
    }

    /// `for_transition` must agree with the workflow config, in both
    /// directions.
    ///
    /// Forward: every move the engine calls valid should name an action, or the
    /// detail endpoint silently drops it and the workflow strands with no
    /// button. Backward: a pair the engine rejects must NOT name one, or we
    /// would offer a move the engine then refuses.
    ///
    /// This is what catches a hand-written map going stale — an earlier draft
    /// of it claimed SUBMITTED_APPROVER_SATKER -> ACTIVE, which the config
    /// explicitly forbids.
    #[test]
    fn for_transition_agrees_with_workflow_config() {
        use crate::workflow::config::WorkflowConfig;
        let config = WorkflowConfig::default_pemakaian_bmn();

        let states = [
            "DRAFT",
            "SUBMITTED",
            "SUBMITTED_APPROVER_SATKER",
            "REVISI_OPERATOR",
            "APPROVED",
            "ACTIVE",
            "REVOKED",
            "EXPIRED",
            "CANCELLED",
            "REJECTED",
        ];

        for from in states {
            for to in states {
                let valid = config.is_valid_transition(from, to);
                let mapped = PemakaianBmnAction::for_transition(from, to).is_some();
                if mapped {
                    assert!(
                        valid,
                        "for_transition maps {from} -> {to}, but the workflow config rejects it"
                    );
                }
            }
        }

        // The approval chain specifically must be fully mapped, or a role that
        // exists in the policy would have no way to act.
        for (from, to) in [
            ("DRAFT", "SUBMITTED"),
            ("SUBMITTED", "SUBMITTED_APPROVER_SATKER"),
            ("SUBMITTED", "REVISI_OPERATOR"),
            ("SUBMITTED_APPROVER_SATKER", "APPROVED"),
            ("SUBMITTED_APPROVER_SATKER", "REVISI_OPERATOR"),
            ("REVISI_OPERATOR", "SUBMITTED"),
        ] {
            assert!(
                PemakaianBmnAction::for_transition(from, to).is_some(),
                "approval-chain move {from} -> {to} has no action"
            );
        }
    }

    /// Each mapped action must also be one the policy grants to somebody in
    /// that source state — otherwise the detail filter strips it for every
    /// caller and the button never appears for anyone.
    #[test]
    fn every_mapped_transition_is_grantable() {
        for (from, to) in [
            ("DRAFT", "SUBMITTED"),
            ("SUBMITTED", "SUBMITTED_APPROVER_SATKER"),
            ("SUBMITTED", "REVISI_OPERATOR"),
            ("SUBMITTED_APPROVER_SATKER", "APPROVED"),
            ("SUBMITTED_APPROVER_SATKER", "REVISI_OPERATOR"),
            ("REVISI_OPERATOR", "SUBMITTED"),
            ("ACTIVE", "REVOKED"),
        ] {
            let action = PemakaianBmnAction::for_transition(from, to)
                .unwrap_or_else(|| panic!("{from} -> {to} unmapped"));
            let roles = PemakaianBmnPolicy.allowed_roles(action, Some(from));
            assert!(
                roles.is_some_and(|r| !r.is_empty()),
                "{from} -> {to} maps to {action:?} but the policy grants it to nobody"
            );
        }
    }

    #[test]
    fn maker_cannot_check_their_own_submission() {
        let me = Uuid::new_v4();
        let other = Uuid::new_v4();
        for action in [
            PemakaianBmnAction::ValidatorSatkerForward,
            PemakaianBmnAction::ValidatorSatkerReturn,
            PemakaianBmnAction::ApproverSatkerApprove,
            PemakaianBmnAction::ApproverSatkerReturn,
        ] {
            assert!(
                enforce_maker_checker(action, me, Some(me), None).is_err(),
                "{action:?}: the creator must not act on their own permit"
            );
            assert!(enforce_maker_checker(action, me, Some(other), None).is_ok());
        }
    }

    #[test]
    fn approver_cannot_be_the_validator_of_the_same_permit() {
        let me = Uuid::new_v4();
        let maker = Uuid::new_v4();
        for action in [
            PemakaianBmnAction::ApproverSatkerApprove,
            PemakaianBmnAction::ApproverSatkerReturn,
        ] {
            assert!(enforce_maker_checker(action, me, Some(maker), Some(me)).is_err());
            assert!(enforce_maker_checker(action, me, Some(maker), Some(Uuid::new_v4())).is_ok());
        }
        // The validator step itself has no validator to conflict with.
        assert!(
            enforce_maker_checker(
                PemakaianBmnAction::ValidatorSatkerForward,
                me,
                Some(maker),
                Some(me)
            )
            .is_ok()
        );
    }

    #[test]
    fn legacy_rows_without_a_maker_are_not_blocked() {
        let me = Uuid::new_v4();
        assert!(
            enforce_maker_checker(PemakaianBmnAction::ApproverSatkerApprove, me, None, None)
                .is_ok()
        );
    }

    #[test]
    fn operator_may_cancel_only_before_review() {
        let p = PemakaianBmnPolicy;
        let op = claims_with("operator_satker");
        for ok in ["DRAFT", "REVISI_OPERATOR"] {
            assert!(
                p.authorize(&op, PemakaianBmnAction::Cancel, Some(ok))
                    .is_ok()
            );
        }
        for bad in ["SUBMITTED", "SUBMITTED_APPROVER_SATKER", "ACTIVE"] {
            assert!(
                p.authorize(&op, PemakaianBmnAction::Cancel, Some(bad))
                    .is_err()
            );
        }
        assert_eq!(
            PemakaianBmnAction::for_transition("DRAFT", "CANCELLED").map(|a| a.action_label()),
            Some("Batalkan Usulan")
        );
    }

    #[test]
    fn manual_activation_is_the_approvers_and_only_when_approved() {
        let p = PemakaianBmnPolicy;
        assert!(
            p.authorize(
                &claims_with("approver_satker"),
                PemakaianBmnAction::Activate,
                Some("APPROVED")
            )
            .is_ok()
        );
        for role in ["operator_satker", "validator_satker", "validator_pusat"] {
            assert!(
                p.authorize(
                    &claims_with(role),
                    PemakaianBmnAction::Activate,
                    Some("APPROVED")
                )
                .is_err(),
                "{role} must not issue a permit number"
            );
        }
        assert!(
            p.authorize(
                &claims_with("approver_satker"),
                PemakaianBmnAction::Activate,
                Some("DRAFT")
            )
            .is_err()
        );
    }

    #[test]
    fn pemakaian_validator_satker_forward_allowed_only_from_submitted() {
        let p = PemakaianBmnPolicy;
        let claims = claims_with("validator_satker");
        assert!(
            p.authorize(
                &claims,
                PemakaianBmnAction::ValidatorSatkerForward,
                Some("SUBMITTED")
            )
            .is_ok()
        );
        // Wrong state → 403
        let err = p
            .authorize(
                &claims,
                PemakaianBmnAction::ValidatorSatkerForward,
                Some("DRAFT"),
            )
            .unwrap_err();
        assert!(matches!(err, AppError::Authorization(_)));
    }

    #[test]
    fn pemakaian_operator_cannot_approve() {
        let p = PemakaianBmnPolicy;
        let claims = claims_with("operator_satker");
        let err = p
            .authorize(
                &claims,
                PemakaianBmnAction::ApproverSatkerApprove,
                Some("SUBMITTED_APPROVER_SATKER"),
            )
            .unwrap_err();
        assert!(matches!(err, AppError::Authorization(_)));
    }

    /// Regression for the removed escape hatch: every application-admin role
    /// used to pass EVERY policy, in EVERY state, silently. It must now be
    /// refused for every business action of every module.
    #[test]
    fn admin_cannot_perform_any_business_action() {
        let states = [
            None,
            Some("DRAFT"),
            Some("SUBMITTED"),
            Some("SUBMITTED_APPROVER_SATKER"),
            Some("REVISI_OPERATOR"),
            Some("ACTIVE"),
            Some("SUBMIT_WILAYAH"),
            Some("SUBMIT_PUSAT"),
            Some("ANALISIS_KELAYAKAN"),
            Some("VERIFIKASI_PUSAT"),
            Some("INPUT"),
        ];
        for role in lib_core::authz::ADMIN_ROLES {
            let admin = claims_with(role);
            for &state in &states {
                for action in [
                    PemakaianBmnAction::Create,
                    PemakaianBmnAction::Submit,
                    PemakaianBmnAction::ValidatorSatkerForward,
                    PemakaianBmnAction::ApproverSatkerApprove,
                    PemakaianBmnAction::Revoke,
                ] {
                    assert!(
                        PemakaianBmnPolicy.authorize(&admin, action, state).is_err(),
                        "{role} must not do {action:?} @ {state:?}"
                    );
                }
                for action in [
                    KebutuhanBmnAction::CreatePeriode,
                    KebutuhanBmnAction::Decide,
                ] {
                    assert!(KebutuhanBmnPolicy.authorize(&admin, action, state).is_err());
                }
                for action in [PenghapusanBmnAction::Create, PenghapusanBmnAction::Reject] {
                    assert!(
                        PenghapusanBmnPolicy
                            .authorize(&admin, action, state)
                            .is_err()
                    );
                }
                for action in [PakaianDinasAction::Create, PakaianDinasAction::Decide] {
                    assert!(PakaianDinasPolicy.authorize(&admin, action, state).is_err());
                }
            }
            // The one thing an application administrator IS for.
            assert!(
                PakaianDinasPolicy
                    .authorize(&admin, PakaianDinasAction::AdminMaster, None)
                    .is_ok()
            );
        }
    }

    /// The acting role handed to the workflow engine is the one that authorized
    /// the move, not the caller's highest-ranked role.
    #[test]
    fn authorize_as_returns_the_authorizing_role() {
        let both = Claims::with_roles(Uuid::nil(), "u", ["operator_satker", "validator_wilayah"]);
        assert_eq!(both.role, "validator_wilayah");
        let acting = PemakaianBmnPolicy
            .authorize_as(&both, PemakaianBmnAction::Submit, Some("DRAFT"))
            .unwrap();
        assert_eq!(acting, "operator_satker");
    }

    /// Detail endpoint narrowing goes through `allowed_roles`; with no bypass
    /// in `authorize`, an administrator is no longer offered "Cabut Izin".
    #[test]
    fn revoke_is_offered_to_approver_satker_and_nobody_else() {
        let allowed = PemakaianBmnPolicy
            .allowed_roles(PemakaianBmnAction::Revoke, Some("ACTIVE"))
            .unwrap();
        assert_eq!(allowed, ["approver_satker"]);
    }

    #[test]
    fn enforce_no_admin_revoke_blocks_admin() {
        let admin = claims_with("admin");
        let err = enforce_no_admin_revoke(&admin).unwrap_err();
        assert!(matches!(err, AppError::Authorization(_)));
        // Non-admin lolos
        let approver = claims_with("approver_satker");
        assert!(enforce_no_admin_revoke(&approver).is_ok());
        // Approver yang JUGA admin: policy meloloskannya lewat role approver,
        // guard inilah yang menolaknya.
        let both = Claims::with_roles(Uuid::nil(), "u", ["approver_satker", "admin"]);
        assert!(
            PemakaianBmnPolicy
                .authorize(&both, PemakaianBmnAction::Revoke, Some("ACTIVE"))
                .is_ok()
        );
        assert!(enforce_no_admin_revoke(&both).is_err());
    }

    #[test]
    fn enforce_monitoring_read_allows_wilayah_pusat_and_satker_roles() {
        for role in [
            "validator_wilayah",
            "validator_pusat",
            "operator_satker",
            "validator_satker",
            "approver_satker",
            "admin",
        ] {
            assert!(
                enforce_monitoring_read(&claims_with(role)).is_ok(),
                "role {role} seharusnya boleh baca monitoring"
            );
        }
    }

    #[test]
    fn enforce_monitoring_read_rejects_unknown_role() {
        let err = enforce_monitoring_read(&claims_with("tamu")).unwrap_err();
        assert!(matches!(err, AppError::Authorization(_)));
    }

    #[test]
    fn monitoring_audience_has_no_pemakaian_write_action() {
        // Validator Wilayah/Pusat read-only: tidak ada satu pun kombinasi
        // (action, state) yg mengizinkan mereka menulis. Walk semua action ×
        // state kanonik → harus selalu 403.
        let p = PemakaianBmnPolicy;
        let states = [
            None,
            Some("DRAFT"),
            Some("SUBMITTED"),
            Some("SUBMITTED_APPROVER_SATKER"),
            Some("REVISI_OPERATOR"),
            Some("ACTIVE"),
        ];
        let actions = [
            PemakaianBmnAction::Create,
            PemakaianBmnAction::Submit,
            PemakaianBmnAction::ValidatorSatkerForward,
            PemakaianBmnAction::ValidatorSatkerReturn,
            PemakaianBmnAction::ApproverSatkerApprove,
            PemakaianBmnAction::ApproverSatkerReturn,
            PemakaianBmnAction::Resubmit,
            PemakaianBmnAction::Revoke,
            PemakaianBmnAction::UpdateDraft,
            PemakaianBmnAction::Cancel,
            PemakaianBmnAction::Activate,
            PemakaianBmnAction::GenerateDocument,
            PemakaianBmnAction::UploadSigned,
        ];
        for role in ["validator_wilayah", "validator_pusat"] {
            for &state in &states {
                for &action in &actions {
                    assert!(
                        p.authorize(&claims_with(role), action, state).is_err(),
                        "{role} tidak boleh menulis ({action:?} @ {state:?})"
                    );
                }
            }
        }
    }

    #[test]
    fn kebutuhan_create_periode_only_validator_pusat() {
        let p = KebutuhanBmnPolicy;
        assert!(
            p.authorize(
                &claims_with("validator_pusat"),
                KebutuhanBmnAction::CreatePeriode,
                None
            )
            .is_ok()
        );
        assert!(
            p.authorize(
                &claims_with("operator_satker"),
                KebutuhanBmnAction::CreatePeriode,
                None
            )
            .is_err()
        );
    }

    #[test]
    fn penghapusan_wilayah_cannot_generate_sk_pusat() {
        let p = PenghapusanBmnPolicy;
        let claims = claims_with("validator_wilayah");
        let err = p
            .authorize(
                &claims,
                PenghapusanBmnAction::GenerateKonsepSKPusat,
                Some("VERIFIKASI_PUSAT"),
            )
            .unwrap_err();
        assert!(matches!(err, AppError::Authorization(_)));
    }

    #[test]
    fn case_insensitive_role_match() {
        let p = PemakaianBmnPolicy;
        let claims = claims_with("Validator_Satker"); // mixed case
        assert!(
            p.authorize(
                &claims,
                PemakaianBmnAction::ValidatorSatkerForward,
                Some("SUBMITTED")
            )
            .is_ok()
        );
    }
}
