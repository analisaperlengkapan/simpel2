//! MFA Backup Codes Management
//!
//! Provides UI for generating, displaying, and managing MFA backup codes.

use crate::components::layout::MainLayout;
use leptos::prelude::*;
use leptos::task::spawn_local;
use lib_ui::components::{Alert, Loading};
use lib_ui::core::types::AlertVariant;
use serde::{Deserialize, Serialize};

/// Backup codes response from API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupCodesResponse {
    /// List of backup codes
    pub codes: Vec<String>,
    /// Total number of codes
    pub count: usize,
    /// Warning message about backup codes
    pub warning: String,
}

/// Backup code status response from API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupCodeStatusResponse {
    /// Whether backup codes are available
    pub available: bool,
    /// Number of remaining backup codes
    pub remaining_codes: usize,
    /// Timestamp of last code generation
    pub last_generated: Option<String>,
}

/// MFA Backup Codes Management Page Component — reads session from context
#[component]
pub fn MfaBackupCodesPage() -> impl IntoView {
    let (backup_codes, set_backup_codes) = signal(None::<BackupCodesResponse>);
    let (status, set_status) = signal(None::<BackupCodeStatusResponse>);
    let (loading, set_loading) = signal(false);
    let (error_message, set_error_message) = signal(None::<String>);
    let (show_codes, set_show_codes) = signal(false);

    // Load backup code status on component mount
    Effect::new(move |_| {
        spawn_local(async move {
            match get_backup_code_status().await {
                Ok(status_data) => set_status.set(Some(status_data)),
                Err(e) => set_error_message.set(Some(e.to_string())),
            }
        });
    });

    let generate_codes = move || {
        set_loading.set(true);
        set_error_message.set(None);

        spawn_local(async move {
            match generate_backup_codes().await {
                Ok(codes_data) => {
                    set_backup_codes.set(Some(codes_data));
                    set_show_codes.set(true);
                    // Refresh status
                    if let Ok(status_data) = get_backup_code_status().await {
                        set_status.set(Some(status_data));
                    }
                }
                Err(e) => set_error_message.set(Some(e.to_string())),
            }
            set_loading.set(false);
        });
    };

    let download_codes = move || {
        if let Some(codes) = backup_codes.get() {
            let _content = format!(
                "SIMPEL Kejaksaan RI - MFA Backup Codes\n\
                Generated: {}\n\
                \n\
                IMPORTANT: Store these codes in a secure location.\n\
                Each code can only be used once.\n\
                \n\
                Backup Codes:\n\
                {}\n\
                \n\
                If you lose access to your authenticator app, you can use these codes\n\
                to regain access to your account. After using a code, it will be\n\
                permanently disabled.",
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
                codes.codes.join("\n")
            );

            #[cfg(target_arch = "wasm32")]
            {
                use wasm_bindgen::JsCast;

                if let Some(window) = web_sys::window()
                    && let Some(document) = window.document()
                {
                    // Create blob and download
                    let array = js_sys::Array::new();
                    array.push(&wasm_bindgen::JsValue::from_str(&_content));

                    if let Ok(blob) = web_sys::Blob::new_with_str_sequence(&array)
                        && let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob)
                        && let Ok(link) = document.create_element("a")
                    {
                        let _ = link.set_attribute("href", &url);
                        let _ = link.set_attribute("download", "simpelv2-mfa-backup-codes.txt");

                        if let Some(body) = document.body() {
                            let _ = body.append_child(&link);
                            if let Some(html_link) = link.dyn_ref::<web_sys::HtmlElement>() {
                                html_link.click();
                            }
                            let _ = body.remove_child(&link);
                        }

                        let _ = web_sys::Url::revoke_object_url(&url);
                    }
                }
            }
        }
    };

    let print_codes = move || {
        #[cfg(target_arch = "wasm32")]
        {
            if backup_codes.get().is_some()
                && let Some(window) = web_sys::window()
            {
                let _ = window.print();
            }
        }
    };

    view! {
        <MainLayout>
            <div class="container mx-auto px-4 py-8">
                <div class="max-w-4xl mx-auto">
                    <div class="bg-white dark:bg-gray-800 shadow-lg rounded-2xl overflow-hidden">
                        <div class="px-6 py-4 bg-gradient-to-r from-blue-600 to-blue-700 text-white">
                            <h1 class="text-2xl font-bold">"MFA Backup Codes"</h1>
                            <p class="text-blue-100 mt-1">
                                "Manage your multi-factor authentication backup codes"
                            </p>
                        </div>

                        <div class="p-6 space-y-6">
                            // Status Section
                            <Show when=move || {
                                status.get().is_some()
                            }>
                                {move || {
                                    status
                                        .get()
                                        .map(|status_data| {
                                            if status_data.available && status_data.remaining_codes > 0
                                            {
                                                view! {
                                                    <Alert
                                                        variant=AlertVariant::Success
                                                        title="Backup Codes Available".to_string()
                                                        message=format!(
                                                            "You have {} backup codes remaining",
                                                            status_data.remaining_codes,
                                                        )
                                                        show=true
                                                    />
                                                }
                                                    .into_any()
                                            } else {
                                                view! {
                                                    <Alert
                                                        variant=AlertVariant::Warning
                                                        title="No Backup Codes".to_string()
                                                        message="You don't have any backup codes. Generate new codes below."
                                                            .to_string()
                                                        show=true
                                                    />
                                                }
                                                    .into_any()
                                            }
                                        })
                                }}
                            </Show>

                            // Warning Section
                            <Alert
                                variant=AlertVariant::Info
                                title="Important Security Information".to_string()
                                message="• Backup codes are for emergency access only\n• Each code can only be used once\n• Store codes in a secure, offline location\n• Generate new codes if you suspect compromise\n• Do not share codes with anyone"
                                    .to_string()
                                show=true
                            />

                            // Generate Codes Section
                            <div>
                                <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">
                                    "Generate New Backup Codes"
                                </h2>
                                <p class="text-gray-600 dark:text-gray-400 mb-4">
                                    "If you don't have backup codes or need to replace existing ones, generate a new set. "
                                    "This will invalidate any previously generated codes."
                                </p>

                                <button
                                    type="button"
                                    disabled=loading.get()
                                    on:click=move |_| generate_codes()
                                    class="px-6 py-3 bg-blue-600 hover:bg-blue-700 text-white font-medium rounded-lg disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                                >
                                    {move || {
                                        if loading.get() {
                                            "Generating..."
                                        } else {
                                            "Generate New Codes"
                                        }
                                    }}
                                </button>

                                <Show when=move || {
                                    status.get().map(|s| s.remaining_codes > 0).unwrap_or(false)
                                }>
                                    <p class="text-sm text-amber-600 dark:text-amber-400 mt-2">
                                        "⚠️ Warning: This will replace your existing backup codes"
                                    </p>
                                </Show>
                            </div>

                            // Display Generated Codes
                            <Show when=move || {
                                show_codes.get() && backup_codes.get().is_some()
                            }>
                                {move || {
                                    backup_codes
                                        .get()
                                        .map(|codes| {
                                            view! {
                                                <div class="p-6 bg-gray-50 dark:bg-gray-900 rounded-xl border-2 border-dashed border-gray-300 dark:border-gray-600">
                                                    <div class="flex justify-between items-center mb-4">
                                                        <h2 class="text-lg font-semibold text-gray-900 dark:text-white">
                                                            "Your New Backup Codes"
                                                        </h2>
                                                        <div class="flex space-x-2">
                                                            <button
                                                                type="button"
                                                                on:click=move |_| download_codes()
                                                                class="px-4 py-2 bg-gray-200 hover:bg-gray-300 dark:bg-gray-700 dark:hover:bg-gray-600 text-gray-700 dark:text-gray-300 font-medium rounded-lg transition-colors"
                                                            >
                                                                "Download"
                                                            </button>
                                                            <button
                                                                type="button"
                                                                on:click=move |_| print_codes()
                                                                class="px-4 py-2 bg-gray-200 hover:bg-gray-300 dark:bg-gray-700 dark:hover:bg-gray-600 text-gray-700 dark:text-gray-300 font-medium rounded-lg transition-colors"
                                                            >
                                                                "Print"
                                                            </button>
                                                        </div>
                                                    </div>

                                                    <div class="grid grid-cols-1 md:grid-cols-2 gap-3 mb-4">
                                                        {codes
                                                            .codes
                                                            .iter()
                                                            .enumerate()
                                                            .map(|(i, code)| {
                                                                view! {
                                                                    <div class="flex items-center p-3 bg-white dark:bg-gray-800 rounded border border-gray-200 dark:border-gray-700">
                                                                        <span class="text-sm text-gray-500 dark:text-gray-400 mr-3 w-6">
                                                                            {format!("{}.", i + 1)}
                                                                        </span>
                                                                        <code class="font-mono text-lg font-semibold text-gray-900 dark:text-white select-all">
                                                                            {code.clone()}
                                                                        </code>
                                                                    </div>
                                                                }
                                                            })
                                                            .collect_view()}
                                                    </div>

                                                    <Alert
                                                        variant=AlertVariant::Error
                                                        message=codes.warning.clone()
                                                        show=true
                                                    />
                                                </div>
                                            }
                                        })
                                }}
                            </Show>

                            // Error Display
                            <Show when=move || {
                                error_message.get().is_some()
                            }>
                                {move || {
                                    error_message
                                        .get()
                                        .map(|msg| {
                                            view! {
                                                <Alert
                                                    variant=AlertVariant::Error
                                                    title="Error".to_string()
                                                    message=msg
                                                    show=true
                                                />
                                            }
                                        })
                                }}
                            </Show>

                            // Loading State
                            <Show when=move || loading.get()>
                                <Loading text="Generating backup codes...".to_string() />
                            </Show>

                            // Instructions Section
                            <div class="bg-blue-50 dark:bg-blue-900/20 rounded-xl p-6 border border-blue-200 dark:border-blue-800">
                                <h3 class="text-blue-900 dark:text-blue-100 font-semibold mb-3">
                                    "How to Use Backup Codes"
                                </h3>
                                <ol class="text-blue-800 dark:text-blue-200 text-sm space-y-2">
                                    <li class="flex items-start">
                                        <span class="font-semibold mr-2">"1."</span>
                                        <span>
                                            "If you lose access to your authenticator app, go to the login page"
                                        </span>
                                    </li>
                                    <li class="flex items-start">
                                        <span class="font-semibold mr-2">"2."</span>
                                        <span>"Enter your username and password as usual"</span>
                                    </li>
                                    <li class="flex items-start">
                                        <span class="font-semibold mr-2">"3."</span>
                                        <span>
                                            "When prompted for MFA code, click 'Use backup code instead'"
                                        </span>
                                    </li>
                                    <li class="flex items-start">
                                        <span class="font-semibold mr-2">"4."</span>
                                        <span>
                                            "Enter one of your backup codes exactly as shown"
                                        </span>
                                    </li>
                                    <li class="flex items-start">
                                        <span class="font-semibold mr-2">"5."</span>
                                        <span>
                                            "The code will be permanently disabled after use"
                                        </span>
                                    </li>
                                </ol>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </MainLayout>
    }
}

