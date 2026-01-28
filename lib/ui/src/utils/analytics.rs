// User analytics and event tracking for SIMPelv2
// Tracks page views, user interactions, feature usage, and conversion funnels

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

/// User event types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EventType {
    PageView,
    Click,
    FormSubmit,
    Search,
    Download,
    Upload,
    Login,
    Logout,
    FeatureUsage,
    Error,
    Custom(String),
}

impl EventType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::PageView => "page_view",
            Self::Click => "click",
            Self::FormSubmit => "form_submit",
            Self::Search => "search",
            Self::Download => "download",
            Self::Upload => "upload",
            Self::Login => "login",
            Self::Logout => "logout",
            Self::FeatureUsage => "feature_usage",
            Self::Error => "error",
            Self::Custom(name) => name,
        }
    }
}

/// User event for analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserEvent {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub event_type: EventType,
    pub page: String,
    pub user_id: Option<String>,
    pub session_id: String,
    pub properties: HashMap<String, serde_json::Value>,
}

/// Page view event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageView {
    pub page_url: String,
    pub page_title: String,
    pub referrer: Option<String>,
    pub duration_ms: Option<u64>,
}

/// Click event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickEvent {
    pub element_id: Option<String>,
    pub element_class: Option<String>,
    pub element_text: Option<String>,
    pub element_type: String,
    pub x: i32,
    pub y: i32,
}

/// Form submission event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormSubmitEvent {
    pub form_id: Option<String>,
    pub form_name: Option<String>,
    pub fields: Vec<String>,
    pub success: bool,
}

/// Search event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchEvent {
    pub query: String,
    pub results_count: usize,
    pub filters: HashMap<String, String>,
}

/// Feature usage event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureUsageEvent {
    pub feature_name: String,
    pub action: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Conversion funnel step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunnelStep {
    pub funnel_name: String,
    pub step_name: String,
    pub step_number: u32,
    pub completed: bool,
}

/// Analytics tracker
pub struct Analytics {
    session_id: String,
    user_id: Option<String>,
    page_start_time: Option<DateTime<Utc>>,
}

static mut ANALYTICS: Option<Analytics> = None;

impl Analytics {
    fn new() -> Self {
        Self {
            session_id: generate_session_id(),
            user_id: None,
            page_start_time: None,
        }
    }

    fn set_user_id(&mut self, user_id: String) {
        self.user_id = Some(user_id);
    }

    fn start_page_view(&mut self) {
        self.page_start_time = Some(Utc::now());
    }

    fn get_page_duration(&self) -> Option<u64> {
        self.page_start_time.map(|start| {
            let duration = Utc::now().signed_duration_since(start);
            duration.num_milliseconds() as u64
        })
    }
}

/// Initialize analytics tracking
pub fn init_analytics() {
    // nosemgrep: rust.lang.security.unsafe-usage.unsafe-usage
    unsafe {
        ANALYTICS = Some(Analytics::new());
    }

    // Track initial page view
    track_page_view();

    // Set up automatic page view tracking on navigation
    setup_navigation_tracking();

    web_sys::console::log_1(&JsValue::from_str("[Analytics] Initialized"));
}

/// Set user ID for analytics
pub fn set_user_id(user_id: String) {
    // nosemgrep: rust.lang.security.unsafe-usage.unsafe-usage
    unsafe {
        if let Some(analytics) = (*std::ptr::addr_of_mut!(ANALYTICS)).as_mut() {
            analytics.set_user_id(user_id);
        }
    }
}

