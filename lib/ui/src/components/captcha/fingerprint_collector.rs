//! Client-side Fingerprint Collection
//!
//! Privacy-compliant browser fingerprinting for bot detection

use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{Navigator, Window};

/// Client-side fingerprint data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientFingerprintData {
    /// Basic browser information
    pub user_agent: String,
    pub language: String,
    pub languages: Vec<String>,
    pub platform: String,
    pub cookie_enabled: bool,
    pub do_not_track: Option<String>,

    /// Screen and display information
    pub screen_width: u32,
    pub screen_height: u32,
    pub screen_color_depth: u32,
    pub screen_pixel_depth: u32,
    pub available_width: u32,
    pub available_height: u32,
    pub inner_width: u32,
    pub inner_height: u32,
    pub outer_width: u32,
    pub outer_height: u32,
    pub device_pixel_ratio: f64,

    /// Hardware information
    pub hardware_concurrency: Option<u32>,
    pub device_memory: Option<f64>,
    pub max_touch_points: Option<u32>,

    /// Timezone and locale
    pub timezone_offset: i32,
    pub timezone: String,

    /// Canvas fingerprint
    pub canvas_fingerprint: Option<String>,

    /// WebGL fingerprint
    pub webgl_fingerprint: Option<String>,

    /// Audio context fingerprint
    pub audio_fingerprint: Option<String>,

    /// Font detection
    pub available_fonts: Vec<String>,

    /// Network information
    pub connection_type: Option<String>,
    pub connection_downlink: Option<f64>,
    pub connection_rtt: Option<u32>,
    pub connection_save_data: Option<bool>,

    /// Performance timing
    pub performance_timing: Option<PerformanceTiming>,

    /// Media devices
    pub media_devices_count: Option<u32>,

    /// Battery information (if available)
    pub battery_level: Option<f64>,
    pub battery_charging: Option<bool>,

    /// WebRTC information
    pub webrtc_fingerprint: Option<String>,

    /// Additional entropy sources
    pub entropy_sources: HashMap<String, String>,
}

/// Performance timing data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTiming {
    pub navigation_start: f64,
    pub dom_content_loaded: f64,
    pub load_complete: f64,
    pub dns_lookup_time: f64,
    pub tcp_connect_time: f64,
}

/// Fingerprint collection configuration
#[derive(Debug, Clone)]
pub struct FingerprintCollectionConfig {
    /// Enable canvas fingerprinting
    pub enable_canvas: bool,

    /// Enable WebGL fingerprinting
    pub enable_webgl: bool,

    /// Enable audio fingerprinting
    pub enable_audio: bool,

    /// Enable font detection
    pub enable_fonts: bool,

    /// Enable WebRTC fingerprinting
    pub enable_webrtc: bool,

    /// Enable performance timing
    pub enable_performance: bool,

    /// Privacy compliance mode
    pub privacy_mode: bool,

    /// Timeout for async operations (ms)
    pub timeout_ms: u32,
}

impl Default for FingerprintCollectionConfig {
    fn default() -> Self {
        Self {
            enable_canvas: true,
            enable_webgl: true,
            enable_audio: false, // Disabled by default for privacy
            enable_fonts: true,
            enable_webrtc: false, // Disabled by default for privacy
            enable_performance: true,
            privacy_mode: true,
            timeout_ms: 5000,
        }
    }
}

/// Fingerprint collector component
#[component]
pub fn FingerprintCollector(
    /// Configuration for fingerprint collection
    #[prop(optional)]
    config: Option<FingerprintCollectionConfig>,
    /// Callback when fingerprint is collected
    #[prop(optional)]
    on_fingerprint_collected: Option<Callback<ClientFingerprintData>>,
) -> impl IntoView {
    let config = config.unwrap_or_default();
    let (_fingerprint_data, set_fingerprint_data) = signal(None::<ClientFingerprintData>);

    // Collect fingerprint on mount
    Effect::new(move |_| {
        let config_clone = config.clone();
        spawn_local(async move {
            if let Ok(fingerprint) = collect_fingerprint(&config_clone).await {
                set_fingerprint_data.set(Some(fingerprint.clone()));

                if let Some(_callback) = on_fingerprint_collected {
                    // callback(fingerprint);
                }
            }
        });
    });

    view! {
        <div
            class="fingerprint-collector"
            style="display: none;"
        >// Hidden component for fingerprint collection
        </div>
    }
}

