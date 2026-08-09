//! Code splitting utilities for lazy loading components and routes
//!
//! Provides utilities for:
//! - Lazy loading components
//! - Route-based code splitting
//! - Dynamic imports
//! - Bundle size optimization
//!
//! # Code Splitting Strategy
//!
//! This module implements route-based code splitting to reduce initial bundle size:
//! - Critical routes (home, login) are eagerly loaded
//! - Feature routes (dashboard, apps) are lazy loaded
//! - Large components are split into separate chunks
//! - Preloading on hover for better UX
//!
//! **Note**: True lazy loading with dynamic imports is not yet fully supported
//! in Leptos 0.8 due to WASM limitations. The current approach focuses on:
//! - Build-time code splitting via Trunk configuration
//! - Route-based chunking through separate WASM modules
//! - Preloading strategies for better perceived performance
//!
//! Instead of runtime lazy loading, we use build-time optimization:
//!
//! 1. **Separate Microfrontends**: Each microfrontend is a separate WASM bundle
//! 2. **Optimized Builds**: Use `-Oz` optimization for smaller bundles
//! 3. **Preloading**: Prefetch routes on hover for instant navigation
//! 4. **Compression**: Enable gzip/brotli in Nginx for 70%+ size reduction
//!
//! # Example: Route Organization
//!
//! ```rust
//! use leptos::prelude::*;
//! use leptos_router::{components::{Router, Routes, Route}, StaticSegment};
//! use lib_ui::utils::code_splitting::RouteLoadingSkeleton;
//!
//! // Placeholder components
//! #[component] fn HomePage() -> impl IntoView { view! { "Home" } }
//! #[component] fn LoginPage() -> impl IntoView { view! { "Login" } }
//! #[component] fn DashboardPage() -> impl IntoView { view! { "Dashboard" } }
//! #[component] fn AppsPage() -> impl IntoView { view! { "Apps" } }
//!
//! // Organize routes by criticality
//! #[component]
//! pub fn App() -> impl IntoView {
//!     view! {
//!         <Router>
//!             <Routes fallback=|| "Not found">
//!                 // Critical routes - always loaded
//!                 <Route path=StaticSegment("/") view=HomePage />
//!                 <Route path=StaticSegment("/login") view=LoginPage />
//!
//!                 // Feature routes - loaded on demand
//!                 <Route path=StaticSegment("/dashboard") view=DashboardPage />
//!                 <Route path=StaticSegment("/apps") view=AppsPage />
//!             </Routes>
//!         </Router>
//!     }
//! }
//! ```
//!
//! # Preloading Example
//!
//! ```rust
//! use leptos::prelude::*;
//! use lib_ui::utils::code_splitting::preload_route;
//!
//! #[component]
//! pub fn NavLink() -> impl IntoView {
//!     view! {
//!         <a
//!             href="/dashboard"
//!             on:mouseenter=move |_| {
//!                 preload_route("dashboard");
//!             }
//!         >
//!             "Dashboard"
//!         </a>
//!     }
//! }
//! ```

use leptos::prelude::*;

/// Default loading skeleton for lazy-loaded routes
///
/// Provides a consistent loading experience across the application
#[component]
pub fn RouteLoadingSkeleton() -> impl IntoView {
    view! {
        <div class="min-h-screen bg-gray-50 animate-pulse">
            // Header skeleton
            <div class="bg-white shadow">
                <div class="container mx-auto px-4 py-4">
                    <div class="flex items-center justify-between">
                        <div class="h-8 w-48 bg-gray-200 rounded"></div>
                        <div class="flex space-x-4">
                            <div class="h-8 w-24 bg-gray-200 rounded"></div>
                            <div class="h-8 w-24 bg-gray-200 rounded"></div>
                        </div>
                    </div>
                </div>
            </div>

            // Content skeleton
            <div class="container mx-auto px-4 py-8">
                <div class="space-y-6">
                    <div class="h-12 w-64 bg-gray-200 rounded"></div>
                    <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                        <div class="h-32 bg-gray-200 rounded"></div>
                        <div class="h-32 bg-gray-200 rounded"></div>
                        <div class="h-32 bg-gray-200 rounded"></div>
                    </div>
                    <div class="h-64 bg-gray-200 rounded"></div>
                </div>
            </div>
        </div>
    }
}

