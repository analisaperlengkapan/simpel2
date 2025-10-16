//! Authentication components for microfrontends
//!
//! Provides reusable authentication UI components

use crate::components::layout::*;
use crate::hooks::use_auth::{get_app_name, use_auth};
use leptos::prelude::*;

/// Login redirect page component
///
/// Shows a branded page with a login button that redirects to the portal.
/// This should be the default route for unauthenticated users.
///
/// # Example
/// ```rust
/// use shared_microfrontend::components::auth::LoginRedirectPage;
///
/// #[component]
/// pub fn App() -> impl IntoView {
///     view! {
///         <Router>
///             <Routes>
///                 <Route path="/" view=LoginRedirectPage />
///       <Route path="/dashboard" view=|| view! {
///                     <ProtectedRoute>
///                         <DashboardPage />
///                     </ProtectedRoute>
///                 } />
///             </Routes>
///         </Router>
///     }
/// }
/// ```
#[component]
pub fn LoginRedirectPage() -> impl IntoView {
    let auth = use_auth();
    let app_name = get_app_name();

    // Check if user is already authenticated
    Effect::new(move || {
        if auth.is_authenticated() {
            // Already authenticated, redirect to dashboard
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href("/dashboard");
            }
        }
    });

    let auth_clone = auth.clone();
    let handle_login = move || {
        auth_clone.redirect_to_login();
    };

    view! {
        <div class="min-h-screen flex items-center justify-center bg-gradient-to-br from-gray-50 to-gray-100 dark:from-gray-900 dark:to-gray-800 px-4">
            <Card class="max-w-md w-full">
                <div class="text-center space-y-6">
                    // Logo
                    <div class="flex justify-center">
                        <div class="w-20 h-20 bg-primary rounded-full flex items-center justify-center">
                            <i class="fas fa-balance-scale text-white text-3xl"></i>
                        </div>
                    </div>

                    // Title
                    <div>
                        <h1 class="text-3xl font-bold text-gray-900 dark:text-white mb-2">
                            {app_name}
                        </h1>
                        <p class="text-gray-600 dark:text-gray-400">
                            "Sistem Informasi Manajemen Perkara Elektronik"
                        </p>
                    </div>

                    // Description
                    <p class="text-gray-700 dark:text-gray-300">
                        "Silakan login untuk mengakses aplikasi"
                    </p>

                    // Login button
                    <button
                        class="w-full inline-flex items-center justify-center font-medium transition-colors focus:outline-none focus:ring-2 focus:ring-offset-2 bg-emerald-700 hover:bg-emerald-800 text-white focus:ring-emerald-500 px-6 py-3 text-lg rounded-lg"
                        on:click=move |_| handle_login()
                    >
                        <i class="fas fa-sign-in-alt mr-2"></i>
                        "Login ke Portal"
                    </button>

                    // Footer info
                    <div class="text-sm text-gray-500 dark:text-gray-400 pt-4 border-t border-gray-200 dark:border-gray-700">
                        <p>"Kejaksaan Republik Indonesia"</p>
                        <p class="mt-1">"© 2025 SIMPelv2"</p>
                    </div>
                </div>
            </Card>
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
/// use shared_microfrontend::components::auth::ProtectedRoute;
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
    let auth_check = auth.clone();
    let perm_check = required_permission.clone();
    Effect::new(move || {
        if !auth_check.is_authenticated() {
            auth_check.redirect_to_login();
        } else if let Some(ref permission) = perm_check {
            if !auth_check.has_permission(permission) {
                // User doesn't have required permission, show error or redirect
                if let Some(window) = web_sys::window() {
                    let _ = window.location().set_href("/unauthorized");
                }
            }
        }
    });

    let auth_show = auth.clone();
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
/// use shared_microfrontend::components::auth::LogoutButton;
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

    let auth_clone = auth.clone();
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
/// use shared_microfrontend::components::auth::UserProfile;
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
    let auth_check = auth.clone();
    let auth_session = auth.clone();
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
/// use shared_microfrontend::components::auth::PermissionGuard;
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
