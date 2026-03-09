//! # SIMPEL Portal Utama - Modern Government Portal
//!
//! Portal utama yang mengintegrasikan semua layanan SIMPEL dengan:
//! - **Modern UI/UX**: Design system berbasis Tailwind CSS
//! - **Performance Optimized**: Code splitting dan lazy loading
//! - **Accessibility**: WCAG 2.1 AA compliance
//! - **Government Branding**: Konsisten dengan identitas Kejaksaan RI

use crate::features::auth::{AuthService, UserSession};

use crate::pages::*;
use leptos::prelude::*;
use leptos::task::spawn_local;
use lib_ui::components::BrandingProvider;

use leptos_router::{
    ParamSegment, StaticSegment,
    components::{Route, Router, Routes},
};

// ============================================================================
// AUTH GUARD HELPER COMPONENT
// ============================================================================

/// Auth guard component that redirects to login if not authenticated.
/// Used by self-service and admin routes to avoid match boilerplate.
#[component]
fn WithAuth(
    /// Current user session signal
    user_session: ReadSignal<Option<UserSession>>,
    /// Login success writer for redirect-to-login fallback
    on_login_success: WriteSignal<Option<UserSession>>,
    /// Child content rendered when authenticated
    children: ChildrenFn,
) -> impl IntoView {
    let children = StoredValue::new_local(children);
    move || match user_session.get() {
        Some(_session) => children.with_value(|c| c().into_any()),
        None => view! {
            <LoginPage on_login_success=on_login_success />
        }
        .into_any(),
    }
}

/// Auth guard component that only allows admin users.
#[component]
fn WithAdminAuth(
    /// Current user session signal
    user_session: ReadSignal<Option<UserSession>>,
    /// Login success writer for redirect-to-login fallback
    on_login_success: WriteSignal<Option<UserSession>>,
    /// Child content rendered when authenticated and admin
    children: ChildrenFn,
) -> impl IntoView {
    let children = StoredValue::new_local(children);
    move || match user_session.get() {
        Some(session) => {
            if session.role.is_admin() {
                children.with_value(|c| c().into_any())
            } else {
                view! { <crate::components::guards::ForbiddenPage /> }.into_any()
            }
        }
        None => view! {
            <LoginPage on_login_success=on_login_success />
        }
        .into_any(),
    }
}

