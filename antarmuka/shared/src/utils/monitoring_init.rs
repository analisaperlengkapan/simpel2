// Monitoring initialization module
// Provides a single entry point to initialize all monitoring systems

use crate::utils::{
    analytics::{init_analytics, set_user_id as set_analytics_user_id},
    error_tracking::{init_error_tracking, set_user_context},
    monitoring::init_performance_monitoring,
};

/// Initialize all monitoring systems
pub fn init_monitoring() {
    // Initialize performance monitoring
    init_performance_monitoring();

    // Initialize error tracking
    init_error_tracking();

    // Initialize analytics
    init_analytics();

    web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(
        "[Monitoring] All monitoring systems initialized",
    ));
}

/// Set user context for all monitoring systems
pub fn set_monitoring_user_context(
    user_id: String,
    username: Option<String>,
    email: Option<String>,
    role: Option<String>,
) {
    // Set user ID for analytics
    set_analytics_user_id(user_id.clone());

    // Set user context for error tracking
    set_user_context(Some(user_id), username, email, role);

    web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(
        "[Monitoring] User context updated",
    ));
}

/// Clear user context (on logout)
pub fn clear_monitoring_user_context() {
    set_user_context(None, None, None, None);

    web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(
        "[Monitoring] User context cleared",
    ));
}