/// Preload a route component before navigation
///
/// This allows preloading components when user hovers over a link
/// to improve perceived performance.
///
/// # Example
/// ```rust
/// use lib_ui::utils::code_splitting::preload_route;
/// use leptos::prelude::*;
///
/// #[component]
/// pub fn NavLink() -> impl leptos::IntoView {
///     view! {
///         <a
///             href="/dashboard"
///             on:mouseenter=move |_| {
///                 preload_route("dashboard");
///             }
///         >
///             "Dashboard"
///         </a>
///     }
/// }
/// ```
#[cfg(target_arch = "wasm32")]
pub fn preload_route(route: &str) {
    use wasm_bindgen::prelude::*;

    // Store preloaded routes in a global cache
    thread_local! {
        static PRELOADED_ROUTES: std::cell::RefCell<std::collections::HashSet<String>> =
            std::cell::RefCell::new(std::collections::HashSet::new());
    }

    PRELOADED_ROUTES.with(|cache| {
        let mut cache = cache.borrow_mut();
        if !cache.contains(route) {
            cache.insert(route.to_string());
            // Trigger preload via link prefetch
            if let Some(document) = web_sys::window().and_then(|w| w.document())
                && let Ok(link) = document.create_element("link")
            {
                let link = link.dyn_into::<web_sys::HtmlLinkElement>().unwrap();
                link.set_rel("prefetch");
                link.set_href(&format!("/{}", route));
                let _ = document
                    .head()
                    .and_then(|head| head.append_child(&link).ok());
            }
        }
    });
}

#[cfg(not(target_arch = "wasm32"))]
pub fn preload_route(_route: &str) {
    // No-op on server side
}

