// Error tracking and reporting for SIMPEL
// Captures unhandled errors, provides stack traces, and sends reports with user context

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{ErrorEvent, PromiseRejectionEvent};

/// Error severity levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ErrorSeverity {
    Debug,
    Info,
    Warning,
    Error,
    Fatal,
}

impl ErrorSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Debug => "debug",
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
            Self::Fatal => "fatal",
        }
    }
}

/// Error report with full context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorReport {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub severity: ErrorSeverity,
    pub message: String,
    pub stack_trace: Option<String>,
    pub source: Option<String>,
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub user_context: UserContext,
    pub environment: EnvironmentContext,
    pub breadcrumbs: Vec<Breadcrumb>,
}

/// User context for error reports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserContext {
    pub user_id: Option<String>,
    pub username: Option<String>,
    pub email: Option<String>,
    pub role: Option<String>,
}

/// Environment context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentContext {
    pub page_url: String,
    pub user_agent: String,
    pub screen_resolution: String,
    pub viewport_size: String,
    pub locale: String,
    pub timezone: String,
}

/// Breadcrumb for tracking user actions leading to error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Breadcrumb {
    pub timestamp: DateTime<Utc>,
    pub category: String,
    pub message: String,
    pub level: ErrorSeverity,
    pub data: Option<serde_json::Value>,
}

/// Global error tracker state
static mut ERROR_TRACKER: Option<ErrorTracker> = None;

/// Error tracker instance
pub struct ErrorTracker {
    breadcrumbs: Vec<Breadcrumb>,
    max_breadcrumbs: usize,
    user_context: Option<UserContext>,
}

impl ErrorTracker {
    fn new() -> Self {
        Self {
            breadcrumbs: Vec::new(),
            max_breadcrumbs: 50,
            user_context: None,
        }
    }

    fn add_breadcrumb(&mut self, breadcrumb: Breadcrumb) {
        self.breadcrumbs.push(breadcrumb);

        // Keep only last N breadcrumbs
        if self.breadcrumbs.len() > self.max_breadcrumbs {
            self.breadcrumbs.remove(0);
        }
    }

    fn get_breadcrumbs(&self) -> Vec<Breadcrumb> {
        self.breadcrumbs.clone()
    }

    fn set_user_context(&mut self, context: UserContext) {
        self.user_context = Some(context);
    }

    fn get_user_context(&self) -> UserContext {
        self.user_context.clone().unwrap_or(UserContext {
            user_id: None,
            username: None,
            email: None,
            role: None,
        })
    }
}

/// Initialize error tracking
pub fn init_error_tracking() {
    // Initialize global tracker
    // nosemgrep: rust.lang.security.unsafe-usage.unsafe-usage
    unsafe {
        ERROR_TRACKER = Some(ErrorTracker::new());
    }

    // Set up panic hook
    setup_panic_hook();

    // Set up global error handler
    setup_error_handler();

    // Set up unhandled promise rejection handler
    setup_promise_rejection_handler();

    web_sys::console::log_1(&JsValue::from_str("[ErrorTracking] Initialized"));
}

/// Set up panic hook for Rust panics
fn setup_panic_hook() {
    std::panic::set_hook(Box::new(|panic_info| {
        let message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic".to_string()
        };

        let location = panic_info
            .location()
            .map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()));

        let error_report = ErrorReport {
            id: generate_error_id(),
            timestamp: Utc::now(),
            severity: ErrorSeverity::Fatal,
            message: format!("Rust Panic: {}", message),
            stack_trace: location.clone(),
            source: location,
            line: None,
            column: None,
            user_context: get_user_context(),
            environment: get_environment_context(),
            breadcrumbs: get_breadcrumbs(),
        };

        log_error(&error_report);
        send_error_report(error_report);
    }));
}

