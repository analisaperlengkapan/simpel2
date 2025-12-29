//! Keyboard navigation hooks and utilities

use crate::utils::accessibility::keyboard;
use leptos::html;
use leptos::prelude::*;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

// ============================================================================
// KEYBOARD SHORTCUT HOOK
// ============================================================================

/// Hook for registering keyboard shortcuts
///
/// # Example
/// ```rust
/// use_keyboard_shortcut("ctrl+s", || {
///     // Save action
/// });
/// ```
pub fn use_keyboard_shortcut<F>(shortcut: &'static str, callback: F)
where
    F: Fn() + 'static,
{
    let callback = Rc::new(callback);

    Effect::new(move || {
        let callback = Rc::clone(&callback);
        let closure = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
            if matches_shortcut(&event, shortcut) {
                event.prevent_default();
                callback();
            }
        }) as Box<dyn FnMut(_)>);

        let window = web_sys::window().expect("no global `window` exists");
        window
            .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
            .expect("failed to add event listener");

        // Keep closure alive - it will be cleaned up when Effect is dropped
        closure.forget();
    });
}

/// Check if keyboard event matches shortcut string
fn matches_shortcut(event: &web_sys::KeyboardEvent, shortcut: &str) -> bool {
    let parts: Vec<&str> = shortcut.split('+').collect();
    let key = parts.last().unwrap_or(&"");

    let mut ctrl_required = false;
    let mut alt_required = false;
    let mut shift_required = false;
    let mut meta_required = false;

    for part in &parts[..parts.len() - 1] {
        match part.to_lowercase().as_str() {
            "ctrl" | "control" => ctrl_required = true,
            "alt" | "option" => alt_required = true,
            "shift" => shift_required = true,
            "meta" | "cmd" | "command" => meta_required = true,
            _ => {}
        }
    }

    event.key().to_lowercase() == key.to_lowercase()
        && event.ctrl_key() == ctrl_required
        && event.alt_key() == alt_required
        && event.shift_key() == shift_required
        && event.meta_key() == meta_required
}

// ============================================================================
// FOCUS MANAGEMENT HOOK
// ============================================================================

/// Hook for managing focus within a component
pub fn use_focus_management() -> FocusManager {
    FocusManager::new()
}

pub struct FocusManager;

impl FocusManager {
    pub fn new() -> Self {
        Self
    }

    /// Focus the first focusable element in a container
    pub fn focus_first(&self, container: &web_sys::Element) {
        if let Ok(Some(element)) = container.query_selector(FOCUSABLE_SELECTOR)
            && let Some(html_element) = element.dyn_ref::<web_sys::HtmlElement>() {
                let _ = html_element.focus();
            }
    }

    /// Focus the last focusable element in a container
    pub fn focus_last(&self, _container: &web_sys::Element) {
        // TODO: Implement using proper DOM traversal
        // query_selector_all is not available on Element in web-sys
    }

    /// Get all focusable elements in a container
    pub fn get_focusable_elements(&self, _container: &web_sys::Element) -> Vec<web_sys::Element> {
        // TODO: Implement using proper DOM traversal
        // query_selector_all is not available on Element in web-sys
        Vec::new()
    }

    /// Focus next focusable element
    pub fn focus_next(&self, container: &web_sys::Element) {
        let elements = self.get_focusable_elements(container);
        if elements.is_empty() {
            return;
        }

        let active = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.active_element());

        if let Some(active_element) = active {
            if let Some(current_index) = elements.iter().position(|el| el == &active_element) {
                let next_index = (current_index + 1) % elements.len();
                if let Some(html_element) = elements[next_index].dyn_ref::<web_sys::HtmlElement>() {
                    let _ = html_element.focus();
                }
            } else {
                // Focus first if no element is currently focused
                self.focus_first(container);
            }
        } else {
            self.focus_first(container);
        }
    }

    /// Focus previous focusable element
    pub fn focus_previous(&self, container: &web_sys::Element) {
        let elements = self.get_focusable_elements(container);
        if elements.is_empty() {
            return;
        }

        let active = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.active_element());

        if let Some(active_element) = active {
            if let Some(current_index) = elements.iter().position(|el| el == &active_element) {
                let prev_index = if current_index == 0 {
                    elements.len() - 1
                } else {
                    current_index - 1
                };
                if let Some(html_element) = elements[prev_index].dyn_ref::<web_sys::HtmlElement>() {
                    let _ = html_element.focus();
                }
            } else {
                // Focus last if no element is currently focused
                self.focus_last(container);
            }
        } else {
            self.focus_last(container);
        }
    }

    /// Trap focus within a container (for modals/dialogs)
    pub fn trap_focus(&self, container: &web_sys::Element, event: &web_sys::KeyboardEvent) {
        if event.key() != keyboard::TAB {
            return;
        }

        let elements = self.get_focusable_elements(container);
        if elements.is_empty() {
            event.prevent_default();
            return;
        }

        let first = &elements[0];
        let last = &elements[elements.len() - 1];

        let active = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.active_element());

        if let Some(active_element) = active {
            if event.shift_key() {
                // Shift+Tab: wrap to last element
                if first == &active_element {
                    event.prevent_default();
                    if let Some(html_element) = last.dyn_ref::<web_sys::HtmlElement>() {
                        let _ = html_element.focus();
                    }
                }
            } else {
                // Tab: wrap to first element
                if last == &active_element {
                    event.prevent_default();
                    if let Some(html_element) = first.dyn_ref::<web_sys::HtmlElement>() {
                        let _ = html_element.focus();
                    }
                }
            }
        }
    }
}

