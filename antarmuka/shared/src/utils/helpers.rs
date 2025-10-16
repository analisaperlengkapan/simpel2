//! DOM helper utilities

use wasm_bindgen::JsCast;
use web_sys::{Element, window};

/// Get window object
pub fn get_window() -> Option<web_sys::Window> {
    window()
}

/// Get document object
pub fn get_document() -> Option<web_sys::Document> {
    window()?.document()
}

/// Get element by ID
pub fn get_element_by_id(id: &str) -> Option<Element> {
    get_document()?.get_element_by_id(id)
}

/// Scroll to element smoothly
pub fn scroll_to_element(element_id: &str) {
    if let Some(element) = get_element_by_id(element_id) {
        element.scroll_into_view();
    }
}

/// Scroll to top of page
pub fn scroll_to_top() {
    if let Some(window) = get_window() {
        #[allow(clippy::let_unit_value)]
        let _ = window.scroll_to_with_x_and_y(0.0, 0.0);
    }
}

/// Get current URL path
pub fn get_current_path() -> Option<String> {
    window()?.location().pathname().ok()
}

/// Set page title
pub fn set_page_title(title: &str) {
    if let Some(document) = get_document() {
        document.set_title(title);
    }
}

/// Check if element is in viewport
pub fn is_in_viewport(element: &Element) -> bool {
    if let Some(window) = window() {
        let rect = element.get_bounding_client_rect();
        let window_height = window
            .inner_height()
            .ok()
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let window_width = window
            .inner_width()
            .ok()
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        rect.top() >= 0.0
            && rect.left() >= 0.0
            && rect.bottom() <= window_height
            && rect.right() <= window_width
    } else {
        false
    }
}

/// Add class to element
pub fn add_class(element: &Element, class_name: &str) {
    let _ = element.class_list().add_1(class_name);
}

/// Remove class from element
pub fn remove_class(element: &Element, class_name: &str) {
    let _ = element.class_list().remove_1(class_name);
}

/// Toggle class on element
pub fn toggle_class(element: &Element, class_name: &str) {
    let _ = element.class_list().toggle(class_name);
}

/// Focus element
pub fn focus_element(element_id: &str) {
    if let Some(element) = get_element_by_id(element_id)
        && let Some(html_element) = element.dyn_ref::<web_sys::HtmlElement>()
    {
        let _ = html_element.focus();
    }
}

/// Blur (unfocus) element
pub fn blur_element(element_id: &str) {
    if let Some(element) = get_element_by_id(element_id)
        && let Some(html_element) = element.dyn_ref::<web_sys::HtmlElement>()
    {
        let _ = html_element.blur();
    }
}

/// Generate unique ID
pub fn generate_id(prefix: &str) -> String {
    use std::sync::atomic::{AtomicU32, Ordering};
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    format!("{}-{}", prefix, COUNTER.fetch_add(1, Ordering::SeqCst))
}