/// Track page view
pub fn track_page_view() {
    use web_sys::window;

    let window = window().expect("No window object");
    let location = window.location();
    let document = window.document().expect("No document");

    // Get page duration if this is not the first page
    // nosemgrep: rust.lang.security.unsafe-usage.unsafe-usage
    let duration_ms = unsafe {
        (*std::ptr::addr_of!(ANALYTICS))
            .as_ref()
            .and_then(|a| a.get_page_duration())
    };

    // Start timing for this page
    // nosemgrep: rust.lang.security.unsafe-usage.unsafe-usage
    unsafe {
        if let Some(analytics) = (*std::ptr::addr_of_mut!(ANALYTICS)).as_mut() {
            analytics.start_page_view();
        }
    }

    let page_view = PageView {
        page_url: location.href().unwrap_or_default(),
        page_title: document.title(),
        referrer: document.referrer().into(),
        duration_ms,
    };

    let mut properties = HashMap::new();
    properties.insert("url".to_string(), serde_json::json!(page_view.page_url));
    properties.insert("title".to_string(), serde_json::json!(page_view.page_title));
    if let Some(ref referrer) = page_view.referrer {
        properties.insert("referrer".to_string(), serde_json::json!(referrer));
    }
    if let Some(duration) = page_view.duration_ms {
        properties.insert("duration_ms".to_string(), serde_json::json!(duration));
    }

    track_event(EventType::PageView, properties);
}

/// Track click event
pub fn track_click(
    element_id: Option<String>,
    element_class: Option<String>,
    element_text: Option<String>,
    element_type: String,
    x: i32,
    y: i32,
) {
    let click_event = ClickEvent {
        element_id,
        element_class,
        element_text,
        element_type,
        x,
        y,
    };

    let mut properties = HashMap::new();
    if let Some(id) = click_event.element_id {
        properties.insert("element_id".to_string(), serde_json::json!(id));
    }
    if let Some(class) = click_event.element_class {
        properties.insert("element_class".to_string(), serde_json::json!(class));
    }
    if let Some(text) = click_event.element_text {
        properties.insert("element_text".to_string(), serde_json::json!(text));
    }
    properties.insert(
        "element_type".to_string(),
        serde_json::json!(click_event.element_type),
    );
    properties.insert("x".to_string(), serde_json::json!(click_event.x));
    properties.insert("y".to_string(), serde_json::json!(click_event.y));

    track_event(EventType::Click, properties);
}

/// Track form submission
pub fn track_form_submit(
    form_id: Option<String>,
    form_name: Option<String>,
    fields: Vec<String>,
    success: bool,
) {
    let form_event = FormSubmitEvent {
        form_id,
        form_name,
        fields,
        success,
    };

    let mut properties = HashMap::new();
    if let Some(id) = form_event.form_id {
        properties.insert("form_id".to_string(), serde_json::json!(id));
    }
    if let Some(name) = form_event.form_name {
        properties.insert("form_name".to_string(), serde_json::json!(name));
    }
    properties.insert("fields".to_string(), serde_json::json!(form_event.fields));
    properties.insert("success".to_string(), serde_json::json!(form_event.success));

    track_event(EventType::FormSubmit, properties);
}

/// Track search
pub fn track_search(query: String, results_count: usize, filters: HashMap<String, String>) {
    let search_event = SearchEvent {
        query,
        results_count,
        filters,
    };

    let mut properties = HashMap::new();
    properties.insert("query".to_string(), serde_json::json!(search_event.query));
    properties.insert(
        "results_count".to_string(),
        serde_json::json!(search_event.results_count),
    );
    properties.insert(
        "filters".to_string(),
        serde_json::json!(search_event.filters),
    );

    track_event(EventType::Search, properties);
}

/// Track feature usage
pub fn track_feature_usage(
    feature_name: String,
    action: String,
    metadata: HashMap<String, serde_json::Value>,
) {
    let feature_event = FeatureUsageEvent {
        feature_name,
        action,
        metadata,
    };

    let mut properties = HashMap::new();
    properties.insert(
        "feature_name".to_string(),
        serde_json::json!(feature_event.feature_name),
    );
    properties.insert(
        "action".to_string(),
        serde_json::json!(feature_event.action),
    );
    properties.insert(
        "metadata".to_string(),
        serde_json::json!(feature_event.metadata),
    );

    track_event(EventType::FeatureUsage, properties);
}

