//! MFA Backup Code Verification page component
//!
//! Handles Multi-Factor Authentication verification using backup codes during login

use crate::components::layout::AuthLayout;
use crate::features::auth::AuthService;
use leptos::prelude::*;
use leptos_router;
use serde::{Deserialize, Serialize};
use leptos::task::spawn_local;

/// Backup code verification request body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupCodeVerificationRequest {
    /// Backup code (8-digit)
    pub code: String,
}

/// Backup code verification response from authenc API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupCodeVerificationResponse {
    /// Success status
    pub success: bool,
    /// Response data
    pub data: BackupCodeVerificationData,
    /// Response message
    pub message: String,
}

/// Backup code verification data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupCodeVerificationData {
    /// JWT access token
    pub access_token: String,
    /// User session data
    pub user: serde_json::Value,
    /// Number of remaining backup codes
    pub remaining_codes: i32,
}

/// API error response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupVerificationApiErrorResponse {
    /// Success status (false for errors)
    pub success: bool,
    /// Error details
    pub error: BackupVerificationApiError,
}

/// API error details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupVerificationApiError {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
}

/// MFA Backup Code Verification page
#[component]
pub fn MfaBackupVerificationPage() -> impl IntoView {
    let (backup_code, set_backup_code) = signal(String::new());
    let (error_message, set_error_message) = signal(String::new());
    let (is_loading, set_is_loading) = signal(false);
    let (attempts_remaining, set_attempts_remaining) = signal(5);
    let (is_locked, set_is_locked) = signal(false);
    let (remaining_codes, set_remaining_codes) = signal(None::<i32>);
    let (verification_success, set_verification_success) = signal(false);

    let navigate = leptos_router::hooks::use_navigate();

    // Obtain the top-level user_session writer so we can update the reactive
    // signal after successful MFA verification.  Without this, route guards
    // that read the signal would still see `None` and redirect to login.
    let set_user_session = use_context::<WriteSignal<Option<crate::features::auth::UserSession>>>();

    // Track whether this component is still mounted so the timer callback
    // inside spawn_local can skip navigation after the user left the page.
    let mounted = std::rc::Rc::new(std::cell::Cell::new(true));
    let mounted_cleanup = mounted.clone();
    on_cleanup(move || {
        mounted_cleanup.set(false);
    });

    // Get temp token from localStorage and store in signal
    let (temp_token_value, _set_temp_token_value) = signal(AuthService::get_temp_token());

    view! {
        <AuthLayout>
            <div class="w-full max-w-md">
                // Header
                <div class="text-center mb-8">
                    <div class="inline-flex items-center justify-center w-20 h-20 bg-gradient-to-br from-amber-600 to-amber-700 rounded-full shadow-lg mb-4">
                        <svg class="w-10 h-10 text-white" fill="currentColor" viewBox="0 0 20 20">
                            <path fill-rule="evenodd" d="M18 8A6 6 0 006 8v1H3a1 1 0 00-1 1v8a1 1 0 001 1h14a1 1 0 001-1v-8a1 1 0 00-1-1h-3V8zM8 8a4 4 0 118 0v1H8V8z" clip-rule="evenodd"/>
                        </svg>
                    </div>
                    <h1 class="text-3xl font-bold text-gray-900 dark:text-white mb-2">
                        "Backup Code Verification"
                    </h1>
                    <p class="text-gray-600 dark:text-gray-400">
                        "Enter one of your 8-digit backup codes to access your account"
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

                    <Show when=move || !is_locked.get() && !verification_success.get()>
                        <div class="space-y-6">
                            // User info
                            <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
                                <div class="flex items-center">
                                    <div class="flex-shrink-0">
                                        <div class="w-10 h-10 bg-amber-600 rounded-full flex items-center justify-center">
                                            <span class="text-white font-medium">"U"</span>
                                        </div>
                                    </div>
                                    <div class="ml-3">
                                        <p class="text-sm font-medium text-gray-900 dark:text-white">
                                            "Logged in as: user@kejaksaan.go.id"
                                        </p>
                                        <p class="text-xs text-gray-500 dark:text-gray-400">
                                            "Use a backup code to complete verification"
                                        </p>
                                    </div>
                                </div>
                            </div>

                            // Backup Code Input
                            <div>
                                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                    "Backup Code"
                                </label>
                                <input
                                    type="text"
                                    inputmode="numeric"
                                    class="block w-full px-4 py-3 text-lg font-mono text-center border border-gray-300 dark:border-gray-600 rounded-lg shadow-sm focus:ring-2 focus:ring-amber-500 focus:border-amber-500 dark:bg-gray-700 dark:text-white transition-colors"
                                    placeholder="12345678"
                                    maxlength="8"
                                    value=move || backup_code.get()
                                    disabled=is_loading.get()
                                    on:input=move |ev| {
                                        let value = event_target_value(&ev);
                                        // Only allow digits, limit to 8 characters
                                        let clean: String = value.chars().filter(|c| c.is_numeric()).take(8).collect();
                                        set_backup_code.set(clean);
         set_error_message.set(String::new());
                                    }
                                />
                                <Show when=move || !error_message.get().is_empty()>
                                    <p class="mt-2 text-sm text-red-600 dark:text-red-400">
                                        {move || error_message.get()}
                                    </p>
                                </Show>
                                <p class="mt-2 text-xs text-gray-500 dark:text-gray-400">
                                    "Enter the 8-digit backup code from your saved codes"
                                </p>
                            </div>

                            // Attempts remaining
                            <Show when=move || { let attempts = attempts_remaining.get(); attempts < 5 && attempts > 0 }>
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
                                disabled=move || backup_code.get().len() != 8 || is_loading.get()
                                class="w-full inline-flex items-center justify-center px-6 py-3 text-lg font-medium text-white bg-amber-600 hover:bg-amber-700 disabled:opacity-50 disabled:cursor-not-allowed rounded-lg focus:outline-none focus:ring-2 focus:ring-amber-500 focus:ring-offset-2"
                                on:click=move |_| {
                                    let code = backup_code.get();
                                    if code.len() != 8 {
                                        set_error_message.set("Please enter an 8-digit backup code".to_string());
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

                                    let nav = navigate.clone();
                                    let is_mounted = mounted.clone();
                                    spawn_local(async move {
                                        match verify_backup_code(&temp_token, &code).await {
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

                                                            // Show success with remaining codes count
                                                            set_remaining_codes.set(Some(response.data.remaining_codes));
                                                            set_verification_success.set(true);
                                                            set_is_loading.set(false);

                                                            // Redirect to dashboard after showing success.
                                                            // Guard: if the component unmounted during
                                                            // the 2-second timer (e.g. user clicked
                                                            // "Back to login"), skip the navigation.
                                                            gloo_timers::future::TimeoutFuture::new(2000).await;

                                                            if is_mounted.get() {
                                                                nav("/dashboard", Default::default());
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
                                                    let _ = response; // Suppress unused warning
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
                                                set_backup_code.set(String::new());
                                            }
                                        }
                                    });
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
                                    view! { "Verify Backup Code" }.into_any()
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
                                        on:click=move |_| {
                                            navigate("/mfa/verify", Default::default());
                                        }
                                    >
                                        <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"/>
                                        </svg>
                                        "Use authenticator app instead"
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

                    <Show when=move || verification_success.get()>
                        <div class="text-center p-8">
                            <div class="inline-flex items-center justify-center w-16 h-16 bg-green-100 rounded-full mb-4">
                                <svg class="w-8 h-8 text-green-600" fill="currentColor" viewBox="0 0 20 20">
                                    <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"/>
                                </svg>
                            </div>
                            <h3 class="text-xl font-semibold text-gray-900 dark:text-white mb-2">
                                "Verification Successful!"
                            </h3>
                            <p class="text-gray-600 dark:text-gray-400 mb-4">
                                "Your backup code has been verified successfully."
          </p>
                            <Show when=move || remaining_codes.get().is_some()>
                                <div class="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-4 mb-4">
                                    <p class="text-sm text-blue-800 dark:text-blue-300">
                                        {move || format!("You have {} backup codes remaining", remaining_codes.get().unwrap_or(0))}
                                    </p>
                                    <p class="text-xs text-blue-600 dark:text-blue-400 mt-1">
                                        "Consider generating new backup codes if you're running low"
                                    </p>
                                </div>
                            </Show>
                            <div class="animate-pulse text-sm text-gray-500">
                                "Redirecting to dashboard..."
                            </div>
                        </div>
                    </Show>
                </div>

                // Help section
                <div class="mt-6 text-center">
                    <p class="text-sm text-gray-500 dark:text-gray-400">
                        "Lost your backup codes? "
                        <a href="#" class="text-amber-600 hover:text-amber-500 font-medium">
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

/// Verify backup code with authenc API
async fn verify_backup_code(
    temp_token: &str,
    code: &str,
) -> Result<BackupCodeVerificationResponse, Box<dyn std::error::Error>> {
    #[cfg(target_arch = "wasm32")]
    {
        use gloo_net::http::Request;

        let api_url = get_api_url();
        let verify_url = format!("{}/api/v1/auth/mfa/verify-recovery", api_url);

        // Prepare request body
        let request_body = BackupCodeVerificationRequest {
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
            let verification_response: BackupCodeVerificationResponse = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;

            Ok(verification_response)
        } else {
            // Try to parse error response
            match response.json::<BackupVerificationApiErrorResponse>().await {
                Ok(error_response) => Err(error_response.error.message.into()),
                Err(_) => Err(format!(
                    "Backup code verification failed: HTTP {}",
                    response.status()
                )
                .into()),
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (temp_token, code); // Suppress unused warnings
        Err("Backup code verification not available in non-WASM environment".into())
    }
}
