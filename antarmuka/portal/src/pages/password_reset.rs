//! Password Reset page component
//!
//! Handles password reset flow with CAPTCHA integration

use crate::components::layout::AuthLayout;
use leptos::prelude::*;
use shared_microfrontend::components::captcha::Captcha;
use wasm_bindgen_futures::spawn_local;

/// Password reset flow state
#[derive(Clone, Debug, PartialEq)]
enum ResetState {
    /// Initial state - requesting reset
    RequestReset,
    /// CAPTCHA verification required
    CaptchaVerification,
    /// Email sent confirmation
    EmailSent,
    /// Reset token validation
    #[allow(dead_code)]
    ValidatingToken,
    /// New password entry
    NewPassword,
    /// Reset complete
    Complete,
}

/// Password Reset page
#[component]
pub fn PasswordResetPage() -> impl IntoView {
    let (email, set_email) = signal(String::new());
    let (reset_token, _set_reset_token) = signal(String::new());
    let (new_password, set_new_password) = signal(String::new());
    let (confirm_password, set_confirm_password) = signal(String::new());
    let (captcha_token, set_captcha_token) = signal(None::<String>);
    let (error_message, set_error_message) = signal(String::new());
    let (is_loading, set_is_loading) = signal(false);
    let (reset_state, set_reset_state) = signal(ResetState::RequestReset);

    let navigate = leptos_router::hooks::use_navigate();

    // Check for reset token in URL query params
    Effect::new(move |_| {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                if let Ok(url) = window.location().href() {
                    if let Ok(parsed_url) = url::Url::parse(&url) {
                        if let Some(token) = parsed_url
                            .query_pairs()
                            .find(|(key, _)| key == "token")
                            .map(|(_, value)| value.to_string())
                        {
                            _set_reset_token.set(token);
                            set_reset_state.set(ResetState::NewPassword);
                        }
                    }
                }
            }
        }
    });

    // Handle CAPTCHA completion
    let handle_captcha_success = move |token: String| {
        set_captcha_token.set(Some(token));
        set_error_message.set(String::new());
    };

    let handle_captcha_failure = move |error: String| {
        set_error_message.set(format!("CAPTCHA verification failed: {}", error));
        set_captcha_token.set(None);
    };

    // Handle reset request submission
    let handle_request_reset = {
        move |ev: web_sys::SubmitEvent| {
            ev.prevent_default();

            // Validate email
            if email.get().trim().is_empty() {
                set_error_message.set("Please enter your email address".to_string());
                return;
            }

            // CAPTCHA is required for password reset
            if captcha_token.get().is_none() {
                set_error_message
                    .set("Please complete the security verification first.".to_string());
                return;
            }

            set_is_loading.set(true);
            set_error_message.set(String::new());

            let email_val = email.get();
            let captcha_val = captcha_token.get();

            spawn_local(async move {
                match request_password_reset(&email_val, captcha_val.as_deref()).await {
                    Ok(_) => {
                        set_reset_state.set(ResetState::EmailSent);
                        set_is_loading.set(false);
                    }
                    Err(e) => {
                        set_error_message.set(format!("Failed to request password reset: {}", e));
                        set_is_loading.set(false);
                        set_captcha_token.set(None); // Reset CAPTCHA on failure
                    }
                }
            });
        }
    };

    // Handle new password submission
    let handle_reset_password = {
        let navigate = navigate.clone();

        move |ev: web_sys::SubmitEvent| {
            ev.prevent_default();

            // Validate passwords
            if new_password.get().is_empty() {
                set_error_message.set("Please enter a new password".to_string());
                return;
            }

            if new_password.get() != confirm_password.get() {
                set_error_message.set("Passwords do not match".to_string());
                return;
            }

            if new_password.get().len() < 8 {
                set_error_message.set("Password must be at least 8 characters long".to_string());
                return;
            }

            set_is_loading.set(true);
            set_error_message.set(String::new());

            let token_val = reset_token.get();
            let password_val = new_password.get();
            let navigate = navigate.clone();

            spawn_local(async move {
                match reset_password(&token_val, &password_val).await {
                    Ok(_) => {
                        set_reset_state.set(ResetState::Complete);
                        set_is_loading.set(false);
                        // Redirect to login after 3 seconds
                        gloo_timers::future::TimeoutFuture::new(3000).await;
                        navigate("/login", Default::default());
                    }
                    Err(e) => {
                        set_error_message.set(format!("Failed to reset password: {}", e));
                        set_is_loading.set(false);
                    }
                }
            });
        }
    };

    view! {
        <AuthLayout>
            <div class="w-full max-w-md">
                // Header
                <div class="text-center mb-8 animate-fade-in">
                    <div class="inline-flex items-center justify-center w-20 h-20 bg-gradient-to-br from-blue-600 to-blue-700 rounded-full shadow-lg mb-4">
                        <span class="text-4xl">"🔑"</span>
                    </div>
                    <h1 class="text-3xl font-bold text-gray-900 dark:text-white mb-2">
                        "Reset Password"
                    </h1>
                    <p class="text-gray-600 dark:text-gray-400">
                        {move || match reset_state.get() {
                            ResetState::RequestReset | ResetState::CaptchaVerification => "Enter your email to receive reset instructions",
                            ResetState::EmailSent => "Check your email for reset link",
                            ResetState::NewPassword => "Enter your new password",
                            ResetState::Complete => "Password reset successful",
                            _ => ""
                        }}
                    </p>
                </div>

                // Main Card
                <div class="bg-white dark:bg-gray-800 rounded-xl shadow-lg p-8">
                    <Show when=move || reset_state.get() == ResetState::RequestReset || reset_state.get() == ResetState::CaptchaVerification>
                        <form on:submit=handle_request_reset class="space-y-6">
                            // Email Input
                            <div>
                                <label for="email" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                    "Email Address"
                                </label>
                                <input
                                    type="email"
                                    id="email"
                                    name="email"
                                    class="w-full px-4 py-3 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white transition-colors"
                                    placeholder="your.email@kejaksaan.go.id"
                                    prop:value=move || email.get()
                                    on:input=move |ev| set_email.set(event_target_value(&ev))
                                    required
                                    disabled=move || is_loading.get()
                                />
                            </div>

                            // CAPTCHA Component (always required for password reset)
                            <div class="captcha-section">
                                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                    "Security Verification"
                                </label>
                                <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
                                    <Captcha
                                        on_success=Callback::new(handle_captcha_success)
                                        on_failure=Callback::new(handle_captcha_failure)
                                        difficulty=4u8
                                        accessibility_enabled=true
                                        behavioral_analysis=true
                                        class="captcha-password-reset"
                                    />
                                </div>
                                <p class="mt-2 text-sm text-gray-500 dark:text-gray-400">
                                    "Security verification is required to prevent automated attacks"
                                </p>
                            </div>

                            // Error Message
                            {move || (!error_message.get().is_empty()).then(|| view! {
                                <div class="bg-red-50 dark:bg-red-900/20 border-l-4 border-red-500 text-red-700 dark:text-red-400 px-4 py-3 rounded">
                                    <div class="flex items-center">
                                        <svg class="w-5 h-5 mr-2" fill="currentColor" viewBox="0 0 20 20">
                                            <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z" clip-rule="evenodd"/>
                                        </svg>
                                        <span class="font-medium">{error_message.get()}</span>
                                    </div>
                                </div>
                            })}

                            // Submit Button
                            <button
                                type="submit"
                                class=move || format!(
                                    "inline-flex items-center justify-center font-medium transition-colors focus:outline-none focus:ring-2 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed px-6 py-3 text-lg rounded-lg w-full {}",
                                    if captcha_token.get().is_none() {
                                        "bg-gray-400 hover:bg-gray-400 text-gray-700 focus:ring-gray-400 cursor-not-allowed"
                                    } else {
                                        "bg-blue-700 hover:bg-blue-800 text-white focus:ring-blue-500"
                                    }
                                )
                                disabled=move || is_loading.get() || captcha_token.get().is_none()
                            >
                                <Show
                                    when=move || is_loading.get()
                                    fallback=move || {
                                        if captcha_token.get().is_none() {
                                            view! { "Complete Security Verification" }
                                        } else {
                                            view! { "Send Reset Link" }
                                        }
                                    }
                                >
                                    <svg class="animate-spin -ml-1 mr-3 h-5 w-5 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                                        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                                        <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                                    </svg>
                                    "Sending..."
                                </Show>
                            </button>
                        </form>
                    </Show>

                    <Show when=move || reset_state.get() == ResetState::EmailSent>
                        <div class="text-center p-8">
                            <div class="inline-flex items-center justify-center w-16 h-16 bg-blue-100 rounded-full mb-4">
                                <svg class="w-8 h-8 text-blue-600" fill="currentColor" viewBox="0 0 20 20">
                                    <path d="M2.003 5.884L10 9.882l7.997-3.998A2 2 0 0016 4H4a2 2 0 00-1.997 1.884z"/>
                                    <path d="M18 8.118l-8 4-8-4V14a2 2 0 002 2h12a2 2 0 002-2V8.118z"/>
                                </svg>
                            </div>
                            <h3 class="text-xl font-semibold text-gray-900 dark:text-white mb-2">
                                "Check Your Email"
                            </h3>
                            <p class="text-gray-600 dark:text-gray-400 mb-4">
                                "We've sent password reset instructions to "
                                <strong>{move || email.get()}</strong>
                            </p>
                            <p class="text-sm text-gray-500 dark:text-gray-400">
                                "The link will expire in 1 hour. If you don't receive the email, check your spam folder."
                            </p>
                        </div>
                    </Show>

                    {move || (reset_state.get() == ResetState::NewPassword).then(|| view! {
                        <form on:submit=handle_reset_password.clone() class="space-y-6">
                            // New Password Input
                            <div>
                                <label for="new_password" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                    "New Password"
                                </label>
                                <input
                                    type="password"
                                    id="new_password"
                                    name="new_password"
                    class="w-full px-4 py-3 bordeorder-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white transition-colors"
                                    placeholder="Enter new password"
                                    prop:value=move || new_password.get()
                                    on:input=move |ev| set_new_password.set(event_target_value(&ev))
                                    required
                                    disabled=move || is_loading.get()
                                />
                                <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
                                    "Minimum 8 characters"
                                </p>
                            </div>

                            // Confirm Password Input
                            <div>
                                <label for="confirm_password" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                    "Confirm Password"
                                </label>
                                <input
                                    type="password"
                                    id="confirm_password"
                                    name="confirm_password"
                                    class="w-full px-4 py-3 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white transition-colors"
                                    placeholder="Confirm new password"
                                    prop:value=move || confirm_password.get()
                                    on:input=move |ev| set_confirm_password.set(event_target_value(&ev))
                                    required
                                    disabled=move || is_loading.get()
                                />
                            </div>

                            // Error Message
                            {move || (!error_message.get().is_empty()).then(|| view! {
                                <div class="bg-red-50 dark:bg-red-900/20 border-l-4 border-red-500 text-red-700 dark:text-red-400 px-4 py-3 rounded">
                                    <div class="flex items-center">
                                        <svg class="w-5 h-5 mr-2" fill="currentColor" viewBox="0 0 20 20">
                                            <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z" clip-rule="evenodd"/>
                                        </svg>
                                        <span class="font-medium">{error_message.get()}</span>
                                    </div>
                                </div>
                            })}

                            // Submit Button
                            <button
                                type="submit"
                                class="w-full inline-flex items-center justify-center px-6 py-3 text-lg font-medium text-white bg-blue-700 hover:bg-blue-800 disabled:opacity-50 disabled:cursor-not-allowed rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2"
                                disabled=move || is_loading.get()
                            >
                                <Show
                                    when=move || is_loading.get()
                                    fallback=move || view! { "Reset Password" }
                                >
                                    <svg class="animate-spin -ml-1 mr-3 h-5 w-5 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                                        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                                        <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                                    </svg>
                                    "Resetting..."
                                </Show>
                            </button>
                        </form>
                    })}

                    <Show when=move || reset_state.get() == ResetState::Complete>
                        <div class="text-center p-8">
                            <div class="inline-flex items-center justify-center w-16 h-16 bg-green-100 rounded-full mb-4">
                                <svg class="w-8 h-8 text-green-600" fill="currentColor" viewBox="0 0 20 20">
                                    <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"/>
                                </svg>
                            </div>
                            <h3 class="text-xl font-semibold text-gray-900 dark:text-white mb-2">
                                "Password Reset Complete!"
                            </h3>
                            <p class="text-gray-600 dark:text-gray-400 mb-4">
                                "Your password has been successfully reset."
                            </p>
                            <div class="animate-pulse text-sm text-gray-500">
                                "Redirecting to login..."
                            </div>
                        </div>
                    </Show>
                </div>

                // Back to login link
                <div class="mt-6 text-center">
                    <a href="/login" class="text-sm text-blue-600 hover:text-blue-500 font-medium">
                        "← Back to Login"
                    </a>
                </div>
            </div>
        </AuthLayout>
    }
}