/// Track conversion funnel step
pub fn track_funnel_step(
    funnel_name: String,
    step_name: String,
    step_number: u32,
    completed: bool,
) {
    let funnel_step = FunnelStep {
        funnel_name,
        step_name,
        step_number,
        completed,
    };

    let mut properties = HashMap::new();
    properties.insert(
        "funnel_name".to_string(),
        serde_json::json!(funnel_step.funnel_name),
    );
    properties.insert(
        "step_name".to_string(),
        serde_json::json!(funnel_step.step_name),
    );
    properties.insert(
        "step_number".to_string(),
        serde_json::json!(funnel_step.step_number),
    );
    properties.insert(
        "completed".to_string(),
        serde_json::json!(funnel_step.completed),
    );

    track_event(EventType::Custom("funnel_step".to_string()), properties);
}

/// Track generic event
pub fn track_event(event_type: EventType, properties: HashMap<String, serde_json::Value>) {
    use web_sys::window;

    let window = window().expect("No window object");
    let page = window.location().href().unwrap_or_default();

    // nosemgrep: rust.lang.security.unsafe-usage.unsafe-usage
    let (session_id, user_id) = unsafe {
        (*std::ptr::addr_of!(ANALYTICS))
            .as_ref()
            .map(|a| (a.session_id.clone(), a.user_id.clone()))
            .unwrap_or_else(|| (generate_session_id(), None))
    };

    let event = UserEvent {
        id: generate_event_id(),
        timestamp: Utc::now(),
        event_type: event_type.clone(),
        page,
        user_id,
        session_id,
        properties,
    };

    log_event(&event);
    send_event(event);
}

/// Set up automatic navigation tracking
fn setup_navigation_tracking() {
    use web_sys::window;

    if let Some(window) = window() {
        // Track page visibility changes
        let closure = Closure::wrap(Box::new(move || {
            track_page_view();
        }) as Box<dyn FnMut()>);

        let _ =
            window.add_event_listener_with_callback("popstate", closure.as_ref().unchecked_ref());
        closure.forget();
    }
}

/// Generate session ID
fn generate_session_id() -> String {
    use uuid::Uuid;
    Uuid::new_v4().to_string()
}

/// Generate event ID
fn generate_event_id() -> String {
    use uuid::Uuid;
    Uuid::new_v4().to_string()
}

/// Log event to console
fn log_event(event: &UserEvent) {
    #[cfg(debug_assertions)]
    {
        web_sys::console::log_1(&JsValue::from_str(&format!(
            "[Analytics] {} - {} (User: {:?})",
            event.event_type.as_str(),
            event.page,
            event.user_id
        )));
    }
}

/// Send event to analytics service
fn send_event(event: UserEvent) {
    #[cfg(not(debug_assertions))]
    {
        use wasm_bindgen_futures::spawn_local;
        spawn_local(async move {
            let _ = send_event_to_service(event).await;
        });
    }

    #[cfg(debug_assertions)]
    {
        // In development, just log to console
        web_sys::console::log_1(&JsValue::from_str(&format!(
            "[Analytics] Would send event: {}",
            serde_json::to_string_pretty(&event).unwrap_or_default()
        )));
    }
}

/// Send event to analytics service
#[cfg(not(debug_assertions))]
async fn send_event_to_service(event: UserEvent) -> Result<(), JsValue> {
    use gloo::net::http::Request;

    let _ = Request::post("/api/analytics/events")
        .json(&event)
        .map_err(|e| JsValue::from_str(&e.to_string()))?
        .send()
        .await;

    Ok(())
}

/// Track user session duration
pub fn track_session_duration() {
    // nosemgrep: rust.lang.security.unsafe-usage.unsafe-usage
    unsafe {
        if let Some(analytics) = (*std::ptr::addr_of!(ANALYTICS)).as_ref()
            && let Some(duration) = analytics.get_page_duration()
        {
            let mut properties = HashMap::new();
            properties.insert("duration_ms".to_string(), serde_json::json!(duration));
            track_event(
                EventType::Custom("session_duration".to_string()),
                properties,
            );
        }
    }
}

/// Track user engagement score
pub fn track_engagement(score: f64, interactions: u32) {
    let mut properties = HashMap::new();
    properties.insert("score".to_string(), serde_json::json!(score));
    properties.insert("interactions".to_string(), serde_json::json!(interactions));
    track_event(EventType::Custom("engagement".to_string()), properties);
}
