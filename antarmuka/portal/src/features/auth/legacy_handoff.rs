//! SSO handoff back to SIMPEL v1 (the legacy Laravel monolith).
//!
//! v1 has no login page of its own. When an unauthenticated browser reaches a
//! protected v1 route, `TokenToOAuthMiddleware` bounces it to the Portal with
//! three query parameters:
//!
//! ```text
//! {PORTAL_URL}/login?return_to=<v1 url>&callback=<v1 oauth-callback url>&state=<nonce>
//! ```
//!
//! The Portal is the only place a user can authenticate, so once login succeeds
//! it must hand the freshly minted JWT back to v1 rather than render the Portal
//! dashboard.
//!
//! Until this module existed the Portal only understood `redirect_uri` and
//! `redirect=perlengkapan` (both v2 targets), so a v1 bounce landed on the
//! Portal dashboard and the round trip dead-ended: v1 was unreachable through
//! the apps page no matter how the user got there.

use serde::{Deserialize, Serialize};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

use super::{AuthService, UserSession};

/// Query parameters v1's `TokenToOAuthMiddleware` sends on a bounce.
pub const RETURN_TO_PARAM: &str = "return_to";
pub const CALLBACK_PARAM: &str = "callback";
pub const STATE_PARAM: &str = "state";

/// Every legitimate v1 deep link shares this path prefix.
const V1_PREFIX: &str = "/perlengkapan/simpel/v1";

/// Where a pending handoff is parked while the user works through
/// MFA / forced-password-change detours.
const STORAGE_KEY: &str = "legacy_handoff";

/// A pending "return the JWT to v1" request.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LegacyHandoff {
    /// Absolute, same-origin v1 `auth/oauth-callback` URL to POST the token to.
    pub callback: String,
    /// v1's login-CSRF nonce; echoed back so the callback can `hash_equals` it.
    pub state: Option<String>,
}

/// Resolve `raw` against the current document and accept it only if it stays on
/// this origin and points at v1.
///
/// `return_to` arrives as an absolute URL (`$request->fullUrl()` in v1), so it
/// cannot be compared textually — and accepting it verbatim would turn the
/// Portal into an open redirector. Parsing against the current location pins the
/// origin to this app's own, which is the only place the token is valid anyway.
#[cfg(target_arch = "wasm32")]
fn resolve_v1_url(raw: &str) -> Option<String> {
    let window = web_sys::window()?;
    let location = window.location();
    let base = location.href().ok()?;
    let url = web_sys::Url::new_with_base(raw, &base).ok()?;

    // The two `origin()` getters are NOT the same shape: `Url::origin()` is a
    // plain string, while `Location::origin()` is fallible. Unwrap only the
    // latter; comparing `String` with `Result<String, JsValue>` does not
    // compile, and defaulting the Err case to "same" would be the wrong way to
    // fail on the one input that matters.
    if url.origin() != location.origin().ok()? {
        return None;
    }
    if !is_v1_path(&url.pathname()) {
        return None;
    }
    Some(url.href())
}

/// Is `path` v1 itself or something under it?
///
/// A bare `starts_with(V1_PREFIX)` also accepted `/perlengkapan/simpel/v1x/…`
/// and `/perlengkapan/simpel/v10/…` — still same-origin, so not a token leak to
/// a third party, but not v1 either, and the token should go to v1's callback
/// and nowhere else. Match on a path-segment boundary. The pathname here has
/// already been normalised by the URL parser (`..` resolved), so a traversal
/// cannot climb out after passing this check.
///
/// Compiled where it is used — the wasm32 resolver — and in tests; on a host
/// non-test build it would be dead code.
#[cfg(any(target_arch = "wasm32", test))]
pub(crate) fn is_v1_path(path: &str) -> bool {
    path == V1_PREFIX
        || path
            .strip_prefix(V1_PREFIX)
            .is_some_and(|rest| rest.starts_with('/'))
}

#[cfg(not(target_arch = "wasm32"))]
fn resolve_v1_url(_raw: &str) -> Option<String> {
    None
}

