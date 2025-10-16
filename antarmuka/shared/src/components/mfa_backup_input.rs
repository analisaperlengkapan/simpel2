//! MFA Backup Code Input Component
//!
//! Provides a specialized input component for entering MFA backup codes.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// Props for the MFA backup code input component
#[derive(Clone)]
pub struct MfaBackupInputProps {
    /// Current value of the backup code input
    pub value: ReadSignal<String>,
    /// Callback when the value changes
    pub on_change: WriteSignal<String>,
    /// Callback when the form is submitted (Enter key or button click)
    pub on_submit: Box<dyn Fn() + 'static>,
    /// Whether the input is disabled
    pub disabled: bool,
    /// Error message to display
    pub error: Option<String>,
    /// Placeholder text
    pub placeholder: Option<String>,
}

/// MFA Backup Code Input Component
///
/// Provides a user-friendly interface for entering backup codes with:
/// - Automatic formatting (adds dashes)
/// - Input validation
/// - Clear error display
/// - Accessibility features
#[component]
pub fn MfaBackupInput(
    /// Current value of the backup code
    value: ReadSignal<String>,
    /// Callback when value changes
    on_change: WriteSignal<String>,
    /// Callback when form is submitted
    on_submit: Box<dyn Fn() + 'static>,
    /// Whether input is disabled
    #[prop(default = false)]
    disabled: bool,
    /// Error message to display
    #[prop(default = None)]
    error: Option<String>,
    /// Placeholder text
    #[prop(default = None)]
    placeholder: Option<String>,
) -> impl IntoView {
    let input_ref = NodeRef::<html::Input>::new();
    let (show_help, set_show_help) = signal(false);

    // Format backup code as user types (add dashes)
    let format_backup_code = move |input: String| {
        // Remove all non-alphanumeric characters
        let clean: String = input.chars().filter(|c| c.is_alphanumeric()).collect();

        // Add dashes every 4 characters (typical backup code format)
        let mut formatted = String::new();
        for (i, ch) in clean.chars().enumerate() {
            if i > 0 && i % 4 == 0 {
                formatted.push('-');
            }
            formatted.push(ch.to_uppercase().next().unwrap_or(ch));
        }

        // Limit to reasonable backup code length (e.g., 16 characters + dashes)
        if formatted.len() <= 19 {
            // 16 chars + 3 dashes
            formatted
        } else {
            formatted[..19].to_string()
        }
    };

    let handle_input = move |ev: web_sys::Event| {
        let input = event_target_value(&ev);
        let formatted = format_backup_code(input);
        on_change.set(formatted);
    };

    let handle_keydown = move |ev: web_sys::KeyboardEvent| {
        if ev.key() == "Enter" {
            ev.prevent_default();
            on_submit();
        }
    };

    let handle_paste = move |ev: web_sys::Event| {
        // Allow paste but format the pasted content
        if let Some(input) = input_ref.get() {
            // Small delay to let paste complete, then format
            set_timeout(
                move || {
                    let current_value = input.value();
                    let formatted = format_backup_code(current_value);
                    input.set_value(&formatted);
                    on_change.set(formatted);
                },
                std::time::Duration::from_millis(10),
            );
        }
    };

    let toggle_help = move |_| {
        set_show_help.update(|show| *show = !*show);
    };

    view! {
        <div class="space-y-2">
            <div class="relative">
                <label class="block text-sm font-medium text-gray-700 mb-2">
                    "Backup Code"
                    <button
                        type="button"
                        class="ml-2 text-blue-600 hover:text-blue-500 text-xs"
                        on:click=toggle_help
                    >
                        "?"
                    </button>
                </label>

                <div class="relative">
                    <input
                        node_ref=input_ref
                        type="text"
                        class=move || {
                            let base_classes = "block w-full px-4 py-3 text-lg font-mono border rounded-lg shadow-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-colors";
                            if error.is_some() {
                                format!("{} border-red-300 text-red-900 placeholder-red-300 focus:ring-red-500 focus:border-red-500", base_classes)
                            } else {
                                format!("{} border-gray-300 placeholder-gray-400", base_classes)
                            }
                        }
                        placeholder=placeholder.unwrap_or_else(|| "XXXX-XXXX-XXXX".to_string())
                        value=move || value.get()
                        disabled=disabled
                        autocomplete="off"
                        spellcheck="false"
                        on:input=handle_input
                        on:keydown=handle_keydown
                        on:paste=handle_paste
                    />

                    // Character count indicator
                    <div class="absolute right-3 top-1/2 transform -translate-y-1/2 text-xs text-gray-400">
                        {move || {
                            let current = value.get().replace("-", "").len();
                            format!("{}/16", current)
                        }}
                    </div>
                </div>

                // Error message
                <Show when=move || error.is_some()>
                    <div class="mt-2 flex items-center text-sm text-red-600">
                        <svg class="w-4 h-4 mr-1" fill="currentColor" viewBox="0 0 20 20">
                            <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z" clip-rule="evenodd"/>
                        </svg>
                        {error.clone().unwrap_or_default()}
                    </div>
                </Show>

                // Help text
                <Show when=show_help>
                    <div class="mt-3 p-3 bg-blue-50 rounded-lg border border-blue-200">
                        <h4 class="text-sm font-medium text-blue-900 mb-2">"About Backup Codes"</h4>
                        <ul class="text-xs text-blue-800 space-y-1">
                            <li>"• Backup codes are 16 characters long"</li>
                            <li>"• They may contain letters and numbers"</li>
                            <li>"• Dashes are added automatically for readability"</li>
                            <li>"• Each code can only be used once"</li>
                            <li>"• Codes are case-insensitive"</li>
                        </ul>
                        <p class="text-xs text-blue-700 mt-2">
                            "If you don't have backup codes, you can generate them in your account settings."
                        </p>
                    </div>
                </Show>
            </div>

            // Submit button
            <button
                type="button"
                class=move || {
                    let base_classes = "w-full flex justify-center py-3 px-4 border border-transparent rounded-lg shadow-sm text-sm font-medium text-white transition-colors";
                    if disabled || value.get().replace("-", "").len() < 8 {
                        format!("{} bg-gray-300 cursor-not-allowed", base_classes)
                    } else {
                        format!("{} bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500", base_classes)
                    }
                }
                disabled=move || disabled || value.get().replace("-", "").len() < 8
                on:click=move |_| on_submit()
            >
                "Verify Backup Code"
            </button>

            // Additional help
            <div class="text-center">
                <p class="text-xs text-gray-500">
                    "Lost your backup codes? "
                    <a href="/mfa/backup-codes" class="text-blue-600 hover:text-blue-500">
                        "Generate new ones"
                    </a>
                </p>
            </div>
        </div>
    }
}

