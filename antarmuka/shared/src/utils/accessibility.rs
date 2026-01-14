//! Accessibility utilities and helpers for WCAG 2.1 AA compliance

use leptos::html;
use leptos::prelude::*;
use wasm_bindgen::JsCast;

// ============================================================================
// ARIA LIVE REGION TYPES
// ============================================================================

/// ARIA live region politeness levels
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AriaLive {
    /// Polite: Announce when user is idle
    Polite,
    /// Assertive: Announce immediately
    Assertive,
    /// Off: Do not announce
    Off,
}

impl AriaLive {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Polite => "polite",
            Self::Assertive => "assertive",
            Self::Off => "off",
        }
    }
}

// ============================================================================
// ARIA ROLE TYPES
// ============================================================================

/// Common ARIA roles for semantic HTML
#[derive(Clone, Debug, PartialEq)]
pub enum AriaRole {
    Alert,
    AlertDialog,
    Application,
    Article,
    Banner,
    Button,
    Checkbox,
    Complementary,
    ContentInfo,
    Dialog,
    Document,
    Feed,
    Form,
    Grid,
    GridCell,
    Group,
    Heading,
    Img,
    Link,
    List,
    ListBox,
    ListItem,
    Main,
    Menu,
    MenuBar,
    MenuItem,
    MenuItemCheckbox,
    MenuItemRadio,
    Navigation,
    None,
    Note,
    Option,
    Presentation,
    ProgressBar,
    Radio,
    RadioGroup,
    Region,
    Row,
    RowGroup,
    Search,
    Separator,
    Slider,
    SpinButton,
    Status,
    Switch,
    Tab,
    TabList,
    TabPanel,
    Table,
    TextBox,
    Timer,
    ToolBar,
    ToolTip,
    Tree,
    TreeGrid,
    TreeItem,
}

impl AriaRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Alert => "alert",
            Self::AlertDialog => "alertdialog",
            Self::Application => "application",
            Self::Article => "article",
            Self::Banner => "banner",
            Self::Button => "button",
            Self::Checkbox => "checkbox",
            Self::Complementary => "complementary",
            Self::ContentInfo => "contentinfo",
            Self::Dialog => "dialog",
            Self::Document => "document",
            Self::Feed => "feed",
            Self::Form => "form",
            Self::Grid => "grid",
            Self::GridCell => "gridcell",
            Self::Group => "group",
            Self::Heading => "heading",
            Self::Img => "img",
            Self::Link => "link",
            Self::List => "list",
            Self::ListBox => "listbox",
            Self::ListItem => "listitem",
            Self::Main => "main",
            Self::Menu => "menu",
            Self::MenuBar => "menubar",
            Self::MenuItem => "menuitem",
            Self::MenuItemCheckbox => "menuitemcheckbox",
            Self::MenuItemRadio => "menuitemradio",
            Self::Navigation => "navigation",
            Self::None => "none",
            Self::Note => "note",
            Self::Option => "option",
            Self::Presentation => "presentation",
            Self::ProgressBar => "progressbar",
            Self::Radio => "radio",
            Self::RadioGroup => "radiogroup",
            Self::Region => "region",
            Self::Row => "row",
            Self::RowGroup => "rowgroup",
            Self::Search => "search",
            Self::Separator => "separator",
            Self::Slider => "slider",
            Self::SpinButton => "spinbutton",
            Self::Status => "status",
            Self::Switch => "switch",
            Self::Tab => "tab",
            Self::TabList => "tablist",
            Self::TabPanel => "tabpanel",
            Self::Table => "table",
            Self::TextBox => "textbox",
            Self::Timer => "timer",
            Self::ToolBar => "toolbar",
            Self::ToolTip => "tooltip",
            Self::Tree => "tree",
            Self::TreeGrid => "treegrid",
            Self::TreeItem => "treeitem",
        }
    }
}

// ============================================================================
// LIVE REGION COMPONENT
// ============================================================================

/// Live region for announcing dynamic content to screen readers
#[component]
pub fn LiveRegion(
    /// Message to announce
    message: ReadSignal<String>,
    /// Politeness level
    #[prop(default = AriaLive::Polite)]
    politeness: AriaLive,
    /// Whether to announce the entire region or just changes
    #[prop(default = true)]
    atomic: bool,
    /// Additional CSS classes
    #[prop(optional, into)]
    class: Option<String>,
) -> impl IntoView {
    let class = class.unwrap_or_default();

    view! {
        <div
            role="status"
            aria-live=politeness.as_str()
            aria-atomic=if atomic { "true" } else { "false" }
            class=format!("sr-only {}", class)
        >
            {move || message.get()}
        </div>
    }
}

// ============================================================================
// SKIP LINK COMPONENT
// ============================================================================

/// Skip link for keyboard navigation
#[component]
pub fn SkipLink(
    /// Target element ID to skip to
    #[prop(into)]
    target: String,
    /// Link text
    #[prop(default = "Skip to main content".to_string(), into)]
    text: String,
) -> impl IntoView {
    view! {
        <a
            href=format!("#{}", target)
            class="sr-only focus:not-sr-only focus:absolute focus:top-4 focus:left-4 focus:z-50 focus:px-4 focus:py-2 focus:bg-primary-600 focus:text-white focus:rounded-md focus:shadow-lg"
        >
            {text}
        </a>
    }
}

// ============================================================================
// VISUALLY HIDDEN COMPONENT
// ============================================================================

