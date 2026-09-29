//! # Break-glass: the audited way to force a workflow state
//!
//! Business decisions belong to business roles. An application administrator
//! therefore cannot approve, reject or complete anything through the normal
//! endpoints (see [`crate::shared::policy`]) — but workflows do get stuck (a
//! validator left the organisation, a state was entered by mistake), and "call a
//! DBA" is worse for audit than a controlled override.
//!
//! This module holds the **pure** half of that override — what may be asked for
//! and what counts as a valid justification — so it is unit-testable without a
//! database. The handler that executes it lives in `admin::break_glass`.
//!
//! What makes it break-glass rather than a back door:
//!
//! * **Reason is mandatory** and must actually be a sentence
//!   ([`MIN_REASON_CHARS`]), not `"x"`.
//! * **Every attempt is logged first** (`perlengkapan.break_glass_log`,
//!   migration V012) — if the log row cannot be written, the transition is not
//!   attempted. Refused and failed attempts are rows too.
//! * The transition runs as the engine's internal actor with the **administrator's
//!   own user id**, so the workflow's activity trail names the person, and the
//!   note is tagged `[BREAK-GLASS]` so it cannot be mistaken for a decision.
//! * It is reviewable: `GET /admin/break-glass` is open to Validator Pusat as
//!   well as administrators, i.e. to the people whose decisions were overridden.

use serde::{Deserialize, Serialize};

use crate::shared::error::AppError;

/// Shortest justification accepted, in characters (after trimming).
pub const MIN_REASON_CHARS: usize = 20;
/// Longest justification accepted, in characters.
pub const MAX_REASON_CHARS: usize = 1000;
/// Longest optional reference (ticket / nota dinas number).
pub const MAX_REFERENCE_CHARS: usize = 100;

/// Workflow modules that support a break-glass transition.
///
/// Deliberately a closed set: `pakaian_dinas` keeps its state per satker
/// response, not in the generic engine, and gets no override here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakGlassModule {
    PemakaianBmn,
    PenghapusanBmn,
    KebutuhanBmn,
}

impl BreakGlassModule {
    /// Parse the `{module}` path segment.
    pub fn parse(raw: &str) -> Result<Self, AppError> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "pemakaian_bmn" | "pemakaian-bmn" => Ok(Self::PemakaianBmn),
            "penghapusan_bmn" | "penghapusan-bmn" => Ok(Self::PenghapusanBmn),
            "kebutuhan_bmn" | "kebutuhan-bmn" => Ok(Self::KebutuhanBmn),
            other => Err(AppError::BadRequest(format!(
                "Modul '{other}' tidak mendukung break-glass (pemakaian_bmn, penghapusan_bmn, kebutuhan_bmn)"
            ))),
        }
    }

    /// The stable name written to the log and the audit trail.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PemakaianBmn => "pemakaian_bmn",
            Self::PenghapusanBmn => "penghapusan_bmn",
            Self::KebutuhanBmn => "kebutuhan_bmn",
        }
    }

    /// The table holding the entity's canonical `status` state-name column —
    /// the same mapping the workflow engine uses. A fixed literal per variant,
    /// never caller input, so it is safe to splice into SQL.
    pub fn status_table(self) -> &'static str {
        match self {
            Self::PemakaianBmn => "izin_pemakaian_bmn",
            Self::PenghapusanBmn => "penghapusan_bmn",
            Self::KebutuhanBmn => "pengajuan_kebutuhan_bmn",
        }
    }
}

/// The request body of `POST /admin/break-glass/{module}/{id}/transition`.
#[derive(Debug, Clone, Deserialize)]
pub struct BreakGlassRequest {
    /// Target state name, e.g. `"ACTIVE"`.
    pub target_status: String,
    /// Why the normal route cannot be used. Mandatory.
    pub alasan: String,
    /// Optional ticket / incident / nota-dinas reference.
    #[serde(default)]
    pub referensi: Option<String>,
}

/// A [`BreakGlassRequest`] that passed [`BreakGlassRequest::validated`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ValidBreakGlass {
    /// Upper-cased, trimmed state name.
    pub target_status: String,
    pub alasan: String,
    pub referensi: Option<String>,
}

impl BreakGlassRequest {
    /// Trim, bound and normalise. Errors are `400`s naming the field.
    pub fn validated(self) -> Result<ValidBreakGlass, AppError> {
        let target_status = self.target_status.trim().to_ascii_uppercase();
        if target_status.is_empty() || target_status.len() > 64 {
            return Err(AppError::BadRequest(
                "target_status wajib diisi (nama state, mis. ACTIVE)".to_string(),
            ));
        }

        let alasan = self.alasan.trim().to_string();
        let chars = alasan.chars().count();
        if chars < MIN_REASON_CHARS {
            return Err(AppError::BadRequest(format!(
                "Alasan break-glass wajib diisi minimal {MIN_REASON_CHARS} karakter (saat ini {chars})"
            )));
        }
        if chars > MAX_REASON_CHARS {
            return Err(AppError::BadRequest(format!(
                "Alasan break-glass maksimal {MAX_REASON_CHARS} karakter"
            )));
        }

        let referensi = self
            .referensi
            .map(|r| r.trim().to_string())
            .filter(|r| !r.is_empty());
        if referensi
            .as_ref()
            .is_some_and(|r| r.chars().count() > MAX_REFERENCE_CHARS)
        {
            return Err(AppError::BadRequest(format!(
                "Referensi maksimal {MAX_REFERENCE_CHARS} karakter"
            )));
        }

        Ok(ValidBreakGlass {
            target_status,
            alasan,
            referensi,
        })
    }
}

