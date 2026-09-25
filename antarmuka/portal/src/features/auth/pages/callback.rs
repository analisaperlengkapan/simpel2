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
                        // Keep the developer-facing detail in the console; the
                        // card gets the operator-facing translation.
                        web_sys::console::warn_1(&format!("[oauth-callback] {e}").into());
                        set_status.set(CallbackStatus::Error);
                        set_error_message.set(Some(user_facing_callback_error(&e)));
                    }
                }
            });
        });
    }

    view! {
        <crate::components::layout::AuthLayout>
            <div class="max-w-md w-full mx-auto">
                {move || match status.get() {
                    CallbackStatus::Processing => {
                        view! {
                            <div class="bg-white rounded-lg shadow-lg p-8 text-center">
                                <div class="mb-6">
                                    <svg
                                        class="animate-spin h-16 w-16 mx-auto text-primary"
                                        xmlns="http://www.w3.org/2000/svg"
                                        fill="none"
                                        viewBox="0 0 24 24"
                                    >
                                        <circle
                                            class="opacity-25"
                                            cx="12"
                                            cy="12"
                                            r="10"
                                            stroke="currentColor"
                                            stroke-width="4"
                                        ></circle>
                                        <path
                                            class="opacity-75"
                                            fill="currentColor"
                                            d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                                        ></path>
                                    </svg>
                                </div>
                                <h2 class="text-2xl font-bold text-gray-900 mb-2">
                                    "Memproses Autentikasi"
                                </h2>
                                <p class="text-gray-600">"Mohon tunggu sebentar..."</p>
                            </div>
                        }
                            .into_any()
                    }
                    CallbackStatus::Success => {

                        view! {
                            <div class="bg-white rounded-lg shadow-lg p-8 text-center">
                                <div class="mb-6">
                                    <svg
                                        class="h-16 w-16 mx-auto text-green-500"
                                        fill="none"
                                        viewBox="0 0 24 24"
                                        stroke="currentColor"
                                    >
                                        <path
                                            stroke-linecap="round"
                                            stroke-linejoin="round"
                                            stroke-width="2"
                                            d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
                                        />
                                    </svg>
                                </div>
                                <h2 class="text-2xl font-bold text-gray-900 mb-2">
                                    "Autentikasi Berhasil"
                                </h2>
                                <p class="text-gray-600">"Mengalihkan ke aplikasi..."</p>
                            </div>
                        }
                            .into_any()
                    }
                    CallbackStatus::Error => {

                        view! {
                            <div class="bg-white rounded-lg shadow-lg p-8">
                                <div class="mb-6 text-center">
                                    <svg
                                        class="h-16 w-16 mx-auto text-red-500"
                                        fill="none"
                                        viewBox="0 0 24 24"
                                        stroke="currentColor"
                                    >
                                        <path
                                            stroke-linecap="round"
                                            stroke-linejoin="round"
                                            stroke-width="2"
                                            d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                                        />
                                    </svg>
                                </div>
                                <h2 class="text-2xl font-bold text-gray-900 mb-4 text-center">
                                    "Autentikasi Gagal"
                                </h2>
                                {move || {
                                    error_message
                                        .get()
                                        .map(|msg| {
                                            view! {
                                                <div class="bg-red-50 border border-red-200 rounded-lg p-4 mb-6">
                                                    <p class="text-sm text-red-800">{msg}</p>
                                                </div>
                                            }
                                        })
                                }}
                                <div class="flex justify-center">
                                    <a
                                        href="/portal/login"
                                        class="inline-flex items-center px-6 py-3 bg-primary text-white font-medium rounded-lg hover:bg-primary-dark transition-colors"
                                    >
                                        <svg
                                            class="w-5 h-5 mr-2"
                                            fill="none"
                                            viewBox="0 0 24 24"
                                            stroke="currentColor"
                                        >
                                            <path
                                                stroke-linecap="round"
                                                stroke-linejoin="round"
                                                stroke-width="2"
                                                d="M11 17l-5-5m0 0l5-5m-5 5h12"
                                            />
                                        </svg>
                                        "Kembali ke Login"
                                    </a>
                                </div>
                            </div>
                        }
                            .into_any()
                    }
                }}
            </div>
        </crate::components::layout::AuthLayout>
    }
}