/// Main application component with session management
#[component]
pub fn App() -> impl IntoView {
    // Load runtime configuration (authenc URL etc.) from config.json
    #[cfg(target_arch = "wasm32")]
    {
        use leptos::task::spawn_local;
        spawn_local(async {
            crate::utils::config::load_config().await;
        });
    }

    // Setup global search providers
    setup_search_providers();

    // Initialize global application state (including AuthencApiClient)
    let _app_state = crate::utils::app_state::provide_app_state();

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
        use leptos::prelude::on_cleanup;
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;

        Effect::new(move |_| {
            if let Some(_session) = user_session.get() {
                // Cancellation token to prevent multiple concurrent tasks
                let is_active = Arc::new(AtomicBool::new(true));
                let is_active_clone = is_active.clone();

                on_cleanup(move || {
                    is_active_clone.store(false, Ordering::SeqCst);
                });

                // Spawn async task for token refresh monitoring
                spawn_local(async move {
                    'monitor: loop {
                        if !is_active.load(Ordering::SeqCst) {
                            break;
                        }

                        if let Some(mut current_session) = AuthService::load_session() {
                            // Check if session is expired
                            if !AuthService::is_session_valid(&current_session) {
                                // Session expired - logout
                                AuthService::broadcast_logout();
                                AuthService::logout();
                                set_user_session.set(None);
                                set_show_timeout_warning.set(false);
                                set_timeout_countdown.set(0);
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
                                        // Update current_session for accurate countdown calculation
                                        if let Some(refreshed_session) = AuthService::load_session()
                                        {
                                            current_session = refreshed_session;
                                        }
                                    }
                                    Err(_) => {
                                        // Refresh failed - logout
                                        AuthService::broadcast_logout();
                                        AuthService::logout();
                                        set_user_session.set(None);
                                        set_show_timeout_warning.set(false);
                                        set_timeout_countdown.set(0);
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
                                    set_timeout_countdown.set(0);
                                }
                            }
                        } else {
                            // No session - stop monitoring
                            break;
                        }

                        if !is_active.load(Ordering::SeqCst) {
                            break;
                        }

                        // Wait 30 seconds, decrementing countdown every second if active
                        for _ in 0..30 {
                            if !is_active.load(Ordering::SeqCst) {
                                break;
                            }
                            TimeoutFuture::new(1_000).await;

                            let current = timeout_countdown.get_untracked();
                            if current > 0 {
                                set_timeout_countdown.set(current - 1);
                                if current - 1 <= 0 {
                                    // Session expired due to countdown
                                    AuthService::broadcast_logout();
                                    AuthService::logout();
                                    set_user_session.set(None);
                                    set_show_timeout_warning.set(false);
                                    break 'monitor;
                                }
                            }
                        }
                    }
                });
            }
        });
    }

    // Logout handler with broadcast — produces Box<dyn Fn()> for each route
    let make_logout = move || -> Box<dyn Fn()> {
        Box::new(move || {
            #[cfg(target_arch = "wasm32")]
            AuthService::broadcast_logout();

            AuthService::logout();
            set_user_session.set(None);
            set_show_timeout_warning.set(false);
            set_timeout_countdown.set(0);
        })
    };

    view! {
        <BrandingProvider unit="portal".to_string()>
            <Router base="/portal">
                <Routes fallback=|| view! { <NotFoundPage /> }>

                // ══════════════════════════════════════════════
                // PUBLIC ROUTES (no auth required)
                // ══════════════════════════════════════════════
                <Route path=StaticSegment("") view=HomePage />
                <Route path=StaticSegment("login") view=move || view! {
                    <LoginPage on_login_success=set_user_session />
                } />
                <Route path=StaticSegment("callback") view=CallbackPage />
                <Route path=StaticSegment("logged-out") view=LoggedOutPage />

                // MFA routes (semi-public, temp-token based)
                <Route path=StaticSegment("mfa/setup") view=MfaSetupPage />
                <Route path=StaticSegment("mfa/verify") view=MfaVerificationPage />
                <Route path=StaticSegment("mfa/backup-verify") view=MfaBackupVerificationPage />

                // ══════════════════════════════════════════════
                // PROTECTED ROUTES (with MainLayout wrapper)
                // ══════════════════════════════════════════════
                <Route path=StaticSegment("mfa/backup-codes") view=move || {
                    match user_session.get() {
                        Some(session) => view! {
                            <MfaBackupCodesPage user_session=session on_logout=make_logout() />
                        }.into_any(),
                        None => view! { <LoginPage on_login_success=set_user_session /> }.into_any(),
                    }
                } />

                <Route path=StaticSegment("dashboard") view=move || {
                    match user_session.get() {
                        Some(session) => view! {
                            <DashboardPage user_session=session on_logout=make_logout() />
                        }.into_any(),
                        None => view! { <LoginPage on_login_success=set_user_session /> }.into_any(),
                    }
                } />

                <Route path=StaticSegment("apps") view=move || {
                    match user_session.get() {
                        Some(session) => view! {
                            <AppsPage user_session=session on_logout=make_logout() />
                        }.into_any(),
                        None => view! { <LoginPage on_login_success=set_user_session /> }.into_any(),
                    }
                } />

                <Route path=StaticSegment("pembinaan") view=move || {
                    match user_session.get() {
                        Some(session) => view! {
                            <PembinaanPage user_session=session on_logout=make_logout() />
                        }.into_any(),
                        None => view! { <LoginPage on_login_success=set_user_session /> }.into_any(),
                    }
                } />

                <Route path=StaticSegment("notifications") view=move || {
                    match user_session.get() {
                        Some(session) => view! {
                            <NotificationsPage user_session=session on_logout=make_logout() />
                        }.into_any(),
                        None => view! { <LoginPage on_login_success=set_user_session /> }.into_any(),
                    }
                } />

                <Route path=StaticSegment("monitoring") view=move || {
                    match user_session.get() {
                        Some(session) => view! {
                            <MonitoringPage user_session=session on_logout=make_logout() />
                        }.into_any(),
                        None => view! { <LoginPage on_login_success=set_user_session /> }.into_any(),
                    }
                } />

                <Route path=StaticSegment("settings") view=move || {
                    match user_session.get() {
                        Some(session) => view! {
                            <SettingsPage user_session=session on_logout=make_logout() />
                        }.into_any(),
                        None => view! { <LoginPage on_login_success=set_user_session /> }.into_any(),
                    }
                } />

                // ══════════════════════════════════════════════
                // SELF-SERVICE ACCOUNT MANAGEMENT (WithAuth guard)
                // ══════════════════════════════════════════════
                <Route path=StaticSegment("profile") view=move || view! {
                    <WithAuth user_session=user_session on_login_success=set_user_session>
                        <crate::pages::profile::ProfilePage />
                    </WithAuth>
                } />
                <Route path=StaticSegment("passkeys") view=move || view! {
                    <WithAuth user_session=user_session on_login_success=set_user_session>
                        <crate::pages::passkeys::PasskeysPage />
                    </WithAuth>
                } />
                <Route path=StaticSegment("password") view=move || view! {
                    <WithAuth user_session=user_session on_login_success=set_user_session>
                        <crate::pages::password_change::PasswordChangePage />
                    </WithAuth>
                } />
                <Route path=StaticSegment("sessions") view=move || view! {
                    <WithAuth user_session=user_session on_login_success=set_user_session>
                        <crate::pages::sessions::SessionsPage />
                    </WithAuth>
                } />

                // ══════════════════════════════════════════════
                // ADMIN IAM ROUTES (WithAdminAuth guard)
                // ══════════════════════════════════════════════
                <Route path=StaticSegment("admin") view=move || view! {
                    <WithAdminAuth user_session=user_session on_login_success=set_user_session>
                        <crate::pages::admin::AdminOverviewPage />
                    </WithAdminAuth>
                } />
                <Route path=StaticSegment("admin/users") view=move || view! {
                    <WithAdminAuth user_session=user_session on_login_success=set_user_session>
                        <crate::pages::admin::UsersManagementPage />
                    </WithAdminAuth>
                } />
                <Route path=(StaticSegment("admin/users"), ParamSegment("id")) view=move || view! {
                    <WithAdminAuth user_session=user_session on_login_success=set_user_session>
                        <crate::pages::admin::UserDetailPage />
                    </WithAdminAuth>
                } />
                <Route path=StaticSegment("admin/realms") view=move || view! {
                    <WithAdminAuth user_session=user_session on_login_success=set_user_session>
                        <crate::pages::admin::RealmsManagementPage />
                    </WithAdminAuth>
                } />
                <Route path=StaticSegment("admin/clients") view=move || view! {
                    <WithAdminAuth user_session=user_session on_login_success=set_user_session>
                        <crate::pages::admin::ClientsManagementPage />
                    </WithAdminAuth>
                } />
                <Route path=(StaticSegment("admin/clients"), ParamSegment("id")) view=move || view! {
                    <WithAdminAuth user_session=user_session on_login_success=set_user_session>
                        <crate::pages::admin::ClientDetailPage />
                    </WithAdminAuth>
                } />
                <Route path=StaticSegment("admin/roles") view=move || view! {
                    <WithAdminAuth user_session=user_session on_login_success=set_user_session>
                        <crate::pages::admin::RolesManagementPage />
                    </WithAdminAuth>
                } />
                <Route path=StaticSegment("admin/federation") view=move || view! {
                    <WithAdminAuth user_session=user_session on_login_success=set_user_session>
                        <crate::pages::admin::FederationManagementPage />
                    </WithAdminAuth>
                } />
                <Route path=StaticSegment("admin/permissions") view=move || view! {
                    <WithAdminAuth user_session=user_session on_login_success=set_user_session>
                        <crate::pages::admin::PermissionsManagementPage />
                    </WithAdminAuth>
                } />
                <Route path=StaticSegment("admin/audit") view=move || view! {
                    <WithAdminAuth user_session=user_session on_login_success=set_user_session>
                        <crate::pages::admin::AuditLogsPage />
                    </WithAdminAuth>
                } />
                <Route path=StaticSegment("admin/groups") view=move || view! {
                    <WithAdminAuth user_session=user_session on_login_success=set_user_session>
                        <crate::pages::admin::GroupsManagementPage />
                    </WithAdminAuth>
                } />
                <Route path=StaticSegment("admin/realm-settings") view=move || view! {
                    <WithAdminAuth user_session=user_session on_login_success=set_user_session>
                        <crate::pages::admin::RealmSettingsPage />
                    </WithAdminAuth>
                } />
                <Route path=StaticSegment("admin/auth-flows") view=move || view! {
                    <WithAdminAuth user_session=user_session on_login_success=set_user_session>
                        <crate::pages::admin::AuthFlowsPage />
                    </WithAdminAuth>
                } />
                <Route path=StaticSegment("admin/linked-accounts") view=move || view! {
                    <WithAdminAuth user_session=user_session on_login_success=set_user_session>
                        <crate::pages::admin::LinkedAccountsPage />
                    </WithAdminAuth>
                } />
            </Routes>
        </Router>

        // Session timeout warning modal (extracted to component)
        <SessionTimeoutModal
            show=show_timeout_warning
            countdown=timeout_countdown
            user_session=user_session
            set_user_session=set_user_session
            set_show=set_show_timeout_warning
            set_countdown=set_timeout_countdown
        />
        </BrandingProvider>
    }
}

