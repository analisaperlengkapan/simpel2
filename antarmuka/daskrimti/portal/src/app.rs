//! # SIMPelv2 Portal Utama - Modern Government Portal
//!
//! Portal utama yang mengintegrasikan semua layanan SIMPelv2 dengan:
//! - **Modern UI/UX**: Design system berbasis Tailwind CSS
//! - **Performance Optimized**: Code splitting dan lazy loading
//! - **Accessibility**: WCAG 2.1 AA compliance
//! - **Government Branding**: Konsisten dengan identitas Kejaksaan RI

use crate::features::auth::AuthService;
use crate::pages::secrets::SecretsPage;
use crate::pages::*;
use leptos::prelude::*;
use leptos::task::spawn_local;
use shared_microfrontend::components::BrandingProvider;

use leptos_router::{
    StaticSegment,
    components::{Route, Router, Routes},
};

/// Main application component with session management
#[component]
pub fn App() -> impl IntoView {
    // Setup global search providers
    setup_search_providers();

    // Global auth state - load from localStorage on mount
    let (user_session, set_user_session) = signal(AuthService::load_session());

    // Session timeout countdown (in seconds)
    #[allow(unused_variables)]
    let (timeout_countdown, set_timeout_countdown) = signal(0i64);
    #[allow(unused_variables)]
    let (show_timeout_warning, set_show_timeout_warning) = signal(false);

    // Setup cross-tab session sync
    #[cfg(target_arch = "wasm32")]
    {
        use leptos::prelude::Effect;

        Effect::new(move |_| {
            AuthService::setup_storage_listener(move |session| {
                // Check if session is None before moving it
                let is_session_none = session.is_none();
                set_user_session.set(session);

                // If session cleared (logout in another tab), hide timeout warning
                if is_session_none {
                    set_show_timeout_warning.set(false);
                    set_timeout_countdown.set(0);
                }
            });
        });
    }

    // Automatic token refresh and session timeout monitoring
    #[cfg(target_arch = "wasm32")]
    {
        use gloo_timers::future::TimeoutFuture;
        use leptos::prelude::Effect;

        Effect::new(move |_| {
            if let Some(session) = user_session.get() {
                // Spawn async task for token refresh monitoring
                spawn_local(async move {
                    loop {
                        // Check every 30 seconds
                        TimeoutFuture::new(30_000).await;

                        if let Some(current_session) = AuthService::load_session() {
                            // Check if session is expired
                            if !AuthService::is_session_valid(&current_session) {
                                // Session expired - logout
                                AuthService::broadcast_logout();
                                AuthService::logout();
                                set_user_session.set(None);
                                set_show_timeout_warning.set(false);
                                break;
                            }

                            // Check if token needs refresh
                            if AuthService::should_refresh_token(&current_session)
                                && let Some(refresh_token) = &current_session.refresh_token
                            {
                                // Attempt token refresh
                                match AuthService::refresh_token(refresh_token).await {
                                    Ok(token_response) => {
                                        // Update session with new token
                                        AuthService::update_session_token(&token_response);
                                        // Reload session to update UI
                                        set_user_session.set(AuthService::load_session());
                                    }
                                    Err(_) => {
                                        // Refresh failed - logout
                                        AuthService::broadcast_logout();
                                        AuthService::logout();
                                        set_user_session.set(None);
                                        break;
                                    }
                                }
                            }

                            // Calculate time until expiry for countdown
                            if let Some(expires_at) = current_session.expires_at {
                                let now = chrono::Utc::now().timestamp();
                                let time_until_expiry = expires_at - now;

                                // Show warning if less than 2 minutes remaining
                                if time_until_expiry > 0 && time_until_expiry <= 120 {
                                    set_show_timeout_warning.set(true);
                                    set_timeout_countdown.set(time_until_expiry);
                                } else {
                                    set_show_timeout_warning.set(false);
                                }
                            }
                        } else {
                            // No session - stop monitoring
                            break;
                        }
                    }
                });
            }
        });
    }

    // Logout handler with broadcast
    let handle_logout = move || {
        #[cfg(target_arch = "wasm32")]
        AuthService::broadcast_logout();

        AuthService::logout();
        set_user_session.set(None);
        set_show_timeout_warning.set(false);
        set_timeout_countdown.set(0);
    };

    view! {
        <BrandingProvider unit="portal".to_string()>
            // Router untuk halaman
            // Base path disesuaikan dengan serving endpoint
            <Router>
                <Routes fallback=|| view! { <NotFoundPage /> }>
                // Public routes
                <Route path=StaticSegment("") view=HomePage />
                <Route path=StaticSegment("login") view=move || view! {
                    <LoginPage on_login_success=set_user_session />
                } />

                // OAuth callback route
                <Route path=StaticSegment("callback") view=CallbackPage />

                // Logout confirmation page
                <Route path=StaticSegment("logged-out") view=LoggedOutPage />

                // Password reset route
                <Route path=StaticSegment("password-reset") view=PasswordResetPage />

                // MFA routes
                <Route path=StaticSegment("mfa/setup") view=MfaSetupPage />
                <Route path=StaticSegment("mfa/verify") view=MfaVerificationPage />
                <Route path=StaticSegment("mfa/backup-verify") view=MfaBackupVerificationPage />

                // Protected MFA backup codes route
                <Route path=StaticSegment("mfa/backup-codes") view=move || {
                    match user_session.get() {
                        Some(session) => view! {
                            <MfaBackupCodesPage
                                user_session=session
                                on_logout=Box::new(handle_logout)
                            />
                        }.into_any(),
                        None => view! {
                            <LoginPage on_login_success=set_user_session />
                        }.into_any(),
                    }
                } />

                // Protected routes - check auth state
                <Route path=StaticSegment("dashboard") view=move || {
                    match user_session.get() {
                        Some(session) => view! {
                            <DashboardPage
                                user_session=session
                                on_logout=Box::new(handle_logout)
                            />
                        }.into_any(),
                        None => view! {
                            <LoginPage on_login_success=set_user_session />
                        }.into_any(),
                    }
                } />

                <Route path=StaticSegment("apps") view=move || {
                    match user_session.get() {
                        Some(session) => view! {
                            <AppsPage
                                user_session=session
                                on_logout=Box::new(handle_logout)
                            />
                        }.into_any(),
                        None => view! {
                            <LoginPage on_login_success=set_user_session />
                        }.into_any(),
                    }
                } />

                <Route path=StaticSegment("pembinaan") view=move || {
                    match user_session.get() {
                        Some(session) => view! {
                            <PembinaanPage
                                user_session=session
                                on_logout=Box::new(handle_logout)
                            />
                        }.into_any(),
                        None => view! {
                            <LoginPage on_login_success=set_user_session />
                        }.into_any(),
                    }
                } />

                <Route path=StaticSegment("notifications") view=move || {
                    match user_session.get() {
                        Some(session) => view! {
                            <NotificationsPage
                                user_session=session
                                on_logout=Box::new(handle_logout)
                            />
                        }.into_any(),
                        None => view! {
                            <LoginPage on_login_success=set_user_session />
                        }.into_any(),
                    }
                } />

                <Route path=StaticSegment("monitoring") view=move || {
                    match user_session.get() {
                        Some(session) => view! {
                            <MonitoringPage
                                user_session=session
                                on_logout=Box::new(handle_logout)
                            />
                        }.into_any(),
                        None => view! {
                            <LoginPage on_login_success=set_user_session />
                        }.into_any(),
                    }
                } />

                <Route path=StaticSegment("settings") view=move || {
                    match user_session.get() {
                        Some(session) => view! {
                            <SettingsPage
                                user_session=session
                                on_logout=Box::new(handle_logout)
                            />
                        }.into_any(),
                        None => view! {
                            <LoginPage on_login_success=set_user_session />
                        }.into_any(),
                    }
                } />

                <Route path=StaticSegment("secrets") view=move || {
                    match user_session.get() {
                        Some(session) => view! {
                            <SecretsPage
                                user_session=session
                                on_logout=Box::new(handle_logout)
                            />
                        }.into_any(),
                        None => view! {
                            <LoginPage on_login_success=set_user_session />
                        }.into_any(),
                    }
                } />
            </Routes>
        </Router>

        // Session timeout warning modal
        {move || {
            if show_timeout_warning.get() {
                let countdown = timeout_countdown.get();
                let minutes = countdown / 60;
                let seconds = countdown % 60;

                Some(view! {
                    <div class="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-50">
                        <div class="bg-white rounded-lg shadow-xl p-6 max-w-md w-full mx-4">
                            <div class="flex items-center mb-4">
                                <div class="flex-shrink-0">
                                    <svg class="h-12 w-12 text-yellow-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                                    </svg>
                                </div>
                                <div class="ml-4">
                                    <h3 class="text-lg font-semibold text-gray-900">
                                        "Sesi Akan Berakhir"
                                    </h3>
                                </div>
                            </div>

                            <p class="text-gray-600 mb-4">
                                "Sesi Anda akan berakhir dalam "
                                <span class="font-bold text-red-600">
                                    {format!("{:02}:{:02}", minutes, seconds)}
                                </span>
                                ". Silakan simpan pekerjaan Anda."
                            </p>

                            <div class="flex justify-end space-x-3">
                                <button
                                    on:click=move |_| {
                                        set_show_timeout_warning.set(false);
                                    }
                                    class="px-4 py-2 text-sm font-medium text-gray-700 bg-gray-100 rounded-lg hover:bg-gray-200 transition-colors"
                                >
                                    "Tutup"
                                </button>
                                <button
                                    on:click=move |_| {
                                        // Extend session by refreshing token
                                        if let Some(session) = user_session.get()
                                            && let Some(refresh_token) = &session.refresh_token {
                                                let refresh_token = refresh_token.clone();
                                                spawn_local(async move {
                                                    if let Ok(token_response) = AuthService::refresh_token(&refresh_token).await {
                                                        AuthService::update_session_token(&token_response);
                                                        set_user_session.set(AuthService::load_session());
                                                        set_show_timeout_warning.set(false);
                                                    }
                                                });
                                            }
                                    }
                                    class="px-4 py-2 text-sm font-medium text-white bg-primary rounded-lg hover:bg-primary-dark transition-colors"
                                >
                                    "Perpanjang Sesi"
                                </button>
                            </div>
                        </div>
                    </div>
                })
            } else {
                None
            }
        }}
        </BrandingProvider>
    }
}