/// Translate an internal OAuth failure into something a Jaksa can act on.
///
/// The strings produced by the flow are written for whoever debugs it — "Missing
/// authorization code", "Invalid state parameter. Possible CSRF attack
/// detected." — and they are rendered verbatim into the error card. Two problems
/// with that. The reader is an operator, not the developer, and has no lever
/// over an authorization code; and the CSRF string announces a security incident
/// for what is far more often a stale bookmark or a session that expired
/// mid-redirect. Naming the likely cause and the next step is the useful thing.
///
/// The raw string is deliberately not appended: it is kept in the browser
/// console for support to retrieve, while the page stays readable.
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
fn user_facing_callback_error(raw: &str) -> String {
    let lower = raw.to_lowercase();
    if lower.contains("no window") || lower.contains("failed to get url") {
        "Browser tidak dapat membaca alamat halaman ini. Muat ulang halaman lalu coba lagi."
            .to_string()
    } else if lower.contains("missing authorization code") || lower.contains("missing state") {
        "Tautan masuk ini tidak lengkap atau sudah kedaluwarsa. Ini biasanya terjadi bila \
         halaman dimuat ulang atau tautan dibuka kembali dari riwayat peramban. Silakan mulai \
         proses masuk kembali dari halaman login."
            .to_string()
    } else if lower.contains("state parameter") {
        "Sesi masuk Anda sudah berakhir sebelum proses selesai. Silakan mulai proses masuk \
         kembali dari halaman login."
            .to_string()
    } else if lower.contains("invalid url") || lower.contains("failed to get origin") {
        "Alamat halaman masuk tidak dikenali. Buka kembali halaman login SIMPEL lalu coba lagi."
            .to_string()
    } else if lower.contains("oauth error") {
        format!(
            "Penyedia identitas menolak permintaan masuk ini ({raw}). Hubungi administrator sistem bila berlanjut."
        )
    } else if lower.contains("failed to decode token") {
        "Server mengirim data sesi yang tidak dapat dibaca. Hubungi administrator sistem."
            .to_string()
    } else {
        "Proses masuk tidak dapat diselesaikan. Silakan coba lagi; bila masih gagal, hubungi \
         administrator sistem."
            .to_string()
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

#[cfg(test)]
mod callback_error_tests {
    use super::user_facing_callback_error;

    /// The equivalence that matters is not "the wording is nice" but "no internal
    /// string escapes to the page". Asserting the raw term is ABSENT is the half
    /// that catches a future edit which appends the cause for convenience.
    #[test]
    fn internal_jargon_never_reaches_the_reader() {
        for raw in [
            "Missing authorization code",
            "Missing state parameter",
            "Invalid state parameter. Possible CSRF attack detected.",
            "Failed to decode token: invalid signature",
            "Invalid URL",
        ] {
            let out = user_facing_callback_error(raw);
            assert!(!out.is_empty());
            for jargon in ["authorization code", "CSRF", "decode", "parameter"] {
                assert!(
                    !out.to_lowercase().contains(jargon.to_lowercase().as_str()),
                    "jargon {jargon:?} leaked from {raw:?} into {out:?}"
                );
            }
        }
    }

    /// The mapping must be discriminating, not a single catch-all: a stale link
    /// and an expired session are different situations and deserve different
    /// instructions.
    #[test]
    fn distinct_causes_map_to_distinct_guidance() {
        let stale = user_facing_callback_error("Missing authorization code");
        let expired =
            user_facing_callback_error("Invalid state parameter. Possible CSRF attack detected.");
        let provider = user_facing_callback_error("OAuth error: access_denied - user cancelled");

        assert_ne!(stale, expired);
        assert!(stale.contains("kedaluwarsa"));
        assert!(expired.contains("berakhir"));
        // The provider's own code is the one thing worth echoing back.
        assert!(provider.contains("access_denied"));
    }

    /// An unseen failure must still produce actionable text rather than an empty
    /// card or a panic.
    #[test]
    fn an_unrecognised_failure_still_tells_the_user_what_to_do() {
        let out = user_facing_callback_error("koneksi terputus di tengah jalan");
        assert!(!out.is_empty());
        assert!(out.contains("coba lagi") || out.contains("administrator"));
    }
}
