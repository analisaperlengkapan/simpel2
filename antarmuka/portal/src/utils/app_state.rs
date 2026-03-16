//! Application State Management
//!
//! Centralized AppState using RwSignal with derived signals for authentication.
//! Provided as context to all components.

use crate::features::auth::UserSession;
use crate::utils::authenc_api::AuthencApiClient;
use leptos::prelude::*;

/// Central application state provided via Leptos context
#[derive(Clone, Debug, Default)]
pub struct AppState {
    /// Current user session (None if not authenticated)
    pub user: Option<UserSession>,
    /// JWT access token
    pub auth_token: Option<String>,
}

impl AppState {
    /// Create AppState from an existing UserSession
    pub fn from_session(session: UserSession) -> Self {
        let token = session.access_token.clone();
        Self {
            user: Some(session),
            auth_token: token,
        }
    }

    /// Check if user is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.user.is_some()
    }

    /// Check if current user is an admin
    pub fn is_admin(&self) -> bool {
        self.user.as_ref().is_some_and(|u| u.role.is_admin())
    }

    /// Get user's display name
    pub fn display_name(&self) -> String {
        self.user
            .as_ref()
            .map(|u| u.name.clone())
            .unwrap_or_else(|| "Pengguna".to_string())
    }

    /// Check if user has a specific permission
    pub fn has_permission(&self, permission: &str) -> bool {
        self.user.as_ref().is_some_and(|u| {
            u.permissions
                .iter()
                .any(|p| p == permission || p == "admin:*")
        })
    }
}

/// Provide the AppState context for the entire application.
///
/// Call this once at the top-level App component.
/// Returns the RwSignal so it can be used in the same scope.
pub fn provide_app_state() -> RwSignal<AppState> {
    // Load existing session from localStorage
    let initial = crate::features::auth::AuthService::load_session()
        .map(AppState::from_session)
        .unwrap_or_default();

    let state = RwSignal::new(initial);
    provide_context(state);

    // Also provide the API client as context
    let api_client = AuthencApiClient::new();
    provide_context(api_client);

    state
}

/// Get the AppState from context
pub fn use_app_state() -> RwSignal<AppState> {
    expect_context::<RwSignal<AppState>>()
}

/// Get the AuthencApiClient from context
pub fn use_api_client() -> AuthencApiClient {
    expect_context::<AuthencApiClient>()
}

/// Derived signal: is the user authenticated?
pub fn use_is_authenticated() -> Signal<bool> {
    let state = use_app_state();
    Signal::derive(move || state.get().is_authenticated())
}

/// Derived signal: is the user an admin?
pub fn use_is_admin() -> Signal<bool> {
    let state = use_app_state();
    Signal::derive(move || state.get().is_admin())
}

/// Derived signal: get the current UserSession (if any)
pub fn use_current_user() -> Signal<Option<UserSession>> {
    let state = use_app_state();
    Signal::derive(move || state.get().user)
}

/// Login: update the AppState with a new session
pub fn app_state_login(session: UserSession) {
    let state = use_app_state();
    state.set(AppState::from_session(session));
}

/// Logout: clear the AppState
pub fn app_state_logout() {
    let state = use_app_state();
    state.set(AppState::default());
}
