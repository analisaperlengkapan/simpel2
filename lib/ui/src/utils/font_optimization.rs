//! Font loading optimization utilities
//!
//! Provides utilities for optimizing web font loading to improve performance

/// Preload critical fonts for better performance
/// Use this to preload fonts that are needed immediately on page load
#[cfg(target_arch = "wasm32")]
pub fn preload_font(href: &str, font_type: FontType) {
    use wasm_bindgen::JsCast;

    if let Some(window) = web_sys::window()
        && let Some(document) = window.document()
        && let Ok(link) = document.create_element("link")
    {
        let link = link.dyn_into::<web_sys::HtmlLinkElement>().unwrap();
        link.set_rel("preload");
        link.set_as("font");
        link.set_href(href);
        link.set_attribute("type", font_type.mime_type()).ok();
        link.set_attribute("crossorigin", "anonymous").ok();

        let _ = document
            .head()
            .and_then(|head| head.append_child(&link).ok());
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn preload_font(_href: &str, _font_type: FontType) {
    // No-op on server side
}

/// Font types for preloading
#[derive(Clone, Copy, Debug)]
pub enum FontType {
    Woff2,
    Woff,
    Ttf,
    Otf,
}

impl FontType {
    pub fn mime_type(&self) -> &'static str {
        match self {
            FontType::Woff2 => "font/woff2",
            FontType::Woff => "font/woff",
            FontType::Ttf => "font/ttf",
            FontType::Otf => "font/otf",
        }
    }
}

/// Setup font display swap for better perceived performance
/// This ensures text is visible immediately with fallback fonts
/// while custom fonts are loading
#[cfg(target_arch = "wasm32")]
pub fn setup_font_display_swap() {
    use wasm_bindgen::JsCast;

    if let Some(window) = web_sys::window()
        && let Some(document) = window.document()
    {
        // Add CSS to enable font-display: swap
        if let Ok(style) = document.create_element("style") {
            let style = style.dyn_into::<web_sys::HtmlStyleElement>().unwrap();
            style.set_inner_html(
                    r#"
                    @font-face {
                        font-family: 'Inter';
                        font-style: normal;
                        font-weight: 400;
                        font-display: swap;
                        src: local('Inter'), url('/fonts/inter-regular.woff2') format('woff2');
                    }
                    @font-face {
                        font-family: 'Inter';
                        font-style: normal;
                        font-weight: 500;
                        font-display: swap;
                        src: local('Inter Medium'), url('/fonts/inter-medium.woff2') format('woff2');
                    }
                    @font-face {
                        font-family: 'Inter';
                        font-style: normal;
                        font-weight: 600;
                        font-display: swap;
                        src: local('Inter SemiBold'), url('/fonts/inter-semibold.woff2') format('woff2');
                    }
                    @font-face {
                        font-family: 'Inter';
                        font-style: normal;
                        font-weight: 700;
                        font-display: swap;
                        src: local('Inter Bold'), url('/fonts/inter-bold.woff2') format('woff2');
                    }
                    "#,
                );

            let _ = document
                .head()
                .and_then(|head| head.append_child(&style).ok());
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn setup_font_display_swap() {
    // No-op on server side
}

/// Check if fonts are loaded
#[cfg(target_arch = "wasm32")]
pub async fn wait_for_fonts_loaded() {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;

    if let Some(window) = web_sys::window()
        && let Some(document) = window.document()
    {
        // Use document.fonts.ready promise
        if let Ok(fonts) = js_sys::Reflect::get(&document, &"fonts".into())
            && let Ok(ready) = js_sys::Reflect::get(&fonts, &"ready".into())
            && let Ok(promise) = ready.dyn_into::<js_sys::Promise>()
        {
            let _ = JsFuture::from(promise).await;
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn wait_for_fonts_loaded() {
    // No-op on server side
}

/// Subset fonts to only include characters used in the application
/// This is a build-time optimization that should be done during asset processing
pub fn get_font_subset_characters() -> &'static str {
    // Indonesian alphabet + common punctuation + numbers
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789.,;:!?-–—()[]{}\"'/@#$%&*+=<>|~`"
}

/// Font loading strategy configuration
#[derive(Clone, Debug)]
pub struct FontLoadingStrategy {
    /// Use font-display: swap
    pub use_swap: bool,
    /// Preload critical fonts
    pub preload_critical: bool,
    /// Subset fonts
    pub use_subset: bool,
    /// Use variable fonts
    pub use_variable: bool,
}

impl Default for FontLoadingStrategy {
    fn default() -> Self {
        Self {
            use_swap: true,
            preload_critical: true,
            use_subset: true,
            use_variable: false, // Not all browsers support variable fonts yet
        }
    }
}

impl FontLoadingStrategy {
    /// Apply the font loading strategy
    #[cfg(target_arch = "wasm32")]
    pub fn apply(&self) {
        if self.use_swap {
            setup_font_display_swap();
        }

        if self.preload_critical {
            // Preload the most commonly used font weights
            preload_font("/fonts/inter-regular.woff2", FontType::Woff2);
            preload_font("/fonts/inter-medium.woff2", FontType::Woff2);
            preload_font("/fonts/inter-semibold.woff2", FontType::Woff2);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn apply(&self) {
        // No-op on server side
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_type_mime_type() {
        assert_eq!(FontType::Woff2.mime_type(), "font/woff2");
        assert_eq!(FontType::Woff.mime_type(), "font/woff");
        assert_eq!(FontType::Ttf.mime_type(), "font/ttf");
        assert_eq!(FontType::Otf.mime_type(), "font/otf");
    }

    #[test]
    fn test_font_subset_characters() {
        let chars = get_font_subset_characters();
        assert!(chars.contains('A'));
        assert!(chars.contains('z'));
        assert!(chars.contains('0'));
        assert!(chars.contains('9'));
        assert!(chars.contains('.'));
    }

    #[test]
    fn test_default_font_loading_strategy() {
        let strategy = FontLoadingStrategy::default();
        assert!(strategy.use_swap);
        assert!(strategy.preload_critical);
        assert!(strategy.use_subset);
        assert!(!strategy.use_variable);
    }
}
