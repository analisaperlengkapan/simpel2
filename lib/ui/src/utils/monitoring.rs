// Performance monitoring utilities for SIMPelv2
// Tracks Core Web Vitals, WASM load times, and API response times

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use web_sys::Performance;

/// Core Web Vitals metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreWebVitals {
    /// Largest Contentful Paint (LCP) - should be < 2.5s
    pub lcp: Option<f64>,
    /// First Input Delay (FID) - should be < 100ms
    pub fid: Option<f64>,
    /// Cumulative Layout Shift (CLS) - should be < 0.1
    pub cls: Option<f64>,
    /// First Contentful Paint (FCP) - should be < 1.8s
    pub fcp: Option<f64>,
    /// Time to First Byte (TTFB) - should be < 600ms
    pub ttfb: Option<f64>,
}

/// Performance metrics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub timestamp: DateTime<Utc>,
    pub page_url: String,
    pub user_agent: String,
    pub core_web_vitals: CoreWebVitals,
    pub wasm_load_time: Option<f64>,
    pub dom_content_loaded: Option<f64>,
    pub load_complete: Option<f64>,
}

/// API request metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiMetrics {
    pub timestamp: DateTime<Utc>,
    pub endpoint: String,
    pub method: String,
    pub status_code: u16,
    pub response_time_ms: f64,
    pub success: bool,
}

/// Initialize performance monitoring
pub fn init_performance_monitoring() {
    // Track navigation timing when page loads
    track_navigation_timing();

    web_sys::console::log_1(&JsValue::from_str("[Performance] Monitoring initialized"));
}

