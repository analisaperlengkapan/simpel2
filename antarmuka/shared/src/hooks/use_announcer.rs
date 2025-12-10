//! Screen reader announcement hook

use crate::utils::accessibility::AriaLive;
use leptos::prelude::*;
use std::time::Duration;

// ============================================================================
// ANNOUNCER HOOK
// ============================================================================

/// Hook for announcing messages to screen readers
/// # Example
/// ```rust
/// let announcer = use_announcer();
/// announcer.announce("Form submitted successfully", AriaLive::Polite);
/// ```
pub fn use_announcer() -> Announcer {
    let (message, set_message) = signal(String::new());
    let (politeness, set_politeness) = signal(AriaLive::Polite);

    Announcer {
        message,
        set_message,
        politeness,
        set_politeness,
    }
}

#[derive(Clone, Copy)]
pub struct Announcer {
    pub message: ReadSignal<String>,
    set_message: WriteSignal<String>,
    pub politeness: ReadSignal<AriaLive>,
    set_politeness: WriteSignal<AriaLive>,
}

impl Announcer {
    /// Announce a message to screen readers
    pub fn announce(&self, message: impl Into<String>, politeness: AriaLive) {
        let message = message.into();
        self.set_politeness.set(politeness);
        self.set_message.set(message.clone());

        // Clear message after a delay to allow re-announcement of same message
        let set_message = self.set_message;
        set_timeout(
            move || {
                set_message.set(String::new());
            },
            Duration::from_millis(100),
        );
    }

    /// Announce with polite politeness (default)
    pub fn announce_polite(&self, message: impl Into<String>) {
        self.announce(message, AriaLive::Polite);
    }

    /// Announce with assertive politeness (interrupts current announcement)
    pub fn announce_assertive(&self, message: impl Into<String>) {
        self.announce(message, AriaLive::Assertive);
    }

    /// Announce an error message
    pub fn announce_error(&self, message: impl Into<String>) {
        self.announce(format!("Error: {}", message.into()), AriaLive::Assertive);
    }

    /// Announce a success message
    pub fn announce_success(&self, message: impl Into<String>) {
        self.announce(format!("Success: {}", message.into()), AriaLive::Polite);
    }

    /// Announce a warning message
    pub fn announce_warning(&self, message: impl Into<String>) {
        self.announce(format!("Warning: {}", message.into()), AriaLive::Polite);
    }

    /// Announce loading state
    pub fn announce_loading(&self, message: impl Into<String>) {
        self.announce(format!("Loading: {}", message.into()), AriaLive::Polite);
    }

    /// Announce completion
    pub fn announce_complete(&self, message: impl Into<String>) {
        self.announce(format!("Complete: {}", message.into()), AriaLive::Polite);
    }
}

// ============================================================================
// ANNOUNCER COMPONENT
// ============================================================================

/// Component that renders the live region for announcements
#[component]
pub fn AnnouncerRegion(announcer: Announcer) -> impl IntoView {
    view! {
        <div
            role="status"
            aria-live=move || announcer.politeness.get().as_str()
            aria-atomic="true"
            class="sr-only"
        >
            {move || announcer.message.get()}
        </div>
    }
}

// ============================================================================
// ROUTE ANNOUNCER
// ============================================================================

/// Announce route changes to screen readers
pub fn use_route_announcer() {
    use leptos_router::hooks::use_location;

    let announcer = use_announcer();
    let location = use_location();

    Effect::new(move |_| {
        let pathname = location.pathname.get();

        // Extract page name from pathname
        let page_name = pathname
            .trim_start_matches('/')
            .split('/')
            .next()
            .unwrap_or("home");

        let formatted_name = page_name.replace('-', " ").replace('_', " ");

        announcer.announce_polite(format!("Navigated to {} page", formatted_name));
    });
}

// ============================================================================
// FORM VALIDATION ANNOUNCER
// ============================================================================

/// Announce form validation errors to screen readers
pub fn announce_form_errors(announcer: &Announcer, errors: &[(String, String)]) {
    if errors.is_empty() {
        return;
    }

    let error_count = errors.len();
    let error_message = if error_count == 1 {
        format!("1 error found: {}", errors[0].1)
    } else {
        let field_names: Vec<&str> = errors.iter().map(|(field, _)| field.as_str()).collect();
        format!(
            "{} errors found in fields: {}",
            error_count,
            field_names.join(", ")
        )
    };

    announcer.announce_error(error_message);
}

// ============================================================================
// DATA TABLE ANNOUNCER
// ============================================================================

/// Announce data table updates to screen readers
pub fn announce_table_update(
    announcer: &Announcer,
    total_rows: usize,
    visible_rows: usize,
    sort_column: Option<&str>,
    sort_direction: Option<&str>,
) {
    let mut message = format!(
        "Table updated. Showing {} of {} rows",
        visible_rows, total_rows
    );

    if let (Some(column), Some(direction)) = (sort_column, sort_direction) {
        message.push_str(&format!(". Sorted by {} {}", column, direction));
    }

    announcer.announce_polite(message);
}

// ============================================================================
// SEARCH RESULTS ANNOUNCER
// ============================================================================

/// Announce search results to screen readers
pub fn announce_search_results(announcer: &Announcer, query: &str, result_count: usize) {
    let message = if result_count == 0 {
        format!("No results found for '{}'", query)
    } else if result_count == 1 {
        format!("1 result found for '{}'", query)
    } else {
        format!("{} results found for '{}'", result_count, query)
    };

    announcer.announce_polite(message);
}

// ============================================================================
// MODAL ANNOUNCER
// ============================================================================

/// Announce modal open/close to screen readers
pub fn announce_modal(announcer: &Announcer, title: &str, is_open: bool) {
    if is_open {
        announcer.announce_polite(format!("{} dialog opened", title));
    } else {
        announcer.announce_polite(format!("{} dialog closed", title));
    }
}

// ============================================================================
// NOTIFICATION ANNOUNCER
// ============================================================================

/// Announce notifications to screen readers
pub fn announce_notification(announcer: &Announcer, notification_type: &str, message: &str) {
    let formatted_message = format!("{}: {}", notification_type, message);

    match notification_type.to_lowercase().as_str() {
        "error" | "danger" => announcer.announce_error(formatted_message),
        "warning" => announcer.announce_warning(formatted_message),
        "success" => announcer.announce_success(formatted_message),
        _ => announcer.announce_polite(formatted_message),
    }
}
