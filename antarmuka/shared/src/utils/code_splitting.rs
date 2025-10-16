//! Code splitting utilities for lazy loading components and routes
//!
//! Provides utilities for:
//! - Lazy loading components
//! - Route-based code splitting
//! - Dynamic imports
//! - Bundle size optimization

use leptos::prelude::*;
use std::future::Future;

/// Lazy load a component with a loading fallback
///
/// # Example
/// ```rust
/// use shared_microfrontend::utils::code_splitting::lazy_component;
///
/// #[component]
/// pub fn App() -> impl IntoView {
///     view! {
///         {lazy_component(
///             || async { DashboardPage() },
///             || view! { <div>"Loading..."</div> }
///         )}
///     }
/// }
/// ```
pub fn lazy_component<F, Fut, V>(
    _loader: F,
    fallback: impl Fn() -> AnyView + Send + 'static,
) -> impl IntoView
where
    F: Fn() -> Fut + Clone + Send + 'static,
    Fut: Future<Output = V> + 'static,
    V: IntoView + 'static,
{
    // TODO: Implement proper lazy loading with Suspense
    // Current limitation: AnyView doesn't implement Send+Sync required for signals
    // For now, just show fallback - lazy loading not fully supported yet
    fallback().into_any()
}

/// Preload a route component before navigation
///
/// This allows preloading components when user hovers over a link
/// to improve perceived performance.
///
/// # Example
/// ```rust
/// use shared_microfrontend::utils::code_splitting::preload_route;
///
/// #[component]
/// pub fn NavLink() -> impl IntoView {
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
            if let Some(document) = web_sys::window().and_then(|w| w.document()) {
                if let Ok(link) = document.create_element("link") {
                    let link = link.dyn_into::<web_sys::HtmlLinkElement>().unwrap();
                    link.set_rel("prefetch");
                    link.set_href(&format!("/{}", route));
                    let _ = document
                        .head()
                        .and_then(|head| head.append_child(&link).ok());
                }
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
/// Returns the size of the WASM bundle and JS glue code
#[cfg(target_arch = "wasm32")]
pub fn get_bundle_size() -> BundleSize {
    use wasm_bindgen::JsCast;

    let window = web_sys::window().expect("no global window");
    let performance = window.performance().expect("no performance object");

    // Get resource timing entries
    let entries = performance.get_entries_by_type("resource");
    let mut wasm_size = 0;
    let mut js_size = 0;

    // For now, return placeholder values since PerformanceResourceTiming
    // is not available in the current web_sys version
    // TODO: Implement proper bundle size detection when web_sys is updated
    let _ = entries; // Suppress unused variable warning

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
        let _ =
            performance.measure_with_start_mark_and_end_mark(&measure_name, &start_mark, &end_mark);

        result
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        render_fn()
    }
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