// ============================================================================
// API FUNCTIONS
// ============================================================================

/// Get authenc API base URL
#[allow(dead_code)]
fn get_authenc_api_url() -> String {
    std::env::var("AUTHENC_API_URL").unwrap_or_else(|_| "http://localhost:3000".to_string())
}

/// Request password reset
async fn request_password_reset(
    email: &str,
    captcha_token: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_arch = "wasm32")]
    {
        use gloo_net::http::Request;

        let api_url = get_authenc_api_url();
        let reset_url = format!("{}/api/auth/password-reset/request", api_url);

        let body = serde_json::json!({
            "email": email,
            "captcha_token": captcha_token,
        });

        let response = Request::post(&reset_url)
            .header("Content-Type", "application/json")
            .json(&body)?
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if response.ok() {
            Ok(())
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(format!("HTTP {}: {}", response.status(), error_text).into())
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (email, captcha_token);
        Err("Password reset not available in non-WASM environment".into())
    }
}

/// Reset password with token
async fn reset_password(token: &str, new_password: &str) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_arch = "wasm32")]
    {
        use gloo_net::http::Request;

        let api_url = get_authenc_api_url();
        let reset_url = format!("{}/api/auth/password-reset/confirm", api_url);

        let body = serde_json::json!({
            "token": token,
            "new_password": new_password,
        });

        let response = Request::post(&reset_url)
            .header("Content-Type", "application/json")
            .json(&body)?
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if response.ok() {
            Ok(())
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(format!("HTTP {}: {}", response.status(), error_text).into())
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (token, new_password);
        Err("Password reset not available in non-WASM environment".into())
    }
}
