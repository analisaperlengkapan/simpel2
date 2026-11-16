//! Account-policy steps that must be completed before a session may leave the
//! portal for another page.
//!
//! A session can carry `require_password_change` or `mfa_setup_required`. The
//! portal's route guards send such a user to `/password` or `/mfa/setup`, but a
//! full page navigation (`location.set_href`) leaves the SPA and skips those
//! guards for one hop: long enough to land on the target with the step still
//! open. Every path that hands a fresh session to a full navigation asks this
//! predicate first.

use super::UserSession;

/// A step the user must finish before the session is used elsewhere.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolicyStep {
    /// The user must change an expired/temporary password.
    PasswordChange,
    /// The user must enrol a second factor.
    MfaSetup,
}

impl PolicyStep {
    /// Portal route that completes the step.
    pub fn route(self) -> &'static str {
        match self {
            Self::PasswordChange => "/password",
            Self::MfaSetup => "/mfa/setup",
        }
    }
}

/// The step that blocks `session`, if any. Password change wins when both flags
/// are set: it is checked first, and either route clears only its own flag, so
/// the order must be stable.
///
/// One predicate instead of a check at each call site: a new login path that
/// forgot the check would silently let the user past the step.
pub fn blocking_policy_step(session: &UserSession) -> Option<PolicyStep> {
    if session.require_password_change {
        return Some(PolicyStep::PasswordChange);
    }
    if session.mfa_setup_required {
        return Some(PolicyStep::MfaSetup);
    }
    None
}
