//! MFA Backup Code Verification Page
//!
//! Allows users to authenticate using backup codes when TOTP is unavailable.

use crate::components::layout::AuthLayout;
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};

/// Backup code verification request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupCodeVerificationRequest {
    /// Temporary token from MFA setup
    pub temp_token: String,
    /// Backup code to verify
    pub backup_code: String,
}

/// Backup code verification response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupCodeVerificationResponse {
    /// Whether the backup code is valid
    pub valid: bool,
    /// Access token if verification successful
    pub access_token: Option<String>,
    /// Number of remaining backup codes
    pub remaining_codes: usize,
    /// Response message
    pub message: String,
}

/// MFA Backup Code Verification Page
#[component]
pub fn MfaBackupVerificationPage() -> impl IntoView {
    let (backup_code, set_backup_code) = signal(String::new());
    let (error_message, set_error_message) = signal(String::new());
    let (is_loading, set_is_loading) = signal(false);
    let (attempts_remaining, set_attempts_remaining) = signal(5);
    let (is_locked, set_is_locked) = signal(false);

    const TEMP_TOKEN: &str = "temp_token_placeholder";

    let format_backup_code = move |input: String| {
        let clean: String = input.chars().filter(|c| c.is_alphanumeric()).collect();
        let mut formatted = String::new();
        for (i, ch) in clean.chars().enumerate() {
            if i > 0 && i % 4 == 0 {
                formatted.push('-');
            }
            formatted.push(ch.to_uppercase().next().unwrap_or(ch));
        }
        if formatted.len() <= 19 {
            formatted
        } else {
            formatted[..19].to_string()
        }
    };

    let handle_input = move |ev: web_sys::Event| {
        let value = event_target_value(&ev);
        let formatted = format_backup_code(value);
        set_backup_code.set(formatted);
        set_error_message.set(String::new());
    };

    view! {
          <AuthLayout>
              <div class="w-full max-w-md">
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
                          "Enter one of your backup codes to access your account"
                      </p>
                  </div>

                  <Show
                      when=move || is_locked.get()
                      fallback=move || view! {
                          <div class="bg-white dark:bg-gray-800 rounded-xl shadow-lg p-8">
                              <div class="space-y-6">
                                  <div>
                                      <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                          "Backup Code"
                                      </label>
                                      <input
                                          type="text"
                                          class="block w-full px-4 py-3 text-lg font-mono border border-gray-300 dark:border-gray-600 rounded-lg shadow-sm focus:ring-2 focus:ring-amber-500 focus:border-amber-500 dark:bg-gray-700 dark:text-white transition-colors"
                                          placeholder="XXXX-XXXX-XXXX-XXXX"
                                          value=move || backup_code.get()
                                          disabled=is_loading.get()
                                          on:input=handle_input
                                      />
                                      {move || if !error_message.get().is_empty() {
                                          view! {
                                              <p class="mt-2 text-sm text-red-600">
                                                  {error_message.get()}
                                              </p>
                                          }.into_any()
                                      } else {
                                          view! { <span></span> }.into_any()
                                      }}
                                      <p class="mt-2 text-xs text-gray-500 dark:text-gray-400">
                                          "Backup codes are 16 characters long. Dashes are added automatically."
                                      </p>
                                  </div>

                                  {move || {
                                      let attempts = attempts_remaining.get();
                                      if attempts < 5 && attempts > 0 {
                                          view! {
                                              <div class="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-3">
                                                  <p class="text-sm text-yellow-800 dark:text-yellow-300">
                                                      {format!("{} attempts remaining", attempts)}
                                                  </p>
                                              </div>
                                          }.into_any()
                                      } else {
                                          view! { <span></span> }.into_any()
                                      }
                                  }}

                                  <button
                                      type="button"
                                      disabled=move || backup_code.get().replace('-', "").len() < 8 || is_loading.get()
                                      on:click=move |_| {
                                          let code = backup_code.get().replace('-', "");
                                          if code.len() < 8 {
                                              set_error_message.set("Please enter a complete backup code".to_string());
                                              return;
                                          }
                                          set_is_loading.set(true);
                                          set_error_message.set(String::new());
                                          spawn_local(async move {
                                              match verify_backup_code_api(TEMP_TOKEN, &code).await {
                                                  Ok(response) => {
                                                      if response.valid {
                                                          #[cfg(target_arch = "wasm32")]
                                                          {
                                                              gloo_timers::future::TimeoutFuture::new(1500).await;
                                                              if let Some(window) = web_sys::window() {
                                                                  let _ = window.location().set_href("/dashboard");
                                                              }
                                                          }
                                                      } else {
                                                          set_error_message.set(response.message);
                                                          set_is_loading.set(false);
                                                          let remaining = attempts_remaining.get() - 1;
                                                          set_attempts_remaining.set(remaining);
                                                          if remaining == 0 {
                                                              set_is_locked.set(true);
                                                          }
    set_backup_code.set(String::new());
                                                      }
                                                  }
                                                  Err(e) => {
                                                      set_error_message.set(format!("Verification failed: {}", e));
                                                      set_is_loading.set(false);
                                                      let remaining = attempts_remaining.get() - 1;
                                                      set_attempts_remaining.set(remaining);
                                                      if remaining == 0 {
                                                          set_is_locked.set(true);
                                                      }
                                                      set_backup_code.set(String::new());
                                                  }
                                              }
                                          });
                                      }
                                      class="w-full px-6 py-3 bg-amber-600 hover:bg-amber-700 text-white font-medium rounded-lg disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                                  >
                                      {move || if is_loading.get() { "Verifying..." } else { "Verify Backup Code" }}
                                  </button>

                                  <div class="space-y-3">
                                      <div class="relative">
                                          <div class="absolute inset-0 flex items-center">
                                              <div class="w-full border-t border-gray-300 dark:border-gray-600"></div>
                                          </div>
                                          <div class="relative flex justify-center text-sm">
                                              <span class="px-2 bg-white dark:bg-gray-800 text-gray-500">"Other options"</span>
                                          </div>
                                      </div>

                                      <div class="grid grid-cols-1 gap-3">
                                          <a
                                              href="/mfa/verify"
                                              class="inline-flex items-center justify-center px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded-md hover:bg-gray-50 dark:hover:bg-gray-600"
                                          >
                                              "Use authenticator app"
                                          </a>

                                          <a
                                              href="/login"
                                              class="inline-flex items-center justify-center px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded-md hover:bg-gray-50 dark:hover:bg-gray-600"
                                          >
                                              "Back to login"
                                          </a>
                                      </div>
                                  </div>
                              </div>
                          </div>
                      }
                  >
                      <div class="bg-white dark:bg-gray-800 rounded-xl shadow-lg p-8">
                          {move || if is_locked.get() {
                              view! { <div class="text-center p-4">
                                  <div class="inline-flex items-center justify-center w-16 h-16 bg-red-100 rounded-full mb-4">
                                      <svg class="w-8 h-8 text-red-600" fill="currentColor" viewBox="0 0 20 20">
                                          <path fill-rule="evenodd" d="M5 9V7a5 5 0 0110 0v2a2 2 0 012 2v5a2 2 0 01-2 2H5a2 2 0 01-2-2v-5a2 2 0 012-2zm8-2v2H7V7a3 3 0 016 0z" clip-rule="evenodd"/>
                                      </svg>
                                  </div>
                                  <h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-2">
                                      "Account Temporarily Locked"
                                  </h3>
                                  <p class="text-gray-600 dark:text-gray-400 mb-4">
                                      "Too many failed backup code attempts. Please contact support for assistance."
                                  </p>
                                  <a
                                      href="/login"
                                      class="inline-flex items-center px-4 py-2 text-sm font-medium text-amber-700 bg-amber-100 hover:bg-amber-200 rounded-md"
                                  >
                                      "Back to Login"
                                  </a>
                              </div> }.into_any()
                          } else {
                              view! { <span></span> }.into_any()
                          }}
                      </div>
                  </Show>

                  <div class="mt-6 text-center">
                      <p class="text-sm text-gray-500 dark:text-gray-400">
                          "Don't have backup codes? "
                          <a href="/mfa/backup-codes" class="text-amber-600 hover:text-amber-500 font-medium">
                              "Generate them here"
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

/// Verify backup code with authenc API
async fn verify_backup_code_api(
    _temp_token: &str,
    _code: &str,
) -> Result<BackupCodeVerificationResponse, Box<dyn std::error::Error>> {
    #[cfg(target_arch = "wasm32")]
    {
        let request = BackupCodeVerificationRequest {
            temp_token: _temp_token.to_string(),
            backup_code: _code.to_string(),
        };

        let response = gloo_net::http::Request::post("/api/auth/mfa/backup-verify")
            .json(&request)?
            .send()
            .await?;

        if response.ok() {
            Ok(response.json().await?)
        } else {
            Err(format!("Backup code verification failed: {}", response.status()).into())
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        Err("Backup code verification not available in non-WASM environment".into())
    }
}