/// Collect comprehensive browser fingerprint
async fn collect_fingerprint(
    config: &FingerprintCollectionConfig,
) -> Result<ClientFingerprintData, JsValue> {
    let window = web_sys::window().ok_or("No window available")?;
    let navigator = window.navigator();
    let screen = window.screen().map_err(|_| "No screen available")?;

    // Basic browser information
    let user_agent = navigator.user_agent().unwrap_or_default();
    let language = navigator.language().unwrap_or_default();
    let languages = get_languages(&navigator);
    let platform = navigator.platform().unwrap_or_default();
    let cookie_enabled = true; // Simplified for compatibility
    let do_not_track = navigator.do_not_track();

    // Screen information
    let screen_width = screen.width().unwrap_or(0) as u32;
    let screen_height = screen.height().unwrap_or(0) as u32;
    let screen_color_depth = screen.color_depth().unwrap_or(0) as u32;
    let screen_pixel_depth = screen.pixel_depth().unwrap_or(0) as u32;
    let available_width = screen.avail_width().unwrap_or(0) as u32;
    let available_height = screen.avail_height().unwrap_or(0) as u32;

    // Window dimensions
    let inner_width = window
        .inner_width()
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0) as u32;
    let inner_height = window
        .inner_height()
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0) as u32;
    let outer_width = window
        .outer_width()
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0) as u32;
    let outer_height = window
        .outer_height()
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0) as u32;
    let device_pixel_ratio = window.device_pixel_ratio();

    // Hardware information
    let hardware_concurrency = get_hardware_concurrency(&navigator);
    let device_memory = get_device_memory(&navigator);
    let max_touch_points = get_max_touch_points(&navigator);

    // Timezone information
    let timezone_offset = js_sys::Date::new_0().get_timezone_offset() as i32;
    let timezone = get_timezone();

    // Canvas fingerprint
    let canvas_fingerprint = if config.enable_canvas {
        generate_canvas_fingerprint(&window).await
    } else {
        None
    };

    // WebGL fingerprint
    let webgl_fingerprint = if config.enable_webgl {
        generate_webgl_fingerprint(&window).await
    } else {
        None
    };

    // Audio fingerprint
    let audio_fingerprint = if config.enable_audio && !config.privacy_mode {
        generate_audio_fingerprint(&window).await
    } else {
        None
    };

    // Font detection
    let available_fonts = if config.enable_fonts {
        detect_fonts(&window).await
    } else {
        Vec::new()
    };

    // Network information
    let (connection_type, connection_downlink, connection_rtt, connection_save_data) =
        get_connection_info(&navigator);

    // Performance timing
    let performance_timing = if config.enable_performance {
        get_performance_timing(&window)
    } else {
        None
    };

    // Media devices
    let media_devices_count = get_media_devices_count(&navigator).await;

    // Battery information
    let (battery_level, battery_charging) = get_battery_info(&navigator).await;

    // WebRTC fingerprint
    let webrtc_fingerprint = if config.enable_webrtc && !config.privacy_mode {
        generate_webrtc_fingerprint(&window).await
    } else {
        None
    };

    // Additional entropy sources
    let entropy_sources = collect_entropy_sources(&window);

    Ok(ClientFingerprintData {
        user_agent,
        language,
        languages,
        platform,
        cookie_enabled,
        do_not_track: Some(do_not_track),
        screen_width,
        screen_height,
        screen_color_depth,
        screen_pixel_depth,
        available_width,
        available_height,
        inner_width,
        inner_height,
        outer_width,
        outer_height,
        device_pixel_ratio,
        hardware_concurrency,
        device_memory,
        max_touch_points,
        timezone_offset,
        timezone,
        canvas_fingerprint,
        webgl_fingerprint,
        audio_fingerprint,
        available_fonts,
        connection_type,
        connection_downlink,
        connection_rtt,
        connection_save_data,
        performance_timing,
        media_devices_count,
        battery_level,
        battery_charging,
        webrtc_fingerprint,
        entropy_sources,
    })
}

/// Get supported languages
fn get_languages(navigator: &Navigator) -> Vec<String> {
    let languages = navigator.languages();
    let mut result = Vec::new();
    for i in 0..languages.length() {
        if let Some(lang) = languages.get(i).as_string() {
            result.push(lang);
        }
    }
    result
}