/// Read a v1 handoff request off the current URL, if one is present.
///
/// Both `return_to` and `callback` must resolve to v1; a request carrying only
/// one of them is malformed (it would leave the user stranded on whichever
/// target did resolve), so it is ignored rather than half-honoured.
pub fn parse_handoff() -> Option<LegacyHandoff> {
    #[cfg(target_arch = "wasm32")]
    {
        let search = web_sys::window()
            .and_then(|w| w.location().search().ok())
            .unwrap_or_default();
        let params = web_sys::UrlSearchParams::new_with_str(&search).ok()?;

        // `return_to` is the page the user originally asked for. It is not
        // needed to complete the handoff (v1 replays it from its own session),
        // but requiring it keeps a bare `?callback=…` — which an attacker could
        // otherwise paste anywhere — from being treated as a real flow.
        let return_to = params.get(RETURN_TO_PARAM)?;
        resolve_v1_url(&return_to)?;

        let callback = resolve_v1_url(&params.get(CALLBACK_PARAM)?)?;

        // Tail expression, not `return`: on wasm32 the sibling
        // `#[cfg(not(target_arch = "wasm32"))]` block below is compiled out, so
        // this block *is* the function's tail (clippy::needless_return).
        Some(LegacyHandoff {
            callback,
            state: params.get(STATE_PARAM),
        })
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        None
    }
}

/// Park a handoff so it survives MFA / password-change detours.
pub fn save_handoff(handoff: &LegacyHandoff) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(storage) = web_sys::window()
            .and_then(|w| w.session_storage().ok())
            .flatten()
            && let Ok(encoded) = serde_json::to_string(handoff)
        {
            let _ = storage.set_item(STORAGE_KEY, &encoded);
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handoff;
    }
}

/// Take the parked handoff, clearing it so it fires at most once.
pub fn take_handoff() -> Option<LegacyHandoff> {
    #[cfg(target_arch = "wasm32")]
    {
        let storage = web_sys::window()
            .and_then(|w| w.session_storage().ok())
            .flatten()?;
        let encoded = storage.get_item(STORAGE_KEY).ok().flatten()?;
        let _ = storage.remove_item(STORAGE_KEY);
        // Re-validate on the way out: the stored value outlives the URL it came
        // from, so it must not become a way to smuggle a foreign target in.
        let handoff: LegacyHandoff = serde_json::from_str(&encoded).ok()?;
        if resolve_v1_url(&handoff.callback).as_deref() != Some(handoff.callback.as_str()) {
            return None;
        }
        Some(handoff)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        None
    }
}

/// Parse the handoff from the current URL, or fall back to a parked one.
///
/// A URL-supplied handoff is parked first so that a detour (MFA, forced
/// password change) — which navigates away and drops the query string — can
/// still complete it.
pub fn current_handoff() -> Option<LegacyHandoff> {
    if let Some(handoff) = parse_handoff() {
        save_handoff(&handoff);
        return Some(handoff);
    }
    take_handoff()
}