/// Track navigation timing metrics
fn track_navigation_timing() {
    use web_sys::window;

    if let Some(win) = window() {
        // Wait for page load to complete
        let closure = Closure::wrap(Box::new(move || {
            if let Some(window) = window()
                && let Some(performance) = window.performance()
            {
                collect_navigation_metrics(&performance);
            }
        }) as Box<dyn FnMut()>);

        win.set_onload(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }
}

/// Collect navigation timing metrics
fn collect_navigation_metrics(performance: &Performance) {
    use js_sys::Reflect;

    // Get timing object
    if let Ok(timing) = Reflect::get(performance, &JsValue::from_str("timing")) {
        // Calculate key metrics
        let navigation_start = get_timing_value(&timing, "navigationStart");
        let response_start = get_timing_value(&timing, "responseStart");
        let dom_content_loaded = get_timing_value(&timing, "domContentLoadedEventEnd");
        let load_complete = get_timing_value(&timing, "loadEventEnd");

        // Calculate TTFB (Time to First Byte)
        if let (Some(nav_start), Some(resp_start)) = (navigation_start, response_start) {
            let ttfb = resp_start - nav_start;
            log_metric("TTFB", ttfb);

            if ttfb > 600.0 {
                send_performance_alert("TTFB", ttfb, 600.0);
            }
        }

        // Calculate DOM Content Loaded time
        if let (Some(nav_start), Some(dcl)) = (navigation_start, dom_content_loaded) {
            let dcl_time = dcl - nav_start;
            log_metric("DOMContentLoaded", dcl_time);
        }

        // Calculate full page load time
        if let (Some(nav_start), Some(load)) = (navigation_start, load_complete) {
            let load_time = load - nav_start;
            log_metric("LoadComplete", load_time);
        }
    }

    // Track WASM load time
    track_wasm_load_time(performance);
}

/// Get timing value from timing object
fn get_timing_value(timing: &JsValue, property: &str) -> Option<f64> {
    use js_sys::Reflect;

    if let Ok(value) = Reflect::get(timing, &JsValue::from_str(property)) {
        value.as_f64()
    } else {
        None
    }
}

/// Track WASM module load time
fn track_wasm_load_time(performance: &Performance) {
    // Get all resource timing entries using getEntriesByType
    let entries_result = js_sys::Reflect::get(performance, &JsValue::from_str("getEntriesByType"));

    if let Ok(get_entries_fn) = entries_result
        && let Ok(entries) = js_sys::Reflect::apply(
            &get_entries_fn.dyn_into::<js_sys::Function>().unwrap(),
            performance,
            &js_sys::Array::of1(&JsValue::from_str("resource")),
        )
        && let Ok(entries_array) = entries.dyn_into::<js_sys::Array>()
    {
        for i in 0..entries_array.length() {
            if let Ok(entry) = entries_array.get(i).dyn_into::<js_sys::Object>()
                && let Ok(name) = js_sys::Reflect::get(&entry, &JsValue::from_str("name"))
                && let Some(name_str) = name.as_string()
            {
                // Check if this is a WASM file
                if name_str.ends_with(".wasm")
                    && let Ok(duration) =
                        js_sys::Reflect::get(&entry, &JsValue::from_str("duration"))
                    && let Some(load_time) = duration.as_f64()
                {
                    log_metric(&format!("WASM Load: {}", name_str), load_time);

                    // Alert if WASM takes > 3s to load
                    if load_time > 3000.0 {
                        send_performance_alert("WASM Load", load_time, 3000.0);
                    }
                }
            }
        }
    }
}

/// Log metric to console (development) and send to monitoring service (production)
fn log_metric(name: &str, value: f64) {
    web_sys::console::log_2(
        &JsValue::from_str(&format!("[Performance] {}", name)),
        &JsValue::from_f64(value),
    );

    // In production, send to monitoring service
    #[cfg(not(debug_assertions))]
    {
        use wasm_bindgen_futures::spawn_local;
        let name = name.to_string();
        spawn_local(async move {
            let _ = send_metric_to_service(&name, value).await;
        });
    }
}

/// Send performance alert for poor metrics
fn send_performance_alert(metric: &str, actual: f64, threshold: f64) {
    web_sys::console::warn_2(
        &JsValue::from_str(&format!(
            "[Performance Alert] {} exceeded threshold",
            metric
        )),
        &JsValue::from_str(&format!(
            "Actual: {:.2}ms, Threshold: {:.2}ms",
            actual, threshold
        )),
    );

    #[cfg(not(debug_assertions))]
    {
        use wasm_bindgen_futures::spawn_local;
        let metric = metric.to_string();
        spawn_local(async move {
            let _ = send_alert_to_service(&metric, actual, threshold).await;
        });
    }
}

/// Send metric to monitoring service
#[cfg(not(debug_assertions))]
async fn send_metric_to_service(name: &str, value: f64) -> Result<(), JsValue> {
    use gloo::net::http::Request;

    let metric = serde_json::json!({
        "name": name,
        "value": value,
        "timestamp": Utc::now().to_rfc3339(),
        "page": web_sys::window().unwrap().location().href().unwrap(),
    });

    let _ = Request::post("/api/monitoring/metrics")
        .json(&metric)
        .map_err(|e| JsValue::from_str(&e.to_string()))?
        .send()
        .await;

    Ok(())
}

/// Send alert to monitoring service
#[cfg(not(debug_assertions))]
async fn send_alert_to_service(metric: &str, actual: f64, threshold: f64) -> Result<(), JsValue> {
    use gloo::net::http::Request;

    let alert = serde_json::json!({
        "metric": metric,
        "actual": actual,
        "threshold": threshold,
        "severity": "warning",
        "timestamp": Utc::now().to_rfc3339(),
        "page": web_sys::window().unwrap().location().href().unwrap(),
    });

    let _ = Request::post("/api/monitoring/alerts")
        .json(&alert)
        .map_err(|e| JsValue::from_str(&e.to_string()))?
        .send()
        .await;

    Ok(())
}

/// Track API request performance
pub fn track_api_request(endpoint: &str, method: &str, status_code: u16, response_time_ms: f64) {
    let _metrics = ApiMetrics {
        timestamp: Utc::now(),
        endpoint: endpoint.to_string(),
        method: method.to_string(),
        status_code,
        response_time_ms,
        success: (200..300).contains(&status_code),
    };

    web_sys::console::log_2(
        &JsValue::from_str(&format!("[API] {} {}", method, endpoint)),
        &JsValue::from_str(&format!("{}ms ({})", response_time_ms, status_code)),
    );

    // Alert on slow API responses (> 5s)
    if response_time_ms > 5000.0 {
        web_sys::console::warn_1(&JsValue::from_str(&format!(
            "[API Alert] Slow response: {} {} took {:.2}ms",
            method, endpoint, response_time_ms
        )));
    }

    #[cfg(not(debug_assertions))]
    {
        use wasm_bindgen_futures::spawn_local;
        spawn_local(async move {
            let _ = send_api_metrics(_metrics).await;
        });
    }
}

/// Send API metrics to monitoring service
#[cfg(not(debug_assertions))]
async fn send_api_metrics(metrics: ApiMetrics) -> Result<(), JsValue> {
    use gloo::net::http::Request;

    let _ = Request::post("/api/monitoring/api-metrics")
        .json(&metrics)
        .map_err(|e| JsValue::from_str(&e.to_string()))?
        .send()
        .await;

    Ok(())
}

/// Get current performance metrics snapshot
pub fn get_performance_snapshot() -> PerformanceMetrics {
    use web_sys::window;

    let window = window().expect("No window object");

    PerformanceMetrics {
        timestamp: Utc::now(),
        page_url: window.location().href().unwrap_or_default(),
        user_agent: window.navigator().user_agent().unwrap_or_default(),
        core_web_vitals: CoreWebVitals {
            lcp: None,
            fid: None,
            cls: None,
            fcp: None,
            ttfb: None,
        },
        wasm_load_time: None,
        dom_content_loaded: None,
        load_complete: None,
    }
}