/// Get hardware concurrency
fn get_hardware_concurrency(navigator: &Navigator) -> Option<u32> {
    js_sys::Reflect::get(navigator, &JsValue::from_str("hardwareConcurrency"))
        .ok()
        .and_then(|v| v.as_f64())
        .map(|v| v as u32)
}

/// Get device memory
fn get_device_memory(navigator: &Navigator) -> Option<f64> {
    js_sys::Reflect::get(navigator, &JsValue::from_str("deviceMemory"))
        .ok()
        .and_then(|v| v.as_f64())
}

/// Get max touch points
fn get_max_touch_points(navigator: &Navigator) -> Option<u32> {
    js_sys::Reflect::get(navigator, &JsValue::from_str("maxTouchPoints"))
        .ok()
        .and_then(|v| v.as_f64())
        .map(|v| v as u32)
}

/// Get timezone using Intl API
fn get_timezone() -> String {
    js_sys::Reflect::get(
        &js_sys::Intl::DateTimeFormat::new(&js_sys::Array::new(), &js_sys::Object::new())
            .resolved_options(),
        &JsValue::from_str("timeZone"),
    )
    .ok()
    .and_then(|v| v.as_string())
    .unwrap_or_default()
}

/// Generate canvas fingerprint
async fn generate_canvas_fingerprint(window: &Window) -> Option<String> {
    let document = window.document()?;
    let canvas = document.create_element("canvas").ok()?;
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into().ok()?;

    canvas.set_width(200);
    canvas.set_height(50);

    let context = canvas.get_context("2d").ok()??;
    let context: web_sys::CanvasRenderingContext2d = context.dyn_into().ok()?;

    // Draw fingerprinting pattern
    context.set_text_baseline("top");
    context.set_font("14px Arial");
    context.set_fill_style_str("#f60");
    context.fill_rect(125.0, 1.0, 62.0, 20.0);

    context.set_fill_style_str("#069");
    context.fill_text("Fingerprint Test 🔒", 2.0, 15.0).ok()?;

    context.set_fill_style_str("rgba(102, 204, 0, 0.2)");
    context.fill_text("Privacy Compliant", 4.0, 17.0).ok()?;

    // Add some geometric shapes
    context.begin_path();
    let _ = context.arc(50.0, 25.0, 20.0, 0.0, 2.0 * std::f64::consts::PI);
    context.set_fill_style_str("rgba(255, 0, 0, 0.5)");
    context.fill();

    canvas.to_data_url().ok()
}

/// Generate WebGL fingerprint
async fn generate_webgl_fingerprint(window: &Window) -> Option<String> {
    let document = window.document()?;
    let canvas = document.create_element("canvas").ok()?;
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into().ok()?;

    let context = canvas.get_context("webgl").ok()??;
    let gl: web_sys::WebGlRenderingContext = context.dyn_into().ok()?;

    let mut fingerprint_parts = Vec::new();

    // Get WebGL parameters
    if let Ok(vendor) = gl.get_parameter(web_sys::WebGlRenderingContext::VENDOR) {
        fingerprint_parts.push(format!("vendor:{}", vendor.as_string().unwrap_or_default()));
    }

    if let Ok(renderer) = gl.get_parameter(web_sys::WebGlRenderingContext::RENDERER) {
        fingerprint_parts.push(format!(
            "renderer:{}",
            renderer.as_string().unwrap_or_default()
        ));
    }

    if let Ok(version) = gl.get_parameter(web_sys::WebGlRenderingContext::VERSION) {
        fingerprint_parts.push(format!(
            "version:{}",
            version.as_string().unwrap_or_default()
        ));
    }

    Some(fingerprint_parts.join("|"))
}

/// Generate audio fingerprint (privacy-sensitive)
async fn generate_audio_fingerprint(_window: &Window) -> Option<String> {
    // Audio fingerprinting is disabled by default for privacy
    // This would use AudioContext and AnalyserNode to generate unique audio signatures
    None
}

