//! Authentication components for microfrontends
//!
//! Provides reusable authentication UI components

use crate::hooks::use_auth::use_auth;
use leptos::prelude::*;

/// Login redirect page component
#[component]
pub fn LoginRedirectPage(
    /// Optional custom application name
    #[prop(optional, into)]
    app_name: Option<String>,
    /// Optional custom application description
    #[prop(optional, into)]
    app_description: Option<String>,
) -> impl IntoView {
    let auth = use_auth();
    let default_app_name = crate::hooks::use_auth::get_app_name();
    let default_app_desc = crate::hooks::use_auth::get_app_description();

    let display_name = app_name.unwrap_or(default_app_name);
    let display_desc = app_description.unwrap_or(default_app_desc);

    // Check if user is already authenticated
    Effect::new(move || {
        if auth.is_authenticated() {
            // Already authenticated, redirect to dashboard
            // Use the current pathname base to determine the correct dashboard URL
            if let Some(window) = web_sys::window() {
                let current_path = window.location().pathname().unwrap_or_default();
                let dashboard_path = if current_path.starts_with("/perlengkapan") {
                    "/perlengkapan/dashboard"
                } else {
                    "/portal/dashboard"
                };
                let _ = window.location().set_href(dashboard_path);
            }
        }
    });

    let _auth_clone = auth;
    let handle_login = move || {
        // BYPASS: redirect directly to dashboard for testing
        if let Some(window) = web_sys::window() {
            let current_path = window.location().pathname().unwrap_or_default();
            let dashboard_path = if current_path.starts_with("/perlengkapan") {
                "/perlengkapan/dashboard"
            } else {
                "/portal/dashboard"
            };
            let _ = window.location().set_href(dashboard_path);
        }
    };

    view! {
        <div style="min-height: 100vh; display: flex; flex-direction: column; align-items: center; justify-content: center; padding: 2rem; background: linear-gradient(135deg, #0f172a 0%, #1e3a5f 50%, #0a1020 100%); font-family: system-ui, -apple-system, sans-serif; position: relative; overflow: hidden;">

            // Glowing orb background effect
            <div style="position: absolute; top: 15%; left: 50%; transform: translateX(-50%); width: 400px; height: 400px; background: radial-gradient(circle, rgba(212,168,67,0.12) 0%, transparent 70%); border-radius: 50%; filter: blur(40px); pointer-events: none;"></div>

            // Main content card
            <div style="position: relative; z-index: 10; width: 100%; max-width: 420px; background: rgba(255,255,255,0.05); backdrop-filter: blur(20px); -webkit-backdrop-filter: blur(20px); border: 1px solid rgba(255,255,255,0.1); border-radius: 24px; padding: 48px 40px; text-align: center; box-shadow: 0 25px 50px rgba(0,0,0,0.4);">

                // Kejaksaan Logo
                <div style="width: 120px; height: 120px; margin: 0 auto 24px auto; position: relative;">
                    <div style="position: absolute; inset: -10px; background: radial-gradient(circle, rgba(212,168,67,0.25) 0%, transparent 70%); border-radius: 50%; filter: blur(15px);"></div>
                    <img
                        src="/perlengkapan/assets/kejaksaan-logo.png"
                        alt="Logo Kejaksaan RI"
                        style="width: 100%; height: 100%; object-fit: contain; position: relative; filter: drop-shadow(0 4px 15px rgba(0,0,0,0.3));"
                    />
                </div>

                // App title
                <h1 style="font-size: 2rem; font-weight: 800; color: #ffffff; letter-spacing: -0.02em; margin: 0 0 8px 0; text-shadow: 0 2px 10px rgba(0,0,0,0.3);">
                    {display_name}
                </h1>
                <p style="font-size: 1rem; color: #94a3b8; font-weight: 500; margin: 0 0 8px 0; letter-spacing: 0.03em;">
                    {display_desc}
                </p>
                <p style="font-size: 0.8rem; color: #64748b; font-style: italic; margin: 0 0 32px 0;">
                    "\"Demi Keadilan Berdasarkan Ketuhanan Yang Maha Esa\""
                </p>

                // Gold divider
                <div style="width: 60px; height: 3px; background: linear-gradient(90deg, transparent, #d4a843, transparent); margin: 0 auto 32px auto; border-radius: 2px;"></div>

                // Login Button
                <button
                    on:click=move |_| handle_login()
                    style="display: flex; align-items: center; justify-content: center; gap: 12px; width: 100%; padding: 16px 24px; font-size: 1.1rem; font-weight: 700; color: #0f172a; background: linear-gradient(135deg, #facc15, #d4a843); border: none; border-radius: 14px; cursor: pointer; box-shadow: 0 0 25px rgba(212,168,67,0.35), 0 4px 15px rgba(0,0,0,0.2); transition: all 0.3s ease; letter-spacing: 0.02em;"
                >
                    <i class="fas fa-sign-in-alt" style="font-size: 1.2rem;"></i>
                    <span>"Masuk"</span>
                </button>

                // Footer text inside card
                <div style="margin-top: 32px; padding-top: 20px; border-top: 1px solid rgba(255,255,255,0.06);">
                    <p style="font-size: 0.7rem; color: #94a3b8; text-transform: uppercase; letter-spacing: 0.1em; margin: 0 0 4px 0; font-weight: 600;">"Kejaksaan Republik Indonesia"</p>
                    <p style="font-size: 0.65rem; color: #64748b; margin: 0;">"\u{00a9} 2025 SIMPEL v2.0"</p>
                </div>
            </div>
        </div>
    }
}


