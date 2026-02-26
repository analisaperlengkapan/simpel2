//! Asset optimization utilities for the portal
//!
//! Provides initialization and utilities for optimizing asset loading

use lib_ui::components::optimized_image::preload_image;
use lib_ui::utils::font_optimization::FontLoadingStrategy;

/// Initialize asset optimization for the portal
/// This should be called early in the application lifecycle
pub fn init_asset_optimization() {
    // Apply font loading strategy
    let font_strategy = FontLoadingStrategy::default();
    font_strategy.apply();

    // Preload critical images
    preload_critical_images();
}

/// Preload critical images that are needed immediately
fn preload_critical_images() {
    // Preload logo
    preload_image("/assets/logo.png");

    // Preload favicon
    preload_image("/favicon.png");

    // Preload common icons (if using local icons instead of Font Awesome)
    // preload_image("/assets/icons/dashboard.svg");
    // preload_image("/assets/icons/apps.svg");
}

/// Get list of critical CSS files to preload
pub fn get_critical_css() -> Vec<&'static str> {
    vec![
        // Tailwind CSS is loaded via CDN in development
        // In production, this should be a local file
        // "/styles/main.css",
    ]
}

/// Get list of critical fonts to preload
pub fn get_critical_fonts() -> Vec<&'static str> {
    vec![
        "/fonts/inter-regular.woff2",
        "/fonts/inter-medium.woff2",
        "/fonts/inter-semibold.woff2",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_critical_fonts() {
        let fonts = get_critical_fonts();
        assert!(!fonts.is_empty());
        assert!(fonts.contains(&"/fonts/inter-regular.woff2"));
    }
}