impl ValidBreakGlass {
    /// The note stamped on the workflow's activity row: tagged so an override is
    /// never mistaken for a decision by the role that normally makes it.
    pub fn activity_note(&self) -> String {
        match &self.referensi {
            Some(reference) => format!("[BREAK-GLASS] {} (ref: {reference})", self.alasan),
            None => format!("[BREAK-GLASS] {}", self.alasan),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(target: &str, alasan: &str, referensi: Option<&str>) -> BreakGlassRequest {
        BreakGlassRequest {
            target_status: target.to_string(),
            alasan: alasan.to_string(),
            referensi: referensi.map(str::to_string),
        }
    }

    const GOOD_REASON: &str = "Validator satker mutasi, usulan macet di SUBMITTED";

    #[test]
    fn a_real_sentence_is_required_as_the_reason() {
        assert!(request("ACTIVE", "x", None).validated().is_err());
        assert!(request("ACTIVE", "", None).validated().is_err());
        // Padding with whitespace must not reach the minimum.
        assert!(
            request("ACTIVE", &format!("a{}", " ".repeat(40)), None)
                .validated()
                .is_err()
        );
        // One short of the minimum, and exactly the minimum.
        let short = "a".repeat(MIN_REASON_CHARS - 1);
        let exact = "a".repeat(MIN_REASON_CHARS);
        assert!(request("ACTIVE", &short, None).validated().is_err());
        assert!(request("ACTIVE", &exact, None).validated().is_ok());
    }

    #[test]
    fn the_minimum_counts_characters_not_bytes() {
        // 20 multi-byte characters: 60 bytes but exactly the minimum in chars.
        let exact = "é".repeat(MIN_REASON_CHARS);
        assert!(request("ACTIVE", &exact, None).validated().is_ok());
        let short = "é".repeat(MIN_REASON_CHARS - 1);
        assert!(request("ACTIVE", &short, None).validated().is_err());
    }

    #[test]
    fn an_overlong_reason_is_refused() {
        let long = "a".repeat(MAX_REASON_CHARS + 1);
        assert!(request("ACTIVE", &long, None).validated().is_err());
    }

    #[test]
    fn the_target_state_is_normalised_and_required() {
        let ok = request("  active ", GOOD_REASON, None).validated().unwrap();
        assert_eq!(ok.target_status, "ACTIVE");
        assert!(request("   ", GOOD_REASON, None).validated().is_err());
        assert!(
            request(&"A".repeat(65), GOOD_REASON, None)
                .validated()
                .is_err()
        );
    }

    #[test]
    fn the_reference_is_optional_trimmed_and_bounded() {
        let none = request("ACTIVE", GOOD_REASON, Some("   "))
            .validated()
            .unwrap();
        assert_eq!(none.referensi, None);
        let some = request("ACTIVE", GOOD_REASON, Some(" ND-123/2026 "))
            .validated()
            .unwrap();
        assert_eq!(some.referensi.as_deref(), Some("ND-123/2026"));
        let long = "r".repeat(MAX_REFERENCE_CHARS + 1);
        assert!(
            request("ACTIVE", GOOD_REASON, Some(&long))
                .validated()
                .is_err()
        );
    }

    #[test]
    fn an_override_is_tagged_so_it_cannot_pass_for_a_decision() {
        let v = request("ACTIVE", GOOD_REASON, Some("ND-1"))
            .validated()
            .unwrap();
        let note = v.activity_note();
        assert!(note.starts_with("[BREAK-GLASS] "));
        assert!(note.contains(GOOD_REASON));
        assert!(note.contains("ND-1"));
        let bare = request("ACTIVE", GOOD_REASON, None).validated().unwrap();
        assert!(!bare.activity_note().contains("ref:"));
    }

    #[test]
    fn only_the_engine_backed_modules_are_supported() {
        assert_eq!(
            BreakGlassModule::parse("pemakaian_bmn").unwrap(),
            BreakGlassModule::PemakaianBmn
        );
        assert_eq!(
            BreakGlassModule::parse("Penghapusan-BMN").unwrap(),
            BreakGlassModule::PenghapusanBmn
        );
        assert_eq!(
            BreakGlassModule::parse(" kebutuhan_bmn ").unwrap(),
            BreakGlassModule::KebutuhanBmn
        );
        for other in [
            "pakaian_dinas",
            "bank_aset",
            "",
            "izin_pemakaian_bmn; DROP TABLE x",
        ] {
            assert!(
                BreakGlassModule::parse(other).is_err(),
                "{other:?} must be refused"
            );
        }
    }

    /// The table name is spliced into SQL, so it must be a fixed identifier for
    /// every variant — never anything derived from the request.
    #[test]
    fn status_tables_are_plain_fixed_identifiers() {
        for m in [
            BreakGlassModule::PemakaianBmn,
            BreakGlassModule::PenghapusanBmn,
            BreakGlassModule::KebutuhanBmn,
        ] {
            let t = m.status_table();
            assert!(t.chars().all(|c| c.is_ascii_lowercase() || c == '_'), "{t}");
        }
    }
}