// ============================================================================
// SESSION TIMEOUT MODAL (extracted from inline closure for readability)
// ============================================================================

/// Session timeout warning modal component
#[component]
fn SessionTimeoutModal(
    show: ReadSignal<bool>,
    countdown: ReadSignal<i64>,
    user_session: ReadSignal<Option<UserSession>>,
    set_user_session: WriteSignal<Option<UserSession>>,
    set_show: WriteSignal<bool>,
    set_countdown: WriteSignal<i64>,
) -> impl IntoView {
    move || {
        if show.get() {
            let secs = countdown.get();
            let minutes = secs / 60;
            let seconds = secs % 60;

            Some(view! {
                <div
                    class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm"
                    role="dialog"
                    aria-modal="true"
                    aria-label="Peringatan sesi akan berakhir"
                >
                    <div class="bg-white dark:bg-gray-800 rounded-2xl shadow-2xl p-6 max-w-md w-full mx-4 border border-gray-200 dark:border-gray-700">
                        <div class="flex items-center mb-4">
                            <div class="flex-shrink-0 w-12 h-12 bg-amber-100 dark:bg-amber-900/30 rounded-full flex items-center justify-center">
                                <svg class="h-6 w-6 text-amber-600 dark:text-amber-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                                </svg>
                            </div>
                            <div class="ml-4">
                                <h3 class="text-lg font-semibold text-gray-900 dark:text-white">
                                    "Sesi Akan Berakhir"
                                </h3>
                            </div>
                        </div>

                        <p class="text-gray-600 dark:text-gray-400 mb-6">
                            "Sesi Anda akan berakhir dalam "
                            <span class="font-bold text-red-600 dark:text-red-400 tabular-nums">
                                {format!("{:02}:{:02}", minutes, seconds)}
                            </span>
                            ". Silakan simpan pekerjaan Anda."
                        </p>

                        <div class="flex justify-end space-x-3">
                            <button
                                on:click=move |_| set_show.set(false)
                                class="px-4 py-2.5 text-sm font-medium text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-700 rounded-xl hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors"
                            >
                                "Tutup"
                            </button>
                            <button
                                on:click=move |_| {
                                    if let Some(session) = user_session.get()
                                        && let Some(refresh_token) = &session.refresh_token {
                                            let refresh_token = refresh_token.clone();
                                            spawn_local(async move {
                                                if let Ok(token_response) = AuthService::refresh_token(&refresh_token).await {
                                                    AuthService::update_session_token(&token_response);
                                                    set_user_session.set(AuthService::load_session());
                                                    set_show.set(false);
                                                    set_countdown.set(0);
                                                }
                                            });
                                        }
                                }
                                class="px-4 py-2.5 text-sm font-medium text-white bg-navy-700 hover:bg-navy-800 dark:bg-gold-500 dark:hover:bg-gold-600 dark:text-navy-900 rounded-xl shadow-sm transition-colors"
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
    }
}

/// Setup global search providers
fn setup_search_providers() {
    use crate::features::microfrontends::MicrofrontendRegistry;
    use lib_ui::hooks::{SearchCategory, SearchResult, use_search};

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
            url: "/portal/dashboard".to_string(),
            icon: "📊".to_string(),
            module: Some("Portal".to_string()),
        },
        SearchResult {
            id: "apps".to_string(),
            title: "Aplikasi".to_string(),
            description: "Daftar semua aplikasi SIMPEL yang tersedia".to_string(),
            category: SearchCategory::Page,
            url: "/portal/apps".to_string(),
            icon: "🚀".to_string(),
            module: Some("Portal".to_string()),
        },
        SearchResult {
            id: "notifications".to_string(),
            title: "Notifikasi".to_string(),
            description: "Semua notifikasi dan pemberitahuan sistem".to_string(),
            category: SearchCategory::Page,
            url: "/portal/notifications".to_string(),
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
            url: "/portal/monitoring".to_string(),
            icon: "📈".to_string(),
            module: Some("Portal".to_string()),
        },
        SearchResult {
            id: "pembinaan".to_string(),
            title: "Pembinaan".to_string(),
            description: "Sistem pembinaan dan pengembangan SDM".to_string(),
            category: SearchCategory::Page,
            url: "/portal/pembinaan".to_string(),
            icon: "🌱".to_string(),
            module: Some("Portal".to_string()),
        },
        SearchResult {
            id: "settings".to_string(),
            title: "Pengaturan".to_string(),
            description: "Kelola preferensi, tema, dan kustomisasi tampilan".to_string(),
            category: SearchCategory::Page,
            url: "/portal/settings".to_string(),
            icon: "⚙️".to_string(),
            module: Some("Portal".to_string()),
        },
        SearchResult {
            id: "profile".to_string(),
            title: "Profil Saya".to_string(),
            description: "Kelola informasi profil dan data pribadi".to_string(),
            category: SearchCategory::Page,
            url: "/portal/profile".to_string(),
            icon: "👤".to_string(),
            module: Some("Portal".to_string()),
        },
        SearchResult {
            id: "passkeys".to_string(),
            title: "Passkey".to_string(),
            description: "Kelola kunci keamanan dan autentikasi biometrik".to_string(),
            category: SearchCategory::Page,
            url: "/portal/passkeys".to_string(),
            icon: "🔐".to_string(),
            module: Some("Portal".to_string()),
        },
        SearchResult {
            id: "password".to_string(),
            title: "Ubah Kata Sandi".to_string(),
            description: "Perbarui kata sandi akun Anda".to_string(),
            category: SearchCategory::Page,
            url: "/portal/password".to_string(),
            icon: "🔒".to_string(),
            module: Some("Portal".to_string()),
        },
        SearchResult {
            id: "sessions".to_string(),
            title: "Sesi Aktif".to_string(),
            description: "Kelola sesi login dan perangkat aktif".to_string(),
            category: SearchCategory::Page,
            url: "/portal/sessions".to_string(),
            icon: "📱".to_string(),
            module: Some("Portal".to_string()),
        },
        SearchResult {
            id: "admin".to_string(),
            title: "Admin Panel".to_string(),
            description: "Administrasi Identity & Access Management".to_string(),
            category: SearchCategory::Page,
            url: "/portal/admin".to_string(),
            icon: "🛡️".to_string(),
            module: Some("Admin".to_string()),
        },
    ];

    search_ctx.register_data(pages);
}