/// Set up global JavaScript error handler
fn setup_error_handler() {
    use web_sys::window;

    if let Some(window) = window() {
        let closure = Closure::wrap(Box::new(move |event: ErrorEvent| {
            let error_report = ErrorReport {
                id: generate_error_id(),
                timestamp: Utc::now(),
                severity: ErrorSeverity::Error,
                message: event.message(),
                stack_trace: event.error().as_string(),
                source: Some(event.filename()),
                line: Some(event.lineno()),
                column: Some(event.colno()),
                user_context: get_user_context(),
                environment: get_environment_context(),
                breadcrumbs: get_breadcrumbs(),
            };

            log_error(&error_report);
            send_error_report(error_report);
        }) as Box<dyn FnMut(ErrorEvent)>);

        window.set_onerror(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }
}

/// Set up unhandled promise rejection handler
fn setup_promise_rejection_handler() {
    use web_sys::window;

    if let Some(window) = window() {
        let closure = Closure::wrap(Box::new(move |event: PromiseRejectionEvent| {
            let reason = event.reason();
            let message = if let Some(s) = reason.as_string() {
                s
            } else {
                format!("{:?}", reason)
            };

            let error_report = ErrorReport {
                id: generate_error_id(),
                timestamp: Utc::now(),
                severity: ErrorSeverity::Error,
                message: format!("Unhandled Promise Rejection: {}", message),
                stack_trace: None,
                source: None,
                line: None,
                column: None,
                user_context: get_user_context(),
                environment: get_environment_context(),
                breadcrumbs: get_breadcrumbs(),
            };

            log_error(&error_report);
            send_error_report(error_report);
        }) as Box<dyn FnMut(PromiseRejectionEvent)>);

        let _ = window.add_event_listener_with_callback(
            "unhandledrejection",
            closure.as_ref().unchecked_ref(),
        );
        closure.forget();
    }
}

/// Add breadcrumb to track user actions
pub fn add_breadcrumb(
    category: &str,
    message: &str,
    level: ErrorSeverity,
    data: Option<serde_json::Value>,
) {
    let breadcrumb = Breadcrumb {
        timestamp: Utc::now(),
        category: category.to_string(),
        message: message.to_string(),
        level,
        data,
    };

    // nosemgrep: rust.lang.security.unsafe-usage.unsafe-usage
    unsafe {
        if let Some(tracker) = (*std::ptr::addr_of_mut!(ERROR_TRACKER)).as_mut() {
            tracker.add_breadcrumb(breadcrumb);
        }
    }
}

/// Set user context for error reports
pub fn set_user_context(
    user_id: Option<String>,
    username: Option<String>,
    email: Option<String>,
    role: Option<String>,
) {
    let context = UserContext {
        user_id,
        username,
        email,
        role,
    };

    // nosemgrep: rust.lang.security.unsafe-usage.unsafe-usage
    unsafe {
        if let Some(tracker) = (*std::ptr::addr_of_mut!(ERROR_TRACKER)).as_mut() {
            tracker.set_user_context(context);
        }
    }
}

/// Get user context
fn get_user_context() -> UserContext {
    // nosemgrep: rust.lang.security.unsafe-usage.unsafe-usage
    unsafe {
        (*std::ptr::addr_of!(ERROR_TRACKER))
            .as_ref()
            .map(|t| t.get_user_context())
            .unwrap_or(UserContext {
                user_id: None,
                username: None,
                email: None,
                role: None,
            })
    }
}

/// Get breadcrumbs
fn get_breadcrumbs() -> Vec<Breadcrumb> {
    // nosemgrep: rust.lang.security.unsafe-usage.unsafe-usage
    unsafe {
        (*std::ptr::addr_of!(ERROR_TRACKER))
            .as_ref()
            .map(|t| t.get_breadcrumbs())
            .unwrap_or_default()
    }
}

/// Get environment context
fn get_environment_context() -> EnvironmentContext {
    use web_sys::window;

    let window = window().expect("No window object");
    let navigator = window.navigator();
    let screen = window.screen().ok();

    EnvironmentContext {
        page_url: window.location().href().unwrap_or_default(),
        user_agent: navigator.user_agent().unwrap_or_default(),
        screen_resolution: screen
            .as_ref()
            .map(|s| format!("{}x{}", s.width().unwrap_or(0), s.height().unwrap_or(0)))
            .unwrap_or_default(),
        viewport_size: format!(
            "{}x{}",
            window.inner_width().unwrap().as_f64().unwrap_or(0.0),
            window.inner_height().unwrap().as_f64().unwrap_or(0.0)
        ),
        locale: navigator.language().unwrap_or_default(),
        timezone: get_timezone(),
    }
}

/// Get timezone
fn get_timezone() -> String {
    use js_sys::Date;

    let date = Date::new_0();
    date.to_locale_time_string("en-US")
        .as_string()
        .unwrap_or_default()
}

/// Generate unique error ID
fn generate_error_id() -> String {
    use uuid::Uuid;
    Uuid::new_v4().to_string()
}

/// Log error to console
fn log_error(error: &ErrorReport) {
    let severity_color = match error.severity {
        ErrorSeverity::Debug => "color: gray",
        ErrorSeverity::Info => "color: blue",
        ErrorSeverity::Warning => "color: orange",
        ErrorSeverity::Error => "color: red",
        ErrorSeverity::Fatal => "color: darkred; font-weight: bold",
    };

    web_sys::console::error_1(&JsValue::from_str(&format!(
        "[{}] {} - {}\nID: {}\nStack: {:?}",
        error.severity.as_str().to_uppercase(),
        error.timestamp.format("%Y-%m-%d %H:%M:%S"),
        error.message,
        error.id,
        error.stack_trace
    )));

    // Log with styling
    web_sys::console::log_2(
        &JsValue::from_str(&format!("%c{}", error.message)),
        &JsValue::from_str(severity_color),
    );
}

/// Send error report to monitoring service
fn send_error_report(error: ErrorReport) {
    #[cfg(not(debug_assertions))]
    {
        use wasm_bindgen_futures::spawn_local;
        spawn_local(async move {
            let _ = send_error_to_service(error).await;
        });
    }

    #[cfg(debug_assertions)]
    {
        // In development, just log to console
        web_sys::console::log_1(&JsValue::from_str(&format!(
            "[ErrorTracking] Would send error report: {}",
            serde_json::to_string_pretty(&error).unwrap_or_default()
        )));
    }
}

/// Send error to monitoring service
#[cfg(not(debug_assertions))]
async fn send_error_to_service(error: ErrorReport) -> Result<(), JsValue> {
    use gloo::net::http::Request;

    let _ = Request::post("/api/monitoring/errors")
        .json(&error)
        .map_err(|e| JsValue::from_str(&e.to_string()))?
        .send()
        .await;

    Ok(())
}

/// Manually capture and report an error
pub fn capture_error(message: &str, severity: ErrorSeverity, stack_trace: Option<String>) {
    let error_report = ErrorReport {
        id: generate_error_id(),
        timestamp: Utc::now(),
        severity,
        message: message.to_string(),
        stack_trace,
        source: None,
        line: None,
        column: None,
        user_context: get_user_context(),
        environment: get_environment_context(),
        breadcrumbs: get_breadcrumbs(),
    };

    log_error(&error_report);
    send_error_report(error_report);
}

/// Capture exception with context
pub fn capture_exception(error: &dyn std::error::Error, severity: ErrorSeverity) {
    let message = error.to_string();
    let stack_trace = format!("{:?}", error);

    capture_error(&message, severity, Some(stack_trace));
}
