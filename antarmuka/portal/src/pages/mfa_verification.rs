//! MFA Verification page component
//!
//! Handles Multi-Factor Authentication verification during login

use crate::components::layout::AuthLayout;
use crate::features::auth::AuthService;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router;
use lib_ui::prelude::*;
use serde::{Deserialize, Serialize};
use std::cell::Cell;
use std::rc::Rc;

/// MFA verification request body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaVerificationRequest {
    /// OTP code from authenticator
    pub code: String,
}

/// MFA verification response from authenc API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaVerificationResponse {
    /// Success status
    pub success: bool,
    /// Response data
    pub data: MfaVerificationData,
    /// Response message
    pub message: String,
}

/// MFA verification data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaVerificationData {
    /// JWT access token
    pub access_token: String,
    /// User session data
    pub user: serde_json::Value,
}

/// API error response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    /// Success status (false for errors)
    pub success: bool,
    /// Error details
    pub error: ApiError,
}

/// API error details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
}

/// MFA Verification page
#[component]
pub fn MfaVerificationPage() -> impl IntoView {
    let (otp_code, set_otp_code) = signal(String::new());
    let (error_message, set_error_message) = signal(String::new());
    let (is_loading, set_is_loading) = signal(false);
    let (attempts_remaining, set_attempts_remaining) = signal(3);
    let (is_locked, set_is_locked) = signal(false);

    let navigate = leptos_router::hooks::use_navigate();
    let navigate_clone = navigate.clone();

    // Track whether this component is still mounted so the timer callback
    // inside spawn_local can skip navigation after the user left the page.
    let mounted = Rc::new(Cell::new(true));
    let mounted_cleanup = mounted.clone();
    on_cleanup(move || {
        mounted_cleanup.set(false);
    });

    // Obtain the top-level user_session writer so we can update the reactive
    // signal after successful MFA verification.  Without this, route guards
    // that read the signal would still see `None` and redirect to login.
    let set_user_session = use_context::<WriteSignal<Option<crate::features::auth::UserSession>>>();

    // Get temp token from localStorage and store in signal
    let (temp_token_value, _set_temp_token_value) = signal(AuthService::get_temp_token());

    view! {
        <AuthLayout>
            <div class="w-full max-w-md">
                // Header
                <div class="text-center mb-8">
                    <div class="inline-flex items-center justify-center w-20 h-20 bg-gradient-to-br from-emerald-600 to-emerald-700 rounded-full shadow-lg mb-4">
                        <span class="text-4xl">"🔐"</span>
                    </div>
                    <h1 class="text-3xl font-bold text-gray-900 dark:text-white mb-2">
                        "Two-Factor Authentication"
                    </h1>
                    <p class="text-gray-600 dark:text-gray-400">
                        "Enter the verification code from your authenticator app"
                    </p>
                </div>

                // Main Card
                <div class="bg-white dark:bg-gray-800 rounded-xl shadow-lg p-8">
                    <Show when=move || is_locked.get()>
                        <div class="text-center p-4">
                            <div class="inline-flex items-center justify-center w-16 h-16 bg-red-100 rounded-full mb-4">
                                <svg class="w-8 h-8 text-red-600" fill="currentColor" viewBox="0 0 20 20">
                                    <path fill-rule="evenodd" d="M5 9V7a5 5 0 0110 0v2a2 2 0 012 2v5a2 2 0 01-2 2H5a2 2 0 01-2-2v-5a2 2 0 012-2zm8-2v2H7V7a3 3 0 016 0z" clip-rule="evenodd"/>
                                </svg>
                            </div>
                            <h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-2">
                                "Account Temporarily Locked"
                            </h3>
                            <p class="text-gray-600 dark:text-gray-400 mb-4">
                                "Too many failed attempts. Please try again in 15 minutes or contact support."
                            </p>
                            <button
                                type="button"
                                class="inline-flex items-center px-4 py-2 text-sm font-medium text-emerald-700 bg-emerald-100 hover:bg-emerald-200 rounded-md"
                                on:click=move |_| {
                                    if let Some(window) = web_sys::window() {
                                        let _ = window.location().set_href("/login");
                                    }
                                }
                            >
                                "Back to Login"
                            </button>
                        </div>
                    </Show>

                    <Show when=move || !is_locked.get()>
                        <div class="space-y-6">
                            // User info
                            <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
                                <div class="flex items-center">
                                    <div class="flex-shrink-0">
                                        <div class="w-10 h-10 bg-emerald-600 rounded-full flex items-center justify-center">
                                            <span class="text-white font-medium">"U"</span>
                                        </div>
                                    </div>
                                    <div class="ml-3">
                                        <p class="text-sm font-medium text-gray-900 dark:text-white">
                                            "Logged in as: user@kejaksaan.go.id"
                                        </p>
                                        <p class="text-xs text-gray-500 dark:text-gray-400">
                                            "Complete verification to access your account"
                                        </p>
                                    </div>
                                </div>
                            </div>

                            // OTP Input
                            <div>
                                <OtpInput
                                    label="Verification Code".to_string()
                                    value=otp_code
                                    on_change=set_otp_code
                                    disabled=is_loading.get()
                                    loading=is_loading.get()
                                    hint="Enter the 6-digit code from your authenticator app".to_string()
                                    error=if error_message.get().is_empty() { String::new() } else { error_message.get() }
                                />
                            </div>

                            // Attempts remaining
                            <Show when=move || { let attempts = attempts_remaining.get(); attempts < 3 && attempts > 0 }>
                                <div class="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-3">
                                    <div class="flex items-center">
                                        <svg class="w-5 h-5 text-yellow-600 mr-2" fill="currentColor" viewBox="0 0 20 20">
                                            <path fill-rule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clip-rule="evenodd"/>
                                        </svg>
                                        <span class="text-sm text-yellow-800 dark:text-yellow-300">
                                            {move || format!("{} attempts remaining", attempts_remaining.get())}
                                        </span>
                                    </div>
                                </div>
                            </Show>

                            // Verify Button
                            <button
                                type="button"
                                disabled=otp_code.get().len() != 6 || is_loading.get()
                                class="w-full inline-flex items-center justify-center px-6 py-3 text-lg font-medium text-white bg-emerald-700 hover:bg-emerald-800 disabled:opacity-50 disabled:cursor-not-allowed rounded-lg focus:outline-none focus:ring-2 focus:ring-emerald-500 focus:ring-offset-2"
                                on:click={
                                    let navigate_clone = navigate_clone.clone();
                                    let is_mounted = mounted.clone();
                                    let set_user_session = set_user_session.clone();
                                    let otp_code = otp_code;
                                    let temp_token_value = temp_token_value;
                                    move |_| {
                                        let code = otp_code.get();
                                        if code.len() != 6 {
                                            set_error_message.set("Please enter a 6-digit code".to_string());
                                            return;
                                        }

                                        // Check if temp_token is available
                                        let temp_token = match temp_token_value.get() {
                                            Some(token) => token,
                                            None => {
                                                set_error_message.set("No authentication token found. Please log in again.".to_string());
                                                return;
                                            }
                                        };

                                        set_is_loading.set(true);
                                        set_error_message.set(String::new());

                                        let nav = navigate_clone.clone();
                                        let is_mounted = is_mounted.clone();
                                        let set_user_session = set_user_session.clone();
                                        spawn_local(async move {
                                            match verify_mfa_code(&temp_token, &code).await {
                                                Ok(response) => {
                                                    // Store access token and upgrade session
                                                    #[cfg(target_arch = "wasm32")]
                                                    {
                                                        use crate::features::auth::AuthService;

                                                        // Save the access token
                                                        AuthService::save_token(&response.data.access_token);

                                                        // Decode JWT and create session
                                                        match AuthService::decode_jwt_claims(&response.data.access_token) {
                                                            Ok(mut session) => {
                                                                // Mark MFA as enabled in session
                                                                session.mfa_enabled = true;
                                                                session.mfa_setup_required = false;
                                                                session.access_token = Some(response.data.access_token);

                                                                // Save session
                                                                AuthService::save_session(&session);
                                                                crate::utils::app_state::app_state_login(session.clone());

                                                                // Update the reactive user_session signal so route
                                                                // guards see the authenticated session immediately.
                                                                if let Some(setter) = set_user_session {
                                                                    setter.set(Some(session.clone()));
                                                                }

                                                                // Clear temp token
                                                                AuthService::clear_temp_token();

                                                                // Navigate directly to password-change page when
                                                                // required, avoiding a double redirect via the
                                                                // dashboard route guard.
                                                                if is_mounted.get() {
                                                                    if session.require_password_change {
                                                                        nav("/password", Default::default());
                                                                    } else {
                                                                        nav("/dashboard", Default::default());
                                                                    }
                                                                }
                                                            }
                                                            Err(e) => {
                                                                set_error_message.set(format!("Failed to decode token: {}", e));
                                                                set_is_loading.set(false);
                                                            }
                                                        }
                                                    }

                                                    #[cfg(not(target_arch = "wasm32"))]
                                                    {
                                                        let _ = (response, nav, is_mounted, set_user_session);
                                                        set_error_message.set("Session management not available in non-WASM environment".to_string());
                                                        set_is_loading.set(false);
                                                    }
                                                }
                                                Err(e) => {
                                                    set_error_message.set(format!("Verification failed: {}", e));
                                                    set_is_loading.set(false);

                                                    // Decrease attempts
                                                    let remaining = attempts_remaining.get() - 1;
                                                    set_attempts_remaining.set(remaining);

                                                    if remaining == 0 {
                                                        set_is_locked.set(true);
                                                    }

                                                    // Clear the code for retry
                                                    set_otp_code.set(String::new());
                                                }
                                            }
                                        });
                                    }
                                }
                            >
                                {move || if is_loading.get() {
                                    view! {
                                        <svg class="animate-spin -ml-1 mr-2 h-5 w-5" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                                            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                                            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 714 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                                        </svg>
                                        "Verifying..."
                                    }.into_any()
                                } else {
                                    view! { "Verify & Continue" }.into_any()
                                }}
                            </button>

                            // Alternative options
                            <div class="space-y-3">
                                <div class="relative">
                                    <div class="absolute inset-0 flex items-center">
                                        <div class="w-full border-t border-gray-300 dark:border-gray-600"></div>
                                    </div>
                                    <div class="relative flex justify-center text-sm">
                                        <span class="px-2 bg-white dark:bg-gray-800 text-gray-500">"Having trouble?"</span>
                                    </div>
                                </div>

                                <div class="grid grid-cols-1 gap-3">
                                    <button
                                        type="button"
                                        class="inline-flex items-center justify-center px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded-md hover:bg-gray-50 dark:hover:bg-gray-600"
                                        on:click={
                                            let navigate = navigate.clone();
                                            move |_| {
                                                navigate("/mfa/backup-verify", Default::default());
                                            }
                                        }
                                    >
                                        <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1721 9z"/>
                                        </svg>
                                        "Use backup code"
                                    </button>

                                    <button
                                        type="button"
                                        class="inline-flex items-center justify-center px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded-md hover:bg-gray-50 dark:hover:bg-gray-600"
                                        on:click=move |_| {
                                            if let Some(window) = web_sys::window() {
                                                let _ = window.location().set_href("/login");
                                            }
                                        }
                                    >
                                        <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 19l-7-7m0 0l7-7m-7 7h18"/>
                                        </svg>
                                        "Back to login"
                                    </button>
                                </div>
                            </div>
                        </div>
                    </Show>
                </div>

                // Help section
                <div class="mt-6 text-center">
                    <p class="text-sm text-gray-500 dark:text-gray-400">
                        "Lost your device? "
                        <a href="#" class="text-emerald-600 hover:text-emerald-500 font-medium">
                            "Contact support"
                        </a>
                    </p>
                </div>
            </div>
        </AuthLayout>
    }
}