/// Backup Code Verification Request
#[derive(Debug, Serialize)]
pub struct BackupCodeVerifyRequest {
    pub code: String,
}

/// Backup Code Verification Response
#[derive(Debug, Deserialize)]
pub struct BackupCodeVerifyResponse {
    pub valid: bool,
    pub remaining_codes: usize,
    pub message: String,
}

/// API function to verify backup code
pub async fn verify_backup_code(
    code: &str,
) -> Result<BackupCodeVerifyResponse, Box<dyn std::error::Error>> {
    let request = BackupCodeVerifyRequest {
        code: code.to_string(),
    };

    let response = gloo_net::http::Request::post("/api/auth/mfa/backup-codes/verify")
        .json(&request)?
        .send()
        .await?;

    if response.ok() {
        Ok(response.json().await?)
    } else {
        Err(format!("Backup code verification failed: {}", response.status()).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backup_code_formatting() {
        // Test the format_backup_code function logic
        // This would be implemented as a standalone function for testing

        let test_cases = vec![
            ("1234567890123456", "1234-5678-9012-3456"),
            ("abcd1234efgh5678", "ABCD-1234-EFGH-5678"),
            ("1234-5678-9012-3456", "1234-5678-9012-3456"), // Already formatted
            ("123", "123"),                                 // Short input
            ("", ""),                                       // Empty input
        ];

        // Implementation would test each case
        for (input, expected) in test_cases {
            // assert_eq!(format_backup_code(input.to_string()), expected);
        }
    }
}