/// Hand `token` back to v1 by submitting v1's own OAuth callback.
///
/// A form POST (not a navigation with `?token=…`) for two reasons: v1 prefers
/// the body over the query string, and a JWT in a query string is persisted in
/// browser history, access logs and `Referer` headers. `state` rides along so
/// v1 can complete its login-CSRF check.
///
/// Returns `true` when the form was submitted — the browser is leaving this
/// document, so there is nothing further to do.
#[cfg(target_arch = "wasm32")]
pub fn complete_handoff(token: &str, handoff: &LegacyHandoff) -> bool {
    let Some(document) = web_sys::window().and_then(|w| w.document()) else {
        return false;
    };
    let Ok(form) = document.create_element("form") else {
        return false;
    };
    let Ok(form) = form.dyn_into::<web_sys::HtmlFormElement>() else {
        return false;
    };

    let _ = form.set_attribute("method", "post");
    let _ = form.set_attribute("action", &handoff.callback);

    for (name, value) in [
        ("token", token),
        ("state", handoff.state.as_deref().unwrap_or("")),
    ] {
        let Ok(input) = document.create_element("input") else {
            return false;
        };
        let Ok(input) = input.dyn_into::<web_sys::HtmlInputElement>() else {
            return false;
        };
        let _ = input.set_attribute("type", "hidden");
        let _ = input.set_attribute("name", name);
        input.set_value(value);
        let _ = form.append_child(&input);
    }

    let Some(body) = document.body() else {
        return false;
    };
    if body.append_child(&form).is_err() {
        return false;
    }
    // `submit()` throws (a `Result`) when the form is not connected or already
    // submitting; report that as "not handed off" so the caller can fall back
    // to the login form instead of appearing to have navigated.
    form.submit().is_ok()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn complete_handoff(_token: &str, _handoff: &LegacyHandoff) -> bool {
    false
}

/// Hand the freshly minted token back to v1 when this login began as a v1
/// bounce. Returns `true` once the form has been submitted — the browser is
/// leaving this document, so no further navigation should be attempted.
///
/// This is the choke point for every post-login path (password, passkey, MFA
/// verify, backup code). Each caller already decides whether to detour to
/// `/password` first, but that is a per-call-site check; re-asserting the policy
/// here means a future login path that forgets it cannot hand a
/// password-change-pending token to v1. The parked handoff is deliberately left
/// in place when a step blocks, so the detour can still complete it.
pub fn complete_handoff_if_pending(session: &UserSession) -> bool {
    let Some(handoff) = take_handoff() else {
        return false;
    };
    if let Some(step) = blocking_policy_step(session) {
        leptos::logging::debug_warn!(
            "v1 handoff deferred: {} still required before the token can be handed back",
            step.label()
        );
        save_handoff(&handoff);
        return false;
    }
    let Some(token) = session.access_token.clone().or_else(AuthService::get_token) else {
        save_handoff(&handoff);
        return false;
    };
    if complete_handoff(&token, &handoff) {
        return true;
    }
    // The form could not be submitted; keep the request parked so a retry can
    // still complete it rather than dropping the user's original destination.
    save_handoff(&handoff);
    false
}

/// If the browser arrived with a v1 handoff and already holds a valid session,
/// complete it immediately instead of showing the login form again.
///
/// This is the ordinary path once a user has logged into the Portal once: v1
/// bounces them to `/portal/login?return_to=…`, the same-origin session is
/// still live, and re-authenticating would be pure friction.
pub fn try_immediate_handoff() -> bool {
    let Some(handoff) = current_handoff() else {
        return false;
    };
    let Some(token) = AuthService::get_token() else {
        // No session yet — park the handoff for after login.
        save_handoff(&handoff);
        return false;
    };
    // A token sitting in localStorage can be stale (the tab was left open past
    // its TTL). Posting that to v1 would "succeed" here and then fail there with
    // a 401 the user cannot explain, so require a live token and let the login
    // form mint a fresh one instead.
    let Some(session) = AuthService::decode_jwt_claims(&token)
        .ok()
        .filter(AuthService::is_session_valid)
    else {
        save_handoff(&handoff);
        return false;
    };
    // Never hand a token to v1 while an account-policy step is still outstanding.
    //
    // v1's callback (`AuthController::oauthCallback`) logs the user straight in
    // once the JWT verifies — it does not read `require_password_change` (or
    // `mfa_setup_required`) from the claims. So posting a token that still
    // carries either flag creates a full v1 session and bypasses the step: the
    // user reaches the v1 dashboard with the password they were told to change
    // (CWE-841). Parking the handoff is what makes the detour work — the login
    // page re-runs this on the next render, and the portal routes the user
    // through `/password` (or MFA setup) first.
    if let Some(step) = blocking_policy_step(&session) {
        leptos::logging::debug_warn!(
            "v1 handoff deferred: {} still required before the token can be handed back",
            step.label()
        );
        save_handoff(&handoff);
        return false;
    }
    complete_handoff(&token, &handoff)
}

/// An account-policy step that must complete before a token may be handed to
/// v1.
///
/// v1's callback logs the user straight in once the JWT verifies — it reads
/// neither `require_password_change` nor `mfa_setup_required` from the claims.
/// Posting a token that still carries either flag therefore creates a full v1
/// session and bypasses the step (CWE-841: the user reaches the v1 dashboard
/// with the password they were told to change). The portal must finish the step
/// first, and this enum is what names it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolicyStep {
    /// The user must change an expired/temporary password.
    PasswordChange,
    /// The user must enrol a second factor.
    MfaSetup,
}

impl PolicyStep {
    /// Human-readable name, for logs.
    pub fn label(self) -> &'static str {
        match self {
            Self::PasswordChange => "password change",
            Self::MfaSetup => "MFA setup",
        }
    }

    /// Portal route that completes the step.
    pub fn route(self) -> &'static str {
        match self {
            Self::PasswordChange => "/password",
            Self::MfaSetup => "/mfa/setup",
        }
    }
}

/// The account-policy step that must complete before a token may be handed to
/// v1, if any.
///
/// This is the single place that decides which session flags block the handoff.
/// The portal's login/passkey flows already test `require_password_change`
/// before navigating, but that check is per-call-site: a new login path that
/// forgets it would silently reintroduce the bypass. Routing the handoff
/// decision through one predicate keeps the rule in a single place.
pub fn blocking_policy_step(session: &UserSession) -> Option<PolicyStep> {
    if session.require_password_change {
        return Some(PolicyStep::PasswordChange);
    }
    if session.mfa_setup_required {
        return Some(PolicyStep::MfaSetup);
    }
    None
}