/// Protected route wrapper component
///
/// Wraps content that requires authentication. If user is not authenticated,
/// redirects to login page.
///
/// # Example
/// ```rust
/// use lib_ui::components::auth::ProtectedRoute;
///
/// #[component]
/// pub fn DashboardPage() -> impl IntoView {
///     view! {
///         <ProtectedRoute>
///             <div>"Protected dashboard content"</div>
///         </ProtectedRoute>
///     }
/// }
/// ```
#[component]
pub fn ProtectedRoute(
    /// Content to show when authenticated
    #[prop(into)]
    children: ViewFn,
    /// Optional: Required permission to access this route
    #[prop(optional)]
    required_permission: Option<String>,
) -> impl IntoView {
    let auth = use_auth();

    // Check authentication and redirect if needed
    let auth_check = auth;
    let perm_check = required_permission.clone();
    Effect::new(move || {
        if !auth_check.is_authenticated() {
            auth_check.redirect_to_login();
        } else if let Some(ref permission) = perm_check
            && !auth_check.has_permission(permission)
        {
            // User doesn't have required permission, show error or redirect
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href("/unauthorized");
            }
        }
    });

    let auth_show = auth;
    let perm_show = required_permission.clone();
    view! {
        <Show
            when=move || {
                auth_show.is_authenticated() && perm_show.as_ref()
                    .map(|p| auth_show.has_permission(p))
                    .unwrap_or(true)
            }
            fallback=|| view! {
                <div class="min-h-screen flex items-center justify-center">
                    <div class="text-center">
                        <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-primary mx-auto mb-4"></div>
                        <p class="text-gray-600">"Memuat..."</p>
                    </div>
                </div>
            }
        >
            {children.run()}
        </Show>
    }
}

/// Logout button component
///
/// Renders a button that logs out the user and redirects to portal.
///
/// # Example
/// ```rust
/// use lib_ui::components::auth::LogoutButton;
///
/// #[component]
/// pub fn Header() -> impl IntoView {
///     view! {
///         <header>
///             <LogoutButton />
///         </header>
///     }
/// }
/// ```
#[component]
pub fn LogoutButton(
    /// Optional custom class
    #[prop(optional)]
    class: String,
    /// Optional: Show as icon only
    #[prop(optional)]
    icon_only: bool,
) -> impl IntoView {
    let auth = use_auth();

    let auth_clone = auth;
    let handle_logout = move || {
        auth_clone.logout();
    };

    view! {
        <button
            class=format!("inline-flex items-center justify-center font-medium transition-colors focus:outline-none hover:bg-gray-100 text-gray-700 px-4 py-2 text-base rounded-md {}", class)
            on:click=move |_| handle_logout()
        >
            <i class="fas fa-sign-out-alt"></i>
            <Show when=move || !icon_only>
                <span class="ml-2">"Logout"</span>
            </Show>
        </button>
    }
}

/// User profile display component
///
/// Shows current user information with avatar and name.
///
/// # Example
/// ```rust
/// use lib_ui::components::auth::UserProfile;
///
/// #[component]
/// pub fn Header() -> impl IntoView {
///     view! {
///         <header>
///             <UserProfile />
///         </header>
///     }
/// }
/// ```
#[component]
pub fn UserProfile(
    /// Optional custom class
    #[prop(optional)]
    class: String,
) -> impl IntoView {
    let auth = use_auth();
    let auth_check = auth;
    let auth_session = auth;
    let class_stored = StoredValue::new(class);

    view! {
        <Show when=move || auth_check.is_authenticated()>
            {move || {
                let class_value = class_stored.get_value();
                auth_session.get_session().map(|session| {
                    view! {
                        <div class=format!("flex items-center space-x-3 {}", class_value)>
                            // Avatar
                            <div class="w-10 h-10 rounded-full bg-primary flex items-center justify-center text-white font-semibold">
                                {session.name.chars().next().unwrap_or('U').to_uppercase().to_string()}
                            </div>

                            // User info
                            <div class="hidden md:block">
                                <p class="text-sm font-medium text-gray-900 dark:text-white">
                                    {session.name}
                                </p>
                                <p class="text-xs text-gray-500 dark:text-gray-400">
                                    {session.role.display_name()}
                                </p>
                            </div>
                        </div>
                    }
                })
            }}
        </Show>
    }
}

/// Permission guard component
///
/// Shows content only if user has required permission.
///
/// # Example
/// ```rust
/// use lib_ui::components::auth::PermissionGuard;
///
/// #[component]
/// pub fn AdminPanel() -> impl IntoView {
///     view! {
///         <PermissionGuard permission="admin:*">
///             <div>"Admin only content"</div>
///         </PermissionGuard>
///     }
/// }
/// ```
#[component]
pub fn PermissionGuard(
    /// Required permission
    permission: String,
    /// Content to show when user has permission
    #[prop(into)]
    children: ViewFn,
) -> impl IntoView {
    let auth = use_auth();
    let perm = permission.clone();

    view! {
        <Show
            when=move || auth.has_permission(&perm)
            fallback=|| view! { <></> }
        >
            {children.run()}
        </Show>
    }
}
