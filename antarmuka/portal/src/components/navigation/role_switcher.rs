//! "Bertindak sebagai" — session role activation (NIST INCITS 359).
//!
//! A user assigned both `operator_satker` and `validator_satker` must not be both
//! in the same session: that is the dynamic separation of duty the issuer
//! enforces when `AUTHENC_ACTIVE_ROLE_ENFORCEMENT` is on. Then `realm_access.roles`
//! holds exactly the active role, and this control is how the user picks another
//! one of the roles they were assigned.
//!
//! It renders nothing unless [`UserSession::switchable_roles`] says there is
//! something to switch — one role, or enforcement off, and there is no control to
//! offer that the server would refuse.

use crate::features::auth::{AuthService, UserSession};
use leptos::prelude::*;
use lib_core::authz::role_label;

/// The choices for the `<select>`: `(value, label, is_active)`.
///
/// Split out so the ordering and the "which one is selected" rule are testable
/// without a DOM.
pub fn role_options(session: &UserSession) -> Vec<(String, String, bool)> {
    let Some(assigned) = session.switchable_roles() else {
        return Vec::new();
    };
    let active = session
        .active_role
        .as_deref()
        .or(session.roles.first().map(String::as_str));
    assigned
        .iter()
        .map(|role| {
            (
                role.clone(),
                role_label(role),
                active.is_some_and(|a| a.eq_ignore_ascii_case(role)),
            )
        })
        .collect()
}

#[component]
pub fn RoleSwitcher(session: UserSession) -> impl IntoView {
    let options = role_options(&session);
    if options.is_empty() {
        return ().into_any();
    }

    let (busy, set_busy) = signal(false);
    let (error, set_error) = signal(None::<String>);

    let on_change = move |ev: leptos::ev::Event| {
        let role = event_target_value(&ev);
        if role.is_empty() || busy.get_untracked() {
            return;
        }
        set_busy.set(true);
        set_error.set(None);
        leptos::task::spawn_local(async move {
            match AuthService::switch_active_role(&role).await {
                Ok(tokens) => {
                    AuthService::update_session_token(&tokens);
                    // Menus, guards and every page's data were resolved for the
                    // role we just left; a soft re-render would show them stale.
                    #[cfg(target_arch = "wasm32")]
                    if let Some(window) = web_sys::window() {
                        let _ = window.location().set_href("/portal/dashboard");
                    }
                }
                Err(message) => {
                    set_error.set(Some(message));
                    set_busy.set(false);
                }
            }
        });
    };

    view! {
        <div class="hidden md:flex flex-col items-start">
            <label for="role-switcher" class="text-[10px] uppercase tracking-wide text-navy-200">
                "Bertindak sebagai"
            </label>
            <select
                id="role-switcher"
                class="bg-white/10 text-white text-xs rounded-md border border-white/20 px-2 py-1 focus:outline-none"
                on:change=on_change
                prop:disabled=move || busy.get()
            >
                {options
                    .into_iter()
                    .map(|(value, label, active)| {
                        view! {
                            <option value=value selected=active class="text-navy-900">
                                {label}
                            </option>
                        }
                    })
                    .collect_view()}
            </select>
            {move || {
                error
                    .get()
                    .map(|message| {
                        view! {
                            <p role="alert" class="text-[10px] text-red-300 mt-0.5">
                                {message}
                            </p>
                        }
                    })
            }}
        </div>
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session(assigned: &[&str], roles: &[&str], active: Option<&str>) -> UserSession {
        UserSession {
            assigned_roles: assigned.iter().map(|r| r.to_string()).collect(),
            roles: roles.iter().map(|r| r.to_string()).collect(),
            active_role: active.map(str::to_string),
            ..Default::default()
        }
    }

    #[test]
    fn nothing_is_offered_with_a_single_assigned_role() {
        let s = session(
            &["operator_satker"],
            &["operator_satker"],
            Some("operator_satker"),
        );
        assert!(s.switchable_roles().is_none());
        assert!(role_options(&s).is_empty());
    }

    #[test]
    fn nothing_is_offered_when_every_role_is_already_in_the_token() {
        // Enforcement off: `roles` is the full assigned set, so a switch would be
        // refused by the issuer. Do not render a control that cannot work.
        let both = ["operator_satker", "validator_satker"];
        let s = session(&both, &both, Some("operator_satker"));
        assert!(s.switchable_roles().is_none());
    }

    #[test]
    fn the_active_role_is_preselected_when_enforcement_is_on() {
        let s = session(
            &["operator_satker", "validator_satker"],
            &["validator_satker"],
            Some("validator_satker"),
        );
        let options = role_options(&s);
        assert_eq!(options.len(), 2);
        let selected: Vec<_> = options
            .iter()
            .filter(|o| o.2)
            .map(|o| o.0.as_str())
            .collect();
        assert_eq!(selected, ["validator_satker"]);
        assert!(
            options.iter().all(|o| !o.1.is_empty()),
            "every role has a label"
        );
    }

    #[test]
    fn an_old_session_without_active_role_falls_back_to_the_token_role() {
        let s = session(
            &["operator_satker", "validator_satker"],
            &["operator_satker"],
            None,
        );
        let selected: Vec<_> = role_options(&s)
            .into_iter()
            .filter(|o| o.2)
            .map(|o| o.0)
            .collect();
        assert_eq!(selected, ["operator_satker"]);
    }
}