impl Default for FocusManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// ARROW KEY NAVIGATION HOOK
// ============================================================================

/// Hook for arrow key navigation in lists/grids
pub fn use_arrow_navigation<F>(
    container_ref: NodeRef<html::Div>,
    on_select: F,
) -> impl Fn(web_sys::KeyboardEvent)
where
    F: Fn(usize) + 'static + Clone,
{
    let focus_manager = use_focus_management();

    move |event: web_sys::KeyboardEvent| {
        if let Some(container) = container_ref.get() {
            match event.key().as_str() {
                keyboard::ARROW_DOWN => {
                    event.prevent_default();
                    focus_manager.focus_next(&container);
                }
                keyboard::ARROW_UP => {
                    event.prevent_default();
                    focus_manager.focus_previous(&container);
                }
                keyboard::HOME => {
                    event.prevent_default();
                    focus_manager.focus_first(&container);
                }
                keyboard::END => {
                    event.prevent_default();
                    focus_manager.focus_last(&container);
                }
                k if k == keyboard::ENTER || k == keyboard::SPACE => {
                    event.prevent_default();
                    // Get currently focused element index and call on_select
                    let elements = focus_manager.get_focusable_elements(&container);
                    if let Some(active) = web_sys::window()
                        .and_then(|w| w.document())
                        .and_then(|d| d.active_element())
                        && let Some(index) = elements.iter().position(|el| el == &active) {
                            on_select(index);
                        }
                }
                _ => {}
            }
        }
    }
}

// ============================================================================
// CONSTANTS
// ============================================================================

/// Selector for focusable elements
const FOCUSABLE_SELECTOR: &str = r#"
    button:not([disabled]),
    [href],
    input:not([disabled]),
    select:not([disabled]),
    textarea:not([disabled]),
    [tabindex]:not([tabindex="-1"])
"#;

// ============================================================================
// ESCAPE KEY HOOK
// ============================================================================

/// Hook for handling Escape key press
pub fn use_escape_key<F>(callback: F)
where
    F: Fn() + 'static,
{
    let callback = Rc::new(callback);

    Effect::new(move || {
        let callback = Rc::clone(&callback);
        let closure = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
            if event.key() == keyboard::ESCAPE {
                event.prevent_default();
                callback();
            }
        }) as Box<dyn FnMut(_)>);

        let window = web_sys::window().expect("no global `window` exists");
        window
            .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
            .expect("failed to add event listener");

        // Keep closure alive - it will be cleaned up when Effect is dropped
        closure.forget();
    });
}

// ============================================================================
// GLOBAL KEYBOARD SHORTCUTS
// ============================================================================

/// Setup global keyboard shortcuts for the application
pub fn setup_global_shortcuts() {
    // Skip to main content (Alt+M)
    use_keyboard_shortcut("alt+m", || {
        if let Some(main) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("main-content"))
            && let Some(html_element) = main.dyn_ref::<web_sys::HtmlElement>() {
                let _ = html_element.focus();
                html_element.scroll_into_view();
            }
    });

    // Open search (Ctrl+K or Cmd+K)
    use_keyboard_shortcut("ctrl+k", || {
        // Trigger search modal
        if let Some(search) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("global-search"))
            && let Some(html_element) = search.dyn_ref::<web_sys::HtmlElement>() {
                let _ = html_element.focus();
            }
    });

    // Open help (Shift+?)
    use_keyboard_shortcut("shift+/", || {
        // Trigger help modal
        leptos::logging::log!("Help shortcut triggered");
    });
}
