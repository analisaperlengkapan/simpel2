//! OAuth2 callback handler page
//!
//! Handles OAuth2 authorization code callback from Authenc

#[allow(unused_imports)]
use crate::features::auth::oauth::OAuthClient;
#[allow(unused_imports)]
use crate::features::auth::{AuthService, UserSession};
use leptos::prelude::*;

/// OAuth callback page component
/// This page handles the OAuth2 authorization code flow callback:
/// 1. Parse authorization code and state from URL
/// 2. Verify state parameter (CSRF protection)
/// 3. Exchange code for access token
/// 4. Store session
/// 5. Redirect to origin URL or dashboard
#[component]
#[allow(unused_variables)]
pub fn CallbackPage() -> impl IntoView {
    let (status, set_status) = signal(CallbackStatus::Processing);
    let (error_message, set_error_message) = signal(None::<String>);

    // Process OAuth callback on mount
    #[cfg(target_arch = "wasm32")]
    {
        use leptos::prelude::Effect;
        use leptos::task::spawn_local;

        Effect::new(move |_| {
            spawn_local(async move {
                match process_oauth_callback().await {
                    Ok(session) => {
                        // Save session
                        AuthService::save_session(&session);

                        // Get return URL or default to dashboard
                        let return_url = OAuthClient::get_return_url()
                            .unwrap_or_else(|| "/dashboard".to_string());

                        set_status.set(CallbackStatus::Success);

                        // Redirect after short delay to show success message
                        gloo_timers::future::TimeoutFuture::new(1000).await;

                        if let Some(window) = web_sys::window() {
                            let _ = window.location().set_href(&return_url);
                        }
                    }
                    Err(e) => {
                        set_status.set(CallbackStatus::Error);
                        set_error_message.set(Some(e));
                    }
                }
            });
        });
    }

    view! {
        <div class="min-h-screen flex items-center justify-center bg-gray-50 px-4">
            <div class="max-w-md w-full">
                {move || match status.get() {
                    CallbackStatus::Processing => view! {
                        <div class="bg-white rounded-lg shadow-lg p-8 text-center">
                            <div class="mb-6">
                                <svg class="animate-spin h-16 w-16 mx-auto text-primary" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                                </svg>
                            </div>
                            <h2 class="text-2xl font-bold text-gray-900 mb-2">
                                "Memproses Autentikasi"
                            </h2>
                            <p class="text-gray-600">
                                "Mohon tunggu sebentar..."
                            </p>
                        </div>
                    }.into_any(),

                    CallbackStatus::Success => view! {
                        <div class="bg-white rounded-lg shadow-lg p-8 text-center">
                            <div class="mb-6">
                                <svg class="h-16 w-16 mx-auto text-green-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                                </svg>
                            </div>
                            <h2 class="text-2xl font-bold text-gray-900 mb-2">
                                "Autentikasi Berhasil"
                            </h2>
                            <p class="text-gray-600">
                                "Mengalihkan ke aplikasi..."
                            </p>
                        </div>
                    }.into_any(),

                    CallbackStatus::Error => view! {
                        <div class="bg-white rounded-lg shadow-lg p-8">
                            <div class="mb-6 text-center">
                                <svg class="h-16 w-16 mx-auto text-red-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                                </svg>
                            </div>
                            <h2 class="text-2xl font-bold text-gray-900 mb-4 text-center">
                                "Autentikasi Gagal"
                            </h2>
                            {move || error_message.get().map(|msg| view! {
                                <div class="bg-red-50 border border-red-200 rounded-lg p-4 mb-6">
                                    <p class="text-sm text-red-800">
                                        {msg}
                                    </p>
                                </div>
                            })}
                            <div class="flex justify-center">
                                <a
                                    href="/portal/login"
                                    class="inline-flex items-center px-6 py-3 bg-primary text-white font-medium rounded-lg hover:bg-primary-dark transition-colors"
                                >
                                    <svg class="w-5 h-5 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 17l-5-5m0 0l5-5m-5 5h12" />
                                    </svg>
                                    "Kembali ke Login"
                                </a>
                            </div>
                        </div>
                    }.into_any(),
                }}
            </div>
        </div>
    }
}

/// Callback processing status
#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(dead_code)]
enum CallbackStatus {
    /// Processing OAuth callback
    Processing,
    /// Successfully authenticated
    Success,
    /// Authentication failed
    Error,
}

/// Process OAuth callback
/// Extracts authorization code from URL, exchanges it for token,
/// and returns user session
#[cfg(target_arch = "wasm32")]
#[allow(dead_code)]
async fn process_oauth_callback() -> Result<UserSession, String> {
    // Get current URL
    let window = web_sys::window().ok_or("No window object")?;
    let location = window.location();
    let url = location.href().map_err(|_| "Failed to get URL")?;

    // Parse callback URL to extract code and state
    let (code, state) = OAuthClient::parse_callback_url(&url)?;

    // Verify state parameter (CSRF protection)
    if !OAuthClient::verify_state(&state) {
        return Err("Invalid state parameter. Possible CSRF attack detected.".to_string());
    }

    // Create OAuth client
    let base_url =
        std::env::var("AUTHENC_API_URL").unwrap_or_else(|_| "http://localhost:8088".to_string());
    let realm = std::env::var("AUTHENC_REALM").unwrap_or_else(|_| "simpel".to_string());
    let client_id = std::env::var("OAUTH_CLIENT_ID").unwrap_or_else(|_| "portal".to_string());

    // Get redirect URI from current origin
    let origin = location.origin().map_err(|_| "Failed to get origin")?;
    let redirect_uri = format!("{}/callback", origin);

    let oauth_client = OAuthClient::new(base_url, realm, client_id, redirect_uri);

    // Exchange authorization code for access token
    let token_response = oauth_client.exchange_code(&code).await?;

    // Decode JWT to extract user info
    let mut session = AuthService::decode_jwt_claims(&token_response.access_token)
        .map_err(|e| format!("Failed to decode token: {}", e))?;

    // Update session with token info
    session.access_token = Some(token_response.access_token.clone());
    session.refresh_token = token_response.refresh_token.clone();

    // Calculate expiration time
    let now = chrono::Utc::now();
    let expires_at = now + chrono::Duration::seconds(token_response.expires_in as i64);
    session.expires_at = Some(expires_at.timestamp());

    Ok(session)
}

/// Non-WASM fallback (for SSR/testing)
#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
async fn process_oauth_callback() -> Result<UserSession, String> {
    Err("OAuth callback not available in non-WASM environment".to_string())
}
