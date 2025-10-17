//! MFA Setup page component
//!
//! Handles initial Multi-Factor Authentication setup for users

use crate::components::layout::AuthLayout;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use shared_microfrontend::prelude::*;
use wasm_bindgen_futures::spawn_local;

/// MFA setup data from API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaSetupData {
    /// QR code URL for authenticator app
    pub qr_code_url: String,
    /// Secret key for manual entry
    pub secret_key: String,
    /// Generated backup codes
    pub backup_codes: Vec<String>,
}

/// MFA Setup page
#[component]
pub fn MfaSetupPage() -> impl IntoView {
    let (mfa_data, set_mfa_data) = signal(None::<MfaSetupData>);
    let (otp_code, set_otp_code) = signal(String::new());
    let (setup_complete, set_setup_complete) = signal(false);
    let (error_message, set_error_message) = signal(String::new());
    let (is_loading, set_is_loading) = signal(false);
    let (is_generating, set_is_generating) = signal(true);

    let navigate = leptos_router::hooks::use_navigate();
    let navigate_clone = navigate.clone();

    // Generate MFA setup data on component mount
    Effect::new(move |_| {
        spawn_local(async move {
            match generate_mfa_setup().await {
                Ok(data) => {
                    set_mfa_data.set(Some(data));
                    set_is_generating.set(false);
                }
                Err(e) => {
                    set_error_message.set(format!("Failed to generate MFA setup: {}", e));
                    set_is_generating.set(false);
                }
            }
        });
    });

    view! {
        <AuthLayout>
            <div class="w-full max-w-2xl">
                // Header
                <div class="text-center mb-8">
                    <div class="inline-flex items-center justify-center w-20 h-20 bg-gradient-to-br from-emerald-600 to-emerald-700 rounded-full shadow-lg mb-4">
                        <span class="text-4xl">"🔐"</span>
                    </div>
                    <h1 class="text-3xl font-bold text-gray-900 dark:text-white mb-2">
                        "Setup Multi-Factor Authentication"
                    </h1>
                    <p class="text-gray-600 dark:text-gray-400">
                        "Secure your account with an additional layer of protection"
                    </p>
                </div>

                // Main Card
                <div class="bg-white dark:bg-gray-800 rounded-xl shadow-lg p-8">
                    <Show when=move || setup_complete.get()>
                        <div class="text-center p-8">
                            <div class="inline-flex items-center justify-center w-16 h-16 bg-green-100 rounded-full mb-4">
                                <svg class="w-8 h-8 text-green-600" fill="currentColor" viewBox="0 0 20 20">
                                    <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"/>
                                </svg>
                            </div>
                            <h3 class="text-xl font-semibold text-gray-900 dark:text-white mb-2">
                                "MFA Setup Complete!"
                            </h3>
                            <p class="text-gray-600 dark:text-gray-400 mb-4">
                                "Your account is now secured with multi-factor authentication."
                            </p>
                            <div class="animate-pulse text-sm text-gray-500">
                                "Redirecting to dashboard..."
                            </div>
                        </div>
                    </Show>

                    <Show when=move || !setup_complete.get()>
                        <div class="space-y-8">
                            // Step 1: QR Code Display
                            <div>
                                <h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-4">
                                    "Step 1: Scan QR Code"
                                </h2>
                                <p class="text-gray-600 dark:text-gray-400 mb-6">
                                    "Use your authenticator app to scan this QR code. We recommend Google Authenticator, Microsoft Authenticator, or FreeOTP."
                                </p>

                                <Show when=move || is_generating.get()>
                                    <div class="flex justify-center py-12">
                                        <div class="text-center">
                                            <svg class="animate-spin h-12 w-12 text-emerald-600 mx-auto mb-4" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                                                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                                                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                                            </svg>
                                            <p class="text-gray-600">"Generating QR code..."</p>
                                        </div>
                                    </div>
                                </Show>

                                <Show when=move || mfa_data.get().is_some()>
                                    <div>
                                        <div class="flex justify-center mb-6">
                                            <QrCodeDisplay
                                                qr_url=mfa_data.get().map(|d| d.qr_code_url).unwrap_or_default()
                                                size="280px".to_string()
                                            />
                                        </div>

                                        // Manual setup option
                                        <details class="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
                                            <summary class="cursor-pointer text-sm font-medium text-gray-700 dark:text-gray-300 hover:text-emerald-600">
                                                "Can't scan the QR code? Enter manually"
                                            </summary>
                                            <div class="mt-4 space-y-3">
                                                <div>
                                                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                                        "Account Name:"
                                                    </label>
                                                    <code class="block text-sm bg-white dark:bg-gray-800 p-2 rounded border font-mono">
                                                        "SIMPelv2 (your-username@kejaksaan.go.id)"
                                                    </code>
                                                </div>
                                                <div>
                                                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                                        "Secret Key:"
                                                    </label>
                                                    <code class="block text-sm bg-white dark:bg-gray-800 p-2 rounded border font-mono break-all">
                                                        {move || mfa_data.get().map(|d| d.secret_key).unwrap_or_default()}
                                                    </code>
                                                </div>
                                            </div>
                                        </details>
                                    </div>
                                </Show>
                            </div>

                            // Step 2: Verification
                            <div class="border-t pt-8">
                                <h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-4">
                                    "Step 2: Verify Setup"
                                </h2>
                                <p class="text-gray-600 dark:text-gray-400 mb-6">
                                    "Enter the 6-digit code from your authenticator app to complete the setup."
                                </p>

                                <div class="max-w-md mx-auto">
                                    <OtpInput
                                        label="Verification Code".to_string()
                                        value=otp_code
                                        on_change=set_otp_code
                                        disabled=is_loading.get() || mfa_data.get().is_none()
                                        loading=is_loading.get()
                                        hint="Enter the 6-digit code from your authenticator app".to_string()
                                        error=if error_message.get().is_empty() { String::new() } else { error_message.get() }
                                    />

                                    <button
                                        type="button"
                                        disabled=otp_code.get().len() != 6 || is_loading.get() || mfa_data.get().is_none()
                                        class="mt-6 w-full inline-flex items-center justify-center px-6 py-3 text-lg font-medium text-white bg-emerald-700 hover:bg-emerald-800 disabled:opacity-50 disabled:cursor-not-allowed rounded-lg focus:outline-none focus:ring-2 focus:ring-emerald-500 focus:ring-offset-2"
                                        on:click={
                                            let navigate_clone = navigate_clone.clone();
                                            move |_| {
                                                let code = otp_code.get();
                                                if code.len() != 6 {
                                                    set_error_message.set("Please enter a 6-digit code".to_string());
                                                    return;
                                                }

                                                set_is_loading.set(true);
                                                set_error_message.set(String::new());

                                                let set_setup_complete = set_setup_complete;
                                                let set_error_message = set_error_message;
                                                let set_is_loading = set_is_loading;
                                                let navigate = navigate_clone.clone();

                                                spawn_local(async move {
                                                    match verify_mfa_setup(&code).await {
                                                        Ok(_) => {
                                                            set_setup_complete.set(true);
                                                            // Update session to mark MFA as enabled
                                                            crate::features::auth::AuthService::update_session_mfa_enabled();
                                                            // Redirect to dashboard after 2 seconds
                                                            gloo_timers::future::TimeoutFuture::new(2000).await;
                                                            navigate("/dashboard", Default::default());
                                                        }
                                                        Err(e) => {
                                                            set_error_message.set(format!("Verification failed: {}", e));
                                                            set_is_loading.set(false);
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
                                            view! { "Complete MFA Setup" }.into_any()
                                        }}
                                    </button>
                                </div>
                            </div>

                            // Backup codes info
                            <Show when=move || mfa_data.get().is_some()>
                                <div class="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-4">
                                    <div class="flex items-start">
                                        <svg class="w-5 h-5 text-blue-600 mt-0.5 mr-3 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
                                            <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clip-rule="evenodd"/>
                                        </svg>
                                        <div>
                                            <h4 class="text-sm font-medium text-blue-800 dark:text-blue-300">
                                                "Backup Codes Generated"
                                            </h4>
                                            <p class="text-sm text-blue-700 dark:text-blue-400 mt-1">
                                                "After completing setup, you'll receive backup codes that can be used if you lose access to your authenticator app. Store them safely!"
                                            </p>
                                        </div>
                                    </div>
                                </div>
                            </Show>
                        </div>
                    </Show>
                </div>

                // Help section
                <div class="mt-6 text-center">
                    <p class="text-sm text-gray-500 dark:text-gray-400">
                        "Need help? "
                        <a href="#" class="text-emerald-600 hover:text-emerald-500 font-medium">
                            "View MFA setup guide"
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

/// Generate MFA setup data from authenc API
async fn generate_mfa_setup() -> Result<MfaSetupData, Box<dyn std::error::Error>> {
    #[cfg(target_arch = "wasm32")]
    {
        // For demo purposes, return mock data
        // In production, this would call the authenc API
        let mock_data = MfaSetupData {
            qr_code_url: "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==".to_string(),
            secret_key: "JBSWY3DPEHPK3PXP".to_string(),
            backup_codes: vec![
                "12345678".to_string(),
                "87654321".to_string(),
                "11223344".to_string(),
                "44332211".to_string(),
                "55667788".to_string(),
            ],
        };

        // Simulate API delay
        gloo_timers::future::TimeoutFuture::new(1000).await;

        Ok(mock_data)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        Err("MFA setup not available in non-WASM environment".into())
    }
}

/// Verify MFA setup with authenc API
async fn verify_mfa_setup(_code: &str) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_arch = "wasm32")]
    {
        // For demo purposes, accept any 6-digit code
        // In production, this would call the authenc API
        // Simulate API delay
        gloo_timers::future::TimeoutFuture::new(500).await;
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        Err("MFA verification not available in non-WASM environment".into())
    }
}