// ============================================================================
// API FUNCTIONS
// ============================================================================

/// Request payload for backup codes management
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
struct MfaBackupCodesRequest {
    /// Action to perform (generate, list)
    action: String,
}

/// Response payload from authenc backup codes API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
struct MfaBackupCodesApiResponse {
    /// Success message
    message: String,
    /// Backup codes (for generate action)
    #[serde(skip_serializing_if = "Option::is_none")]
    codes: Option<Vec<String>>,
    /// Number of remaining codes (for list action)
    #[serde(skip_serializing_if = "Option::is_none")]
    remaining: Option<i32>,
}

/// Get authentication token from storage
#[cfg(target_arch = "wasm32")]
fn get_auth_token() -> Option<String> {
    use crate::features::auth::AuthService;

    // Try to get access_token first (for authenticated users)
    AuthService::get_token()
}

/// Generate new backup codes via authenc API
async fn generate_backup_codes() -> Result<BackupCodesResponse, Box<dyn std::error::Error>> {
    #[cfg(target_arch = "wasm32")]
    {
        // Get authentication token
        let token =
            get_auth_token().ok_or("No authentication token found. Please log in again.")?;

        // Prepare request body
        let request_body = MfaBackupCodesRequest {
            action: "generate".to_string(),
        };

        // Make API request — the access token travels in the Authorization
        // header, which is the only credential path authenc accepts.
        let response = gloo_net::http::Request::post("/api/v1/auth/mfa/backup-codes")
            .header("Authorization", &format!("Bearer {token}"))
            .json(&request_body)?
            .send()
            .await?;

        if response.ok() {
            let api_response: MfaBackupCodesApiResponse = response.json().await?;

            // Convert to frontend response format
            let codes = api_response.codes.unwrap_or_default();
            let count = codes.len();
            Ok(BackupCodesResponse {
                codes,
                count,
                warning: "⚠️ IMPORTANT: Save these codes immediately! Each code can only be used once. Store them in a secure, offline location.".to_string(),
            })
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(format!(
                "Failed to generate backup codes: {} - {}",
                response.status(),
                error_text
            )
            .into())
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        Err("Backup code generation not available in non-WASM environment".into())
    }
}

/// Get backup code status via authenc API
async fn get_backup_code_status() -> Result<BackupCodeStatusResponse, Box<dyn std::error::Error>> {
    #[cfg(target_arch = "wasm32")]
    {
        // Get authentication token
        let token =
            get_auth_token().ok_or("No authentication token found. Please log in again.")?;

        // Prepare request body
        let request_body = MfaBackupCodesRequest {
            action: "list".to_string(),
        };

        // Make API request — see `generate_backup_codes` for why the token goes
        // in the header rather than the body.
        let response = gloo_net::http::Request::post("/api/v1/auth/mfa/backup-codes")
            .header("Authorization", &format!("Bearer {token}"))
            .json(&request_body)?
            .send()
            .await?;

        if response.ok() {
            let api_response: MfaBackupCodesApiResponse = response.json().await?;

            // Convert to frontend response format
            let remaining = api_response.remaining.unwrap_or(0);
            Ok(BackupCodeStatusResponse {
                available: remaining > 0,
                remaining_codes: remaining as usize,
                last_generated: None, // Backend doesn't provide this yet
            })
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(format!(
                "Failed to get backup code status: {} - {}",
                response.status(),
                error_text
            )
            .into())
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        Err("Backup code status not available in non-WASM environment".into())
    }
}