/// Component that is visually hidden but accessible to screen readers
#[component]
pub fn VisuallyHidden(
    #[prop(optional, into)] class: Option<String>,
    children: Children,
) -> impl IntoView {
    let class = class.unwrap_or_default();

    view! {
        <span class=format!("sr-only {}", class)>
            {children()}
        </span>
    }
}

// ============================================================================
// FOCUS TRAP COMPONENT
// ============================================================================

/// Selector for all focusable elements
const FOCUSABLE_ELEMENTS_SELECTOR: &str = "button:not([disabled]), [href], input:not([disabled]):not([type=\"hidden\"]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex=\"-1\"])";

/// Focus trap for modals and dialogs
#[component]
pub fn FocusTrap(
    /// Whether the focus trap is active
    #[prop(into)]
    active: Signal<bool>,
    /// Additional CSS classes
    #[prop(optional, into)]
    class: Option<String>,
    children: Children,
) -> impl IntoView {
    let class = class.unwrap_or_default();
    let container_ref = NodeRef::<html::Div>::new();

    // Setup focus trap when active
    Effect::new(move |_| {
        if active.get()
            && let Some(container) = container_ref.get() {
                // Focus first focusable element
                if let Ok(focusable) = container.query_selector(FOCUSABLE_ELEMENTS_SELECTOR)
                    && let Some(element) = focusable {
                        let _ = element.dyn_ref::<web_sys::HtmlElement>().map(|el| el.focus());
                    }
            }
    });

    view! {
        <div
            node_ref=container_ref
            class=class
            on:keydown=move |ev: web_sys::KeyboardEvent| {
                if !active.get() {
                    return;
                }

                if ev.key() == "Tab" {
                    if let Some(container) = container_ref.get() {
                        // Get all focusable elements
                        if let Ok(elements) = container.query_selector_all(FOCUSABLE_ELEMENTS_SELECTOR) {
                            let length = elements.length();
                            if length == 0 {
                                return;
                            }

                            let first = elements.get(0);
                            let last = elements.get(length - 1);

                            if let Some(active_element) = web_sys::window()
                                .and_then(|w| w.document())
                                .and_then(|d| d.active_element())
                            {
                                if ev.shift_key() {
                                    // Shift+Tab: wrap to last element if current is first
                                    let is_first = first.as_ref().map_or(false, |n| active_element.is_same_node(Some(n)));
                                    if is_first {
                                        ev.prevent_default();
                                        if let Some(last_el) = last {
                                            let _ = last_el.dyn_ref::<web_sys::HtmlElement>().map(|el| el.focus());
                                        }
                                    }
                                } else {
                                    // Tab: wrap to first element if current is last
                                    let is_last = last.as_ref().map_or(false, |n| active_element.is_same_node(Some(n)));
                                    if is_last {
                                        ev.prevent_default();
                                        if let Some(first_el) = first {
                                            let _ = first_el.dyn_ref::<web_sys::HtmlElement>().map(|el| el.focus());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        >
            {children()}
        </div>
    }
}

// ============================================================================
// ACCESSIBILITY HELPERS
// ============================================================================

/// Generate a unique ID for accessibility attributes
pub fn generate_a11y_id(prefix: &str) -> String {
    use std::sync::atomic::{AtomicU32, Ordering};
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{}-{}", prefix, id)
}

/// Check if user prefers reduced motion
pub fn prefers_reduced_motion() -> bool {
    web_sys::window()
        .and_then(|w| w.match_media("(prefers-reduced-motion: reduce)").ok())
        .and_then(|mql| mql)
        .map(|mql| mql.matches())
        .unwrap_or(false)
}

/// Check if user prefers high contrast
pub fn prefers_high_contrast() -> bool {
    web_sys::window()
        .and_then(|w| w.match_media("(prefers-contrast: high)").ok())
        .and_then(|mql| mql)
        .map(|mql| mql.matches())
        .unwrap_or(false)
}

/// Check if user prefers dark color scheme
pub fn prefers_dark_scheme() -> bool {
    web_sys::window()
        .and_then(|w| w.match_media("(prefers-color-scheme: dark)").ok())
        .and_then(|mql| mql)
        .map(|mql| mql.matches())
        .unwrap_or(false)
}

// ============================================================================
// KEYBOARD NAVIGATION HELPERS
// ============================================================================

/// Common keyboard shortcuts
pub mod keyboard {
    pub const ENTER: &str = "Enter";
    pub const SPACE: &str = " ";
    pub const ESCAPE: &str = "Escape";
    pub const TAB: &str = "Tab";
    pub const ARROW_UP: &str = "ArrowUp";
    pub const ARROW_DOWN: &str = "ArrowDown";
    pub const ARROW_LEFT: &str = "ArrowLeft";
    pub const ARROW_RIGHT: &str = "ArrowRight";
    pub const HOME: &str = "Home";
    pub const END: &str = "End";
    pub const PAGE_UP: &str = "PageUp";
    pub const PAGE_DOWN: &str = "PageDown";
}

/// Check if key event is an activation key (Enter or Space)
pub fn is_activation_key(key: &str) -> bool {
    key == keyboard::ENTER || key == keyboard::SPACE
}

/// Check if key event is an arrow key
pub fn is_arrow_key(key: &str) -> bool {
    matches!(
        key,
        keyboard::ARROW_UP | keyboard::ARROW_DOWN | keyboard::ARROW_LEFT | keyboard::ARROW_RIGHT
    )
}
