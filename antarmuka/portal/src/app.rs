//! # SIMPEL Portal Utama - Modern Government Portal
//!
//! Portal utama yang mengintegrasikan semua layanan SIMPEL dengan:
//! - **Modern UI/UX**: Design system berbasis Tailwind CSS
//! - **Performance Optimized**: Code splitting dan lazy loading
//! - **Accessibility**: WCAG 2.1 AA compliance
//! - **Government Branding**: Konsisten dengan identitas Kejaksaan RI

use crate::components::guards::{PortalAdminLayout, PortalAuthLayout};
use crate::components::session_timeout_modal::SessionTimeoutModal;
use crate::features::auth::AuthService;
use crate::features::auth::{CallbackPage, LoggedOutPage, LoginPage, PasswordChangePage};
use crate::features::session::{setup_cross_tab_session_sync, setup_session_refresh_monitor};
use crate::routes;

use crate::pages::*;
use leptos::prelude::*;
use lib_ui::components::BrandingProvider;
use lib_ui::components::app_shell::AppShell;

use leptos_router::{
    ParamSegment, StaticSegment,
    components::{ParentRoute, Route, Router, Routes},
};

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
    crate::utils::search_registry::setup_search_providers();

    // Initialize global application state (including AuthencApiClient)
    let _app_state = crate::utils::app_state::provide_app_state();

    // Global auth state - load from localStorage on mount
    let (user_session, set_user_session) = signal(AuthService::load_session());

    // Global session context — any child can access via use_context()
    // (like Laravel Auth::user() or Next.js useSession())
    provide_context(user_session);
    provide_context(set_user_session);

    // Session timeout countdown (in seconds)
    let (timeout_countdown, set_timeout_countdown) = signal(0i64);
    let (show_timeout_warning, set_show_timeout_warning) = signal(false);

    // Expose the timeout-warning writers via context so descendants
    // (e.g. MainLayout's logout handler) can dismiss the modal before
    // navigating away — otherwise the modal could flash on the login
    // page if it was visible at logout time.
    provide_context(set_show_timeout_warning);
    provide_context(set_timeout_countdown);

    // Setup cross-tab session sync and token refresh/session timeout monitoring.
    setup_cross_tab_session_sync(
        set_user_session,
        set_show_timeout_warning,
        set_timeout_countdown,
    );
    setup_session_refresh_monitor(
        user_session,
        timeout_countdown,
        set_user_session,
        set_show_timeout_warning,
        set_timeout_countdown,
    );

    view! {
        <AppShell>
        <BrandingProvider unit="portal".to_string()>
            <Router base="/portal">
                <Routes fallback=|| view! { <NotFoundPage /> }>
                // ══════════════════════════════════════════════
                // PUBLIC ROUTES (no auth required)
                // ══════════════════════════════════════════════
                <Route path=StaticSegment(routes::segment::HOME) view=HomePage />
                <Route path=StaticSegment(routes::segment::LOGIN) view=move || view! {
                    <LoginPage on_login_success=set_user_session />
                } />
                <Route path=StaticSegment(routes::segment::CALLBACK) view=CallbackPage />
                <Route path=StaticSegment(routes::segment::LOGGED_OUT) view=LoggedOutPage />

                // MFA routes (semi-public, temp-token based)
                <Route path=StaticSegment(routes::segment::MFA_SETUP) view=MfaSetupPage />
                <Route path=StaticSegment(routes::segment::MFA_VERIFY) view=MfaVerificationPage />
                <Route path=StaticSegment(routes::segment::MFA_BACKUP_VERIFY) view=MfaBackupVerificationPage />

                // ══════════════════════════════════════════════
                // AUTHENTICATED ROUTES — PortalAuthLayout guards
                // ALL children (like Next.js layout.tsx / Laravel
                // Route::middleware('auth')->group())
                // ══════════════════════════════════════════════
                <ParentRoute path=StaticSegment("") view=move || view! {
                    <PortalAuthLayout user_session=user_session on_login_success=set_user_session />
                }>
                    // Layout pages (now read session from context)
                    <Route path=StaticSegment(routes::segment::DASHBOARD) view=PortalDashboardPage />
                    <Route path=StaticSegment(routes::segment::APPS) view=AppsPage />
                    <Route path=StaticSegment(routes::segment::NOTIFICATIONS) view=NotificationsPage />
                    <Route path=StaticSegment(routes::segment::SETTINGS) view=SettingsPage />
                    <Route path=StaticSegment(routes::segment::MFA_BACKUP_CODES) view=MfaBackupCodesPage />

                    // Self-service account management
                    <Route path=StaticSegment(routes::segment::PROFILE) view=ProfilePage />
                    <Route path=StaticSegment(routes::segment::PASSKEYS) view=PasskeysPage />
                    <Route path=StaticSegment(routes::segment::PASSWORD) view=PasswordChangePage />
                    <Route path=StaticSegment(routes::segment::SESSIONS) view=SessionsPage />
                </ParentRoute>

                // ══════════════════════════════════════════════
                // ADMIN ROUTES — PortalAdminLayout guards all
                // children (admin role required)
                // ══════════════════════════════════════════════
                <ParentRoute path=StaticSegment(routes::segment::ADMIN) view=move || view! {
                    <PortalAdminLayout user_session=user_session on_login_success=set_user_session />
                }>
                    <Route path=StaticSegment("") view=AdminOverviewPage />
                    <Route path=StaticSegment("users") view=UsersManagementPage />
                    <Route path=(StaticSegment("users"), ParamSegment("id")) view=UserDetailPage />
                    <Route path=StaticSegment("realms") view=RealmsManagementPage />
                    <Route path=StaticSegment("clients") view=ClientsManagementPage />
                    <Route path=(StaticSegment("clients"), ParamSegment("id")) view=ClientDetailPage />
                    <Route path=StaticSegment("roles") view=RolesManagementPage />
                    <Route path=StaticSegment("federation") view=FederationManagementPage />
                    <Route path=StaticSegment("permissions") view=PermissionsManagementPage />
                    <Route path=StaticSegment("audit") view=AuditLogsPage />
                    <Route path=StaticSegment("groups") view=GroupsManagementPage />
                    <Route path=StaticSegment("realm-settings") view=RealmSettingsPage />
                    <Route path=StaticSegment("auth-flows") view=AuthFlowsPage />
                    <Route path=StaticSegment("linked-accounts") view=LinkedAccountsPage />
                </ParentRoute>
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
        </AppShell>
    }
}
