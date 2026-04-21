//! # SIMPEL Portal Utama - Modern Government Portal
//!
//! Portal utama yang mengintegrasikan semua layanan SIMPEL dengan:
//! - **Modern UI/UX**: Design system berbasis Tailwind CSS
//! - **Performance Optimized**: Code splitting dan lazy loading
//! - **Accessibility**: WCAG 2.1 AA compliance
//! - **Government Branding**: Konsisten dengan identitas Kejaksaan RI

use crate::app_page_views;
use crate::app_routes::{
    render_session_admin_page, render_session_auth_page, render_session_layout_page,
};
use crate::components::session_timeout_modal::SessionTimeoutModal;
use crate::features::auth::AuthService;
use crate::features::session_monitor::{
    setup_cross_tab_session_sync, setup_session_refresh_monitor,
};
use crate::routes;

use crate::pages::*;
use leptos::prelude::*;
use lib_ui::components::BrandingProvider;

use leptos_router::{
    ParamSegment, StaticSegment,
    components::{Route, Router, Routes},
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

    // Provide the writer as context so child components (e.g. PasswordChangePage)
    // can update the reactive session signal without prop-drilling.
    provide_context(set_user_session);

    // Session timeout countdown (in seconds)
    let (timeout_countdown, set_timeout_countdown) = signal(0i64);
    let (show_timeout_warning, set_show_timeout_warning) = signal(false);

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
                // PROTECTED ROUTES (with MainLayout wrapper)
                // ══════════════════════════════════════════════
                <Route path=StaticSegment(routes::segment::MFA_BACKUP_CODES) view=move || {
                    render_session_layout_page(
                        user_session,
                        set_user_session,
                        &make_logout,
                        |session, on_logout| {
                            view! {
                                <MfaBackupCodesPage user_session=session on_logout=on_logout />
                            }
                            .into_any()
                        },
                    )
                } />

                <Route path=StaticSegment(routes::segment::DASHBOARD) view=move || {
                    render_session_layout_page(
                        user_session,
                        set_user_session,
                        &make_logout,
                        |session, on_logout| {
                            view! {
                                <PortalDashboardPage user_session=session on_logout=on_logout />
                            }
                            .into_any()
                        },
                    )
                } />

                <Route path=StaticSegment(routes::segment::APPS) view=move || {
                    render_session_layout_page(
                        user_session,
                        set_user_session,
                        &make_logout,
                        |session, on_logout| {
                            view! {
                                <AppsPage user_session=session on_logout=on_logout />
                            }
                            .into_any()
                        },
                    )
                } />

                <Route path=StaticSegment(routes::segment::NOTIFICATIONS) view=move || {
                    render_session_layout_page(
                        user_session,
                        set_user_session,
                        &make_logout,
                        |session, on_logout| {
                            view! {
                                <NotificationsPage user_session=session on_logout=on_logout />
                            }
                            .into_any()
                        },
                    )
                } />

                <Route path=StaticSegment(routes::segment::SETTINGS) view=move || {
                    render_session_layout_page(
                        user_session,
                        set_user_session,
                        &make_logout,
                        |session, on_logout| {
                            view! {
                                <SettingsPage user_session=session on_logout=on_logout />
                            }
                            .into_any()
                        },
                    )
                } />

                // ══════════════════════════════════════════════
                // SELF-SERVICE ACCOUNT MANAGEMENT (WithAuth guard)
                // ══════════════════════════════════════════════
                <Route path=StaticSegment(routes::segment::PROFILE) view=move || {
                    render_session_auth_page(
                        user_session,
                        set_user_session,
                        false,
                        app_page_views::profile_page,
                    )
                } />
                <Route path=StaticSegment(routes::segment::PASSKEYS) view=move || {
                    render_session_auth_page(
                        user_session,
                        set_user_session,
                        false,
                        app_page_views::passkeys_page,
                    )
                } />
                <Route path=StaticSegment(routes::segment::PASSWORD) view=move || {
                    render_session_auth_page(
                        user_session,
                        set_user_session,
                        true,
                        app_page_views::password_change_page,
                    )
                } />
                <Route path=StaticSegment(routes::segment::SESSIONS) view=move || {
                    render_session_auth_page(
                        user_session,
                        set_user_session,
                        false,
                        app_page_views::sessions_page,
                    )
                } />

                // ══════════════════════════════════════════════
                // ADMIN IAM ROUTES (WithAdminAuth guard)
                // ══════════════════════════════════════════════
                <Route path=StaticSegment(routes::segment::ADMIN) view=move || {
                    render_session_admin_page(
                        user_session,
                        set_user_session,
                        app_page_views::admin_overview_page,
                    )
                } />
                <Route path=StaticSegment(routes::segment::ADMIN_USERS) view=move || {
                    render_session_admin_page(
                        user_session,
                        set_user_session,
                        app_page_views::admin_users_page,
                    )
                } />
                <Route path=(StaticSegment(routes::segment::ADMIN_USERS), ParamSegment("id")) view=move || {
                    render_session_admin_page(
                        user_session,
                        set_user_session,
                        app_page_views::admin_user_detail_page,
                    )
                } />
                <Route path=StaticSegment(routes::segment::ADMIN_REALMS) view=move || {
                    render_session_admin_page(
                        user_session,
                        set_user_session,
                        app_page_views::admin_realms_page,
                    )
                } />
                <Route path=StaticSegment(routes::segment::ADMIN_CLIENTS) view=move || {
                    render_session_admin_page(
                        user_session,
                        set_user_session,
                        app_page_views::admin_clients_page,
                    )
                } />
                <Route path=(StaticSegment(routes::segment::ADMIN_CLIENTS), ParamSegment("id")) view=move || {
                    render_session_admin_page(
                        user_session,
                        set_user_session,
                        app_page_views::admin_client_detail_page,
                    )
                } />
                <Route path=StaticSegment(routes::segment::ADMIN_ROLES) view=move || {
                    render_session_admin_page(
                        user_session,
                        set_user_session,
                        app_page_views::admin_roles_page,
                    )
                } />
                <Route path=StaticSegment(routes::segment::ADMIN_FEDERATION) view=move || {
                    render_session_admin_page(
                        user_session,
                        set_user_session,
                        app_page_views::admin_federation_page,
                    )
                } />
                <Route path=StaticSegment(routes::segment::ADMIN_PERMISSIONS) view=move || {
                    render_session_admin_page(
                        user_session,
                        set_user_session,
                        app_page_views::admin_permissions_page,
                    )
                } />
                <Route path=StaticSegment(routes::segment::ADMIN_AUDIT) view=move || {
                    render_session_admin_page(
                        user_session,
                        set_user_session,
                        app_page_views::admin_audit_page,
                    )
                } />
                <Route path=StaticSegment(routes::segment::ADMIN_GROUPS) view=move || {
                    render_session_admin_page(
                        user_session,
                        set_user_session,
                        app_page_views::admin_groups_page,
                    )
                } />
                <Route path=StaticSegment(routes::segment::ADMIN_REALM_SETTINGS) view=move || {
                    render_session_admin_page(
                        user_session,
                        set_user_session,
                        app_page_views::admin_realm_settings_page,
                    )
                } />
                <Route path=StaticSegment(routes::segment::ADMIN_AUTH_FLOWS) view=move || {
                    render_session_admin_page(
                        user_session,
                        set_user_session,
                        app_page_views::admin_auth_flows_page,
                    )
                } />
                <Route path=StaticSegment(routes::segment::ADMIN_LINKED_ACCOUNTS) view=move || {
                    render_session_admin_page(
                        user_session,
                        set_user_session,
                        app_page_views::admin_linked_accounts_page,
                    )
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
