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
// menentukan role yg sah berdasar (action, state). Admin/superadmin bypass
// (escape hatch untuk recovery) — tapi tetap di-log via tracing.
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

    /// Authorize satu aksi. Admin/superadmin bypass (di-log). Selain itu,
    /// role caller harus ada di whitelist yg di-return `allowed_roles`.
    fn authorize(
        &self,
        claims: &Claims,
        action: Self::Action,
        current_state: Option<&str>,
    ) -> Result<(), AppError> {
        let role_lower = claims.role.to_ascii_lowercase();

        // Admin escape-hatch — di-log agar audit tahu manual override.
        if matches!(role_lower.as_str(), "admin" | "admin_pusat" | "superadmin") {
            tracing::warn!(
                module = self.module_name(),
                action = ?action,
                state = current_state.unwrap_or("<none>"),
                user_id = %claims.user_id,
                role = %claims.role,
                "policy: admin/superadmin bypass — review apakah ini override yg sah"
            );
            return Ok(());
        }

        let allowed = self.allowed_roles(action, current_state).ok_or_else(|| {
            AppError::Authorization(format!(
                "Aksi {:?} tidak valid pada state '{}' utk modul {}",
                action,
                current_state.unwrap_or("<none>"),
                self.module_name()
            ))
        })?;

        if allowed.iter().any(|r| r.eq_ignore_ascii_case(&role_lower)) {
            tracing::debug!(
                module = self.module_name(),
                action = ?action,
                state = current_state.unwrap_or("<none>"),
                user_id = %claims.user_id,
                role = %claims.role,
                "policy: allow"
            );
            Ok(())
        } else {
            tracing::info!(
                module = self.module_name(),
                action = ?action,
                state = current_state.unwrap_or("<none>"),
                user_id = %claims.user_id,
                role = %claims.role,
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
            (ApproverSatkerReturn, Some("SUBMITTED_APPROVER_SATKER")) => {
                Some(&["approver_satker"])
            }

            // Resubmit: hanya dari REVISI_OPERATOR.
            (Resubmit, Some("REVISI_OPERATOR")) => Some(&["operator_satker"]),

            // Revoke: hanya Approver Satker, dan hanya saat ACTIVE.
            // Stakeholder eksplisit: Admin TIDAK boleh revoke. Tapi karena
            // authorize() men-bypass admin di awal, kita tidak bisa menolak
            // admin di level ini — di handler revoke wajib panggil
            // `enforce_no_admin_revoke(&claims)` sebagai guard tambahan.
            (Revoke, Some("ACTIVE")) => Some(&["approver_satker"]),

            // Kombinasi lain → tidak valid (None → 403).
            _ => None,
        }
    }
}

/// Stakeholder mandate: Admin TIDAK boleh revoke izin pemakaian BMN.
/// Karena `WorkflowPolicy::authorize` punya admin bypass utk recovery,
/// guard tambahan ini dipanggil di handler revoke utk menolak admin.
pub fn enforce_no_admin_revoke(claims: &Claims) -> Result<(), AppError> {
    if matches!(
        claims.role.to_ascii_lowercase().as_str(),
        "admin" | "admin_pusat" | "superadmin"
    ) {
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
            (GenerateLaporan, _) => Some(&["validator_pusat", "validator_wilayah", "operator_satker"]),
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
            // Admin master: tidak peduli state. Note: admin bypass otomatis
            // di authorize() — list ini cuma utk role non-admin yg juga boleh
            // (di sini kosong → hanya admin yg lewat).
            (AdminMaster, _) => Some(&[]),
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
        Claims {
            user_id: Uuid::nil(),
            username: "test".into(),
            role: role.into(),
            permissions: vec![],
            nip: None,
            name: None,
            nama: None,
            jabatan: None,
            satker_code: None,
        }
    }

    #[test]
    fn pemakaian_validator_satker_forward_allowed_only_from_submitted() {
        let p = PemakaianBmnPolicy;
        let claims = claims_with("validator_satker");
        assert!(p
            .authorize(
                &claims,
                PemakaianBmnAction::ValidatorSatkerForward,
                Some("SUBMITTED")
            )
            .is_ok());
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

    #[test]
    fn pemakaian_admin_bypass_with_audit_log() {
        let p = PemakaianBmnPolicy;
        let claims = claims_with("admin");
        // Admin lolos meski state mismatch.
        assert!(p
            .authorize(
                &claims,
                PemakaianBmnAction::ApproverSatkerApprove,
                Some("DRAFT")
            )
            .is_ok());
    }

    #[test]
    fn enforce_no_admin_revoke_blocks_admin() {
        let admin = claims_with("admin");
        let err = enforce_no_admin_revoke(&admin).unwrap_err();
        assert!(matches!(err, AppError::Authorization(_)));
        // Non-admin lolos
        let approver = claims_with("approver_satker");
        assert!(enforce_no_admin_revoke(&approver).is_ok());
    }

    #[test]
    fn kebutuhan_create_periode_only_validator_pusat() {
        let p = KebutuhanBmnPolicy;
        assert!(p
            .authorize(
                &claims_with("validator_pusat"),
                KebutuhanBmnAction::CreatePeriode,
                None
            )
            .is_ok());
        assert!(p
            .authorize(
                &claims_with("operator_satker"),
                KebutuhanBmnAction::CreatePeriode,
                None
            )
            .is_err());
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
        assert!(p
            .authorize(
                &claims,
                PemakaianBmnAction::ValidatorSatkerForward,
                Some("SUBMITTED")
            )
            .is_ok());
    }
}