/// Setup global search providers
fn setup_search_providers() {
    use crate::features::microfrontends::MicrofrontendRegistry;
    use shared_microfrontend::hooks::{SearchCategory, SearchResult, use_search};

    let search_ctx = use_search();

    // Register applications
    let apps: Vec<SearchResult> = MicrofrontendRegistry::get_all_apps()
        .into_iter()
        .map(|app| SearchResult {
            id: app.id.clone(),
            title: app.name,
            description: app.description,
            category: SearchCategory::Application,
            url: app.url,
            icon: app.icon,
            module: None,
        })
        .collect();

    search_ctx.register_data(apps);

    // Register pages
    let pages = vec![
        SearchResult {
            id: "dashboard".to_string(),
            title: "Dashboard".to_string(),
            description: "Dashboard utama dengan statistik dan aktivitas terbaru".to_string(),
            category: SearchCategory::Page,
            url: "/dashboard".to_string(),
            icon: "📊".to_string(),
            module: Some("Portal".to_string()),
        },
        SearchResult {
            id: "apps".to_string(),
            title: "Aplikasi".to_string(),
            description: "Daftar semua aplikasi SIMPelv2 yang tersedia".to_string(),
            category: SearchCategory::Page,
            url: "/apps".to_string(),
            icon: "🚀".to_string(),
            module: Some("Portal".to_string()),
        },
        SearchResult {
            id: "notifications".to_string(),
            title: "Notifikasi".to_string(),
            description: "Semua notifikasi dan pemberitahuan sistem".to_string(),
            category: SearchCategory::Page,
            url: "/notifications".to_string(),
            icon: "🔔".to_string(),
            module: Some("Portal".to_string()),
        },
        SearchResult {
            id: "monitoring".to_string(),
            title: "Monitoring".to_string(),
            description:
                "Dashboard monitoring dengan metrik performa, error tracking, dan analytics"
                    .to_string(),
            category: SearchCategory::Page,
            url: "/monitoring".to_string(),
            icon: "📈".to_string(),
            module: Some("Portal".to_string()),
        },
        SearchResult {
            id: "pembinaan".to_string(),
            title: "Pembinaan".to_string(),
            description: "Sistem pembinaan dan pengembangan SDM".to_string(),
            category: SearchCategory::Page,
            url: "/pembinaan".to_string(),
            icon: "🌱".to_string(),
            module: Some("Portal".to_string()),
        },
        SearchResult {
            id: "settings".to_string(),
            title: "Pengaturan".to_string(),
            description: "Kelola preferensi, tema, dan kustomisasi tampilan".to_string(),
            category: SearchCategory::Page,
            url: "/settings".to_string(),
            icon: "⚙️".to_string(),
            module: Some("Portal".to_string()),
        },
    ];

    search_ctx.register_data(pages);
}
