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
    if !url.pathname().starts_with(V1_PREFIX) {
        return None;
    }
    Some(url.href())
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
    let Some(token) = crate::features::auth::AuthService::get_token() else {
        // No session yet — park the handoff for after login.
        save_handoff(&handoff);
        return false;
    };
    // A token sitting in localStorage can be stale (the tab was left open past
    // its TTL). Posting that to v1 would "succeed" here and then fail there with
    // a 401 the user cannot explain, so require a live token and let the login
    // form mint a fresh one instead.
    let fresh = crate::features::auth::AuthService::decode_jwt_claims(&token)
        .map(|session| crate::features::auth::AuthService::is_session_valid(&session))
        .unwrap_or(false);
    if !fresh {
        save_handoff(&handoff);
        return false;
    }
    complete_handoff(&token, &handoff)
}