/// Get bundle size information for monitoring
///
/// Returns the size of the WASM bundle and JS glue code by analyzing
/// the Performance API resource timing entries.
#[cfg(target_arch = "wasm32")]
pub fn get_bundle_size() -> BundleSize {
    let window = web_sys::window().expect("no global window");
    let performance = window.performance().expect("no performance object");

    // Get resource timing entries
    let entries = performance.get_entries_by_type("resource");
    let mut wasm_size = 0;
    let mut js_size = 0;

    // Iterate through resources to find WASM and JS files
    for i in 0..entries.length() {
        let entry = entries.get(i);
        // Try to get the name property from the entry
        if let Ok(name) = js_sys::Reflect::get(&entry, &"name".into())
            && let Some(name_str) = name.as_string()
        {
            // Try to get transfer size or encoded body size
            let size = if let Ok(transfer_size) =
                js_sys::Reflect::get(&entry, &"transferSize".into())
            {
                transfer_size.as_f64().unwrap_or(0.0) as usize
            } else if let Ok(encoded_size) = js_sys::Reflect::get(&entry, &"encodedBodySize".into())
            {
                encoded_size.as_f64().unwrap_or(0.0) as usize
            } else {
                0
            };

            if name_str.ends_with(".wasm") {
                wasm_size += size;
            } else if name_str.ends_with(".js") {
                js_size += size;
            }
        }
    }

    BundleSize {
        wasm_bytes: wasm_size,
        js_bytes: js_size,
        total_bytes: wasm_size + js_size,
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_bundle_size() -> BundleSize {
    BundleSize {
        wasm_bytes: 0,
        js_bytes: 0,
        total_bytes: 0,
    }
}

/// Analyze and log bundle size on application load
///
/// Call this function in your app initialization to track bundle sizes
/// and get optimization recommendations.
///
/// # Example
/// ```rust
/// use lib_ui::utils::code_splitting::analyze_bundle_size;
///
/// #[component]
/// pub fn App() -> impl IntoView {
///     // Analyze bundle size on mount
///     create_effect(move |_| {
///         analyze_bundle_size();
///     });
///
///     view! { /* ... */ }
/// }
/// ```
#[cfg(target_arch = "wasm32")]
pub fn analyze_bundle_size() {
    use gloo_timers::future::TimeoutFuture;
    use leptos::task::spawn_local;

    // Wait for resources to load before analyzing
    spawn_local(async move {
        TimeoutFuture::new(2000).await;

        let bundle_size = get_bundle_size();

        // Log bundle information
        web_sys::console::group_1(&"📦 Bundle Size Analysis".into());
        web_sys::console::log_1(
            &format!("WASM: {}", BundleSize::format_size(bundle_size.wasm_bytes)).into(),
        );
        web_sys::console::log_1(
            &format!(
                "JavaScript: {}",
                BundleSize::format_size(bundle_size.js_bytes)
            )
            .into(),
        );
        web_sys::console::log_1(
            &format!(
                "Total: {}",
                BundleSize::format_size(bundle_size.total_bytes)
            )
            .into(),
        );

        // Log optimization status
        if bundle_size.is_optimal() {
            web_sys::console::log_1(&"✅ Bundle size is optimal!".into());
        } else {
            web_sys::console::warn_1(&"⚠️ Bundle size exceeds recommendations".into());
            for suggestion in bundle_size.get_suggestions() {
                web_sys::console::log_1(&format!("  • {}", suggestion).into());
            }
        }

        web_sys::console::group_end();
    });
}

#[cfg(not(target_arch = "wasm32"))]
pub fn analyze_bundle_size() {
    // No-op on server side
}

/// Bundle size information
#[derive(Debug, Clone, Copy)]
pub struct BundleSize {
    pub wasm_bytes: usize,
    pub js_bytes: usize,
    pub total_bytes: usize,
}

impl BundleSize {
    /// Format size in human-readable format
    pub fn format_size(bytes: usize) -> String {
        const KB: usize = 1024;
        const MB: usize = KB * 1024;

        if bytes >= MB {
            format!("{:.2} MB", bytes as f64 / MB as f64)
        } else if bytes >= KB {
            format!("{:.2} KB", bytes as f64 / KB as f64)
        } else {
            format!("{} bytes", bytes)
        }
    }

    /// Check if bundle size is within recommended limits
    pub fn is_optimal(&self) -> bool {
        // Recommended: WASM < 400KB, Total < 500KB
        self.wasm_bytes < 400 * 1024 && self.total_bytes < 500 * 1024
    }

    /// Get optimization suggestions
    pub fn get_suggestions(&self) -> Vec<String> {
        let mut suggestions = Vec::new();

        if self.wasm_bytes > 400 * 1024 {
            suggestions.push(format!(
                "WASM bundle is {} (recommended < 400KB). Consider code splitting.",
                Self::format_size(self.wasm_bytes)
            ));
        }

        if self.total_bytes > 500 * 1024 {
            suggestions.push(format!(
                "Total bundle is {} (recommended < 500KB). Enable compression.",
                Self::format_size(self.total_bytes)
            ));
        }

        if suggestions.is_empty() {
            suggestions.push("Bundle size is optimal! 🎉".to_string());
        }

        suggestions
    }
}

/// Measure component render time for performance monitoring
///
/// Uses the Performance API to measure how long a component takes to render.
/// Results are logged to the console and can be viewed in browser DevTools.
///
/// # Example
/// ```rust
/// use lib_ui::utils::code_splitting::measure_render_time;
/// use leptos::prelude::*;
///
/// #[component]
/// pub fn ExpensiveComponent() -> impl IntoView {
///     measure_render_time("ExpensiveComponent", || {
///         view! {
///             <div>"Complex rendering logic"</div>
///         }
///     })
/// }
/// ```
pub fn measure_render_time<F, V>(_name: &str, render_fn: F) -> V
where
    F: FnOnce() -> V,
{
    #[cfg(target_arch = "wasm32")]
    {
        let window = web_sys::window().expect("no global window");
        let performance = window.performance().expect("no performance object");

        let start_mark = format!("{}-start", _name);
        let end_mark = format!("{}-end", _name);
        let measure_name = format!("{}-render", _name);

        let _ = performance.mark(&start_mark);
        let result = render_fn();
        let _ = performance.mark(&end_mark);

        if performance
            .measure_with_start_mark_and_end_mark(&measure_name, &start_mark, &end_mark)
            .is_ok()
        {
            // Get the measure and log it
            let entries = performance.get_entries_by_name(&measure_name);
            let entry = entries.get(0);
            // Try to get duration from the entry
            if let Ok(duration) = js_sys::Reflect::get(&entry, &"duration".into())
                && let Some(duration_val) = duration.as_f64()
            {
                web_sys::console::log_1(
                    &format!("⏱️ {} rendered in {:.2}ms", _name, duration_val).into(),
                );
            }
        }

        result
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        render_fn()
    }
}

/// Track route navigation performance
///
/// Measures the time it takes to navigate between routes and load components.
/// Useful for identifying slow route transitions.
#[cfg(target_arch = "wasm32")]
pub fn track_route_navigation(_from: &str, _to: &str) {
    let window = web_sys::window().expect("no global window");
    let performance = window.performance().expect("no performance object");

    let mark_name = format!("route-{}-to-{}", _from, _to);
    let _ = performance.mark(&mark_name);

    web_sys::console::log_1(&format!("🧭 Navigating from {} to {}", _from, _to).into());
}

#[cfg(not(target_arch = "wasm32"))]
pub fn track_route_navigation(_from: &str, _to: &str) {
    // No-op on server side
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bundle_size_formatting() {
        assert_eq!(BundleSize::format_size(500), "500 bytes");
        assert_eq!(BundleSize::format_size(1024), "1.00 KB");
        assert_eq!(BundleSize::format_size(1024 * 1024), "1.00 MB");
    }

    #[test]
    fn test_bundle_size_optimal() {
        let optimal = BundleSize {
            wasm_bytes: 300 * 1024,
            js_bytes: 100 * 1024,
            total_bytes: 400 * 1024,
        };
        assert!(optimal.is_optimal());

        let too_large = BundleSize {
            wasm_bytes: 500 * 1024,
            js_bytes: 100 * 1024,
            total_bytes: 600 * 1024,
        };
        assert!(!too_large.is_optimal());
    }
}