/// Detect available fonts
async fn detect_fonts(_window: &Window) -> Vec<String> {
    // Font detection using CSS and measurement techniques
    // This is a simplified version - full implementation would test many fonts
    let common_fonts = vec![
        "Arial",
        "Helvetica",
        "Times New Roman",
        "Courier New",
        "Verdana",
        "Georgia",
        "Palatino",
        "Garamond",
        "Bookman",
        "Comic Sans MS",
        "Trebuchet MS",
        "Arial Black",
        "Impact",
        "Tahoma",
        "Lucida Console",
        "Monaco",
    ];

    // In a real implementation, we would test each font by measuring text dimensions
    // For now, return a subset to simulate detection
    common_fonts
        .into_iter()
        .take(8)
        .map(|s| s.to_string())
        .collect()
}

/// Get connection information
fn get_connection_info(
    navigator: &Navigator,
) -> (Option<String>, Option<f64>, Option<u32>, Option<bool>) {
    let connection = js_sys::Reflect::get(navigator, &JsValue::from_str("connection"))
        .ok()
        .or_else(|| js_sys::Reflect::get(navigator, &JsValue::from_str("mozConnection")).ok())
        .or_else(|| js_sys::Reflect::get(navigator, &JsValue::from_str("webkitConnection")).ok());

    if let Some(conn) = connection {
        let connection_type = js_sys::Reflect::get(&conn, &JsValue::from_str("effectiveType"))
            .ok()
            .and_then(|v| v.as_string());

        let downlink = js_sys::Reflect::get(&conn, &JsValue::from_str("downlink"))
            .ok()
            .and_then(|v| v.as_f64());

        let rtt = js_sys::Reflect::get(&conn, &JsValue::from_str("rtt"))
            .ok()
            .and_then(|v| v.as_f64())
            .map(|v| v as u32);

        let save_data = js_sys::Reflect::get(&conn, &JsValue::from_str("saveData"))
            .ok()
            .and_then(|v| v.as_bool());

        (connection_type, downlink, rtt, save_data)
    } else {
        (None, None, None, None)
    }
}

/// Get performance timing information
fn get_performance_timing(window: &Window) -> Option<PerformanceTiming> {
    let performance = window.performance()?;
    let timing = js_sys::Reflect::get(&performance, &JsValue::from_str("timing")).ok()?;

    Some(PerformanceTiming {
        navigation_start: js_sys::Reflect::get(&timing, &JsValue::from_str("navigationStart"))
            .ok()?
            .as_f64()?,
        dom_content_loaded: js_sys::Reflect::get(
            &timing,
            &JsValue::from_str("domContentLoadedEventEnd"),
        )
        .ok()?
        .as_f64()?,
        load_complete: js_sys::Reflect::get(&timing, &JsValue::from_str("loadEventEnd"))
            .ok()?
            .as_f64()?,
        dns_lookup_time: (js_sys::Reflect::get(&timing, &JsValue::from_str("domainLookupEnd"))
            .ok()?
            .as_f64()?
            - js_sys::Reflect::get(&timing, &JsValue::from_str("domainLookupStart"))
                .ok()?
                .as_f64()?) as f64,
        tcp_connect_time: (js_sys::Reflect::get(&timing, &JsValue::from_str("connectEnd"))
            .ok()?
            .as_f64()?
            - js_sys::Reflect::get(&timing, &JsValue::from_str("connectStart"))
                .ok()?
                .as_f64()?) as f64,
    })
}

/// Get media devices count
async fn get_media_devices_count(_navigator: &Navigator) -> Option<u32> {
    // MediaDevices API access requires user permission
    // For privacy, we don't enumerate actual devices
    None
}

/// Get battery information
async fn get_battery_info(_navigator: &Navigator) -> (Option<f64>, Option<bool>) {
    // Battery API is deprecated in most browsers for privacy reasons
    (None, None)
}

/// Generate WebRTC fingerprint (privacy-sensitive)
async fn generate_webrtc_fingerprint(_window: &Window) -> Option<String> {
    // WebRTC fingerprinting is disabled by default for privacy
    // This would use RTCPeerConnection to gather local IP addresses
    None
}

/// Collect additional entropy sources
fn collect_entropy_sources(window: &Window) -> HashMap<String, String> {
    let mut entropy = HashMap::new();

    // Add timestamp for uniqueness
    entropy.insert("timestamp".to_string(), js_sys::Date::now().to_string());

    // Add random values for additional entropy
    entropy.insert("random".to_string(), js_sys::Math::random().to_string());

    // Add window properties
    if let Ok(history_length) = window.history().and_then(|h| h.length()) {
        entropy.insert("history_length".to_string(), history_length.to_string());
    }

    entropy
}