// ============================================================================
// API FUNCTIONS
// ============================================================================

/// Get authenc API base URL (origin for same-origin requests via gateway)
#[cfg(target_arch = "wasm32")]
fn get_api_url() -> String {
    web_sys::window()
        .and_then(|w| w.location().origin().ok())
        .unwrap_or_else(|| "http://localhost:8080".to_string())
}

/// Verify MFA code with authenc API
async fn verify_mfa_code(
    temp_token: &str,
    code: &str,
) -> Result<MfaVerificationResponse, Box<dyn std::error::Error>> {
    #[cfg(target_arch = "wasm32")]
    {
        use gloo_net::http::Request;

        let api_url = get_api_url();
        let verify_url = format!("{}/api/v1/auth/mfa/verify", api_url);

        // Prepare request body
        let request_body = MfaVerificationRequest {
            code: code.to_string(),
        };

        // Make API request
        let response = Request::post(&verify_url)
            .header("Content-Type", "application/json")
            .header("Authorization", &format!("Bearer {}", temp_token))
            .json(&request_body)?
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if response.ok() {
            // Parse success response
            let verification_response: MfaVerificationResponse = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;

            Ok(verification_response)
        } else {
            // Try to parse error response
            match response.json::<ApiErrorResponse>().await {
                Ok(error_response) => Err(error_response.error.message.into()),
                Err(_) => {
                    Err(format!("MFA verification failed: HTTP {}", response.status()).into())
                }
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (temp_token, code); // Suppress unused warnings
        Err("MFA verification not available in non-WASM environment".into())
    }
}
