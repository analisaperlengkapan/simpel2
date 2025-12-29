//! Behavioral Data Collection Component
//!
//! Client-side behavioral data collection for bot detection

use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{KeyboardEvent, MouseEvent as WebMouseEvent, Window};

/// Mouse event data for behavioral analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseEventData {
    pub x: f64,
    pub y: f64,
    pub timestamp: u64,
    pub event_type: String,
    pub velocity: Option<f64>,
    pub acceleration: Option<f64>,
}

/// Keystroke event data for behavioral analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeystrokeEventData {
    pub key: String,
    pub timestamp: u64,
    pub duration: u64,
    pub dwell_time: u64,
    pub flight_time: Option<u64>,
}

/// Timing analysis data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingAnalysisData {
    pub total_interaction_time: u64,
    pub pause_patterns: Vec<u64>,
    pub rhythm_consistency: f64,
    pub typing_speed: Option<f64>,
}

/// Browser fingerprint data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserFingerprintData {
    pub user_agent: String,
    pub screen_resolution: String,
    pub timezone: String,
    pub language: String,
    pub plugins: Vec<String>,
    pub canvas_fingerprint: Option<String>,
    pub webgl_fingerprint: Option<String>,
}

/// Complete behavioral metrics collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehavioralMetricsData {
    pub session_id: String,
    pub mouse_movements: Vec<MouseEventData>,
    pub keystroke_dynamics: Vec<KeystrokeEventData>,
    pub timing_patterns: TimingAnalysisData,
    pub browser_fingerprint: BrowserFingerprintData,
    pub start_time: u64,
}

/// Behavioral collector configuration
#[derive(Debug, Clone)]
pub struct BehavioralCollectorConfig {
    /// Maximum number of mouse events to store
    pub max_mouse_events: usize,
    /// Maximum number of keystroke events to store
    pub max_keystroke_events: usize,
    /// Minimum time between mouse events to record (ms)
    pub mouse_sampling_interval: u64,
    /// Enable canvas fingerprinting
    pub enable_canvas_fingerprinting: bool,
    /// Enable WebGL fingerprinting
    pub enable_webgl_fingerprinting: bool,
}

impl Default for BehavioralCollectorConfig {
    fn default() -> Self {
        Self {
            max_mouse_events: 100,
            max_keystroke_events: 50,
            mouse_sampling_interval: 10, // 10ms
            enable_canvas_fingerprinting: true,
            enable_webgl_fingerprinting: true,
        }
    }
}

/// Behavioral data collector component
#[component]
pub fn BehavioralCollector(
    /// Session ID for tracking
    session_id: String,
    /// Configuration for data collection
    #[prop(optional)]
    config: Option<BehavioralCollectorConfig>,
    /// Callback when behavioral data is collected
    #[prop(optional)]
    on_data_collected: Option<Callback<BehavioralMetricsData>>,
    /// Children components
    children: Children,
) -> impl IntoView {
    let config = config.unwrap_or_default();

    // State for behavioral data
    let (_behavioral_data, set_behavioral_data) = signal(BehavioralMetricsData {
        session_id: session_id.clone(),
        mouse_movements: Vec::new(),
        keystroke_dynamics: Vec::new(),
        timing_patterns: TimingAnalysisData {
            total_interaction_time: 0,
            pause_patterns: Vec::new(),
            rhythm_consistency: 0.0,
            typing_speed: None,
        },
        browser_fingerprint: BrowserFingerprintData {
            user_agent: String::new(),
            screen_resolution: String::new(),
            timezone: String::new(),
            language: String::new(),
            plugins: Vec::new(),
            canvas_fingerprint: None,
            webgl_fingerprint: None,
        },
        start_time: js_sys::Date::now() as u64,
    });

    // Mouse event tracking
    let mouse_events = RwSignal::new(VecDeque::<MouseEventData>::new());
    let last_mouse_time = RwSignal::new(0u64);

    // Keystroke event tracking
    let keystroke_events = RwSignal::new(VecDeque::<KeystrokeEventData>::new());
    let key_down_times = RwSignal::new(std::collections::HashMap::<String, u64>::new());
    let last_keystroke_time = RwSignal::new(0u64);

    // Clone config for use in effect
    let config_clone = config.clone();

    // Initialize browser fingerprinting on mount
    Effect::new(move |_| {
        if let Ok(window) = web_sys::window().ok_or("No window") {
            let fingerprint = collect_browser_fingerprint(&window, &config_clone);
            set_behavioral_data.update(|data| {
                data.browser_fingerprint = fingerprint;
            });
        }
    });

    // Mouse event handler
    let handle_mouse_event = {
        let config = config.clone();
        move |event: WebMouseEvent| {
            let now = js_sys::Date::now() as u64;
            let last_time = last_mouse_time.get();

            // Sample mouse events based on interval
            if now - last_time >= config.mouse_sampling_interval {
                let x = event.client_x() as f64;
                let y = event.client_y() as f64;

                // Calculate velocity if we have previous event
                let velocity = mouse_events.with(|events| {
                    if let Some(last_event) = events.back() {
                        let dx = x - last_event.x;
                        let dy = y - last_event.y;
                        let dt = (now - last_event.timestamp) as f64 / 1000.0;
                        if dt > 0.0 {
                            Some(((dx * dx + dy * dy).sqrt()) / dt)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                });

                let mouse_event = MouseEventData {
                    x,
                    y,
                    timestamp: now,
                    event_type: event.type_(),
                    velocity,
                    acceleration: None, // Will be calculated in post-processing
                };

                mouse_events.update(|events| {
                    events.push_back(mouse_event);
                    if events.len() > config.max_mouse_events {
                        events.pop_front();
                    }
                });

                last_mouse_time.set(now);
            }
        }
    };

    // Keyboard event handlers
    let handle_key_down = move |event: KeyboardEvent| {
        let now = js_sys::Date::now() as u64;
        let key = event.key();

        key_down_times.update(|times| {
            times.insert(key, now);
        });
    };

    let handle_key_up = {
        let config = config.clone();
        move |event: KeyboardEvent| {
            let now = js_sys::Date::now() as u64;
            let key = event.key();

            if let Some(down_time) = key_down_times.with(|times| times.get(&key).copied()) {
                let dwell_time = now - down_time;
                let flight_time = if last_keystroke_time.get() > 0 {
                    Some(down_time - last_keystroke_time.get())
                } else {
                    None
                };

                let keystroke_event = KeystrokeEventData {
                    key: key.clone(),
                    timestamp: now,
                    duration: dwell_time,
                    dwell_time,
                    flight_time,
                };

                keystroke_events.update(|events| {
                    events.push_back(keystroke_event);
                    if events.len() > config.max_keystroke_events {
                        events.pop_front();
                    }
                });

                last_keystroke_time.set(now);

                // Remove from key down times
                key_down_times.update(|times| {
                    times.remove(&key);
                });
            }
        }
    };

    // Periodic data collection and analysis
    Effect::new(move |_| {
        let interval = set_interval(
            move || {
                let now = js_sys::Date::now() as u64;

                // Update behavioral data
                set_behavioral_data.update(|data| {
                    // Update mouse movements
                    data.mouse_movements =
                        mouse_events.with(|events| events.iter().cloned().collect());

                    // Update keystroke dynamics
                    data.keystroke_dynamics =
                        keystroke_events.with(|events| events.iter().cloned().collect());

                    // Update timing patterns
                    data.timing_patterns.total_interaction_time = now - data.start_time;

                    // Calculate rhythm consistency from keystroke data
                    if data.keystroke_dynamics.len() > 1 {
                        let intervals: Vec<f64> = data
                            .keystroke_dynamics
                            .windows(2)
                            .map(|pair| (pair[1].timestamp - pair[0].timestamp) as f64)
                            .collect();

                        if !intervals.is_empty() {
                            let mean = intervals.iter().sum::<f64>() / intervals.len() as f64;
                            let variance =
                                intervals.iter().map(|x| (x - mean).powi(2)).sum::<f64>()
                                    / intervals.len() as f64;

                            data.timing_patterns.rhythm_consistency = if variance > 0.0 {
                                1.0 / (1.0 + variance / 10000.0)
                            } else {
                                1.0
                            };
                        }
                    }
                });

                // Trigger callback if provided
                if let Some(_callback) = on_data_collected {
                    // callback(behavioral_data.get());
                }
            },
            std::time::Duration::from_millis(1000), // Update every second
        );

        on_cleanup(move || {
            clear_interval(interval);
        });
    });

    view! {
        <div
            class="behavioral-collector"
            on:mousemove=handle_mouse_event
            on:mousedown=handle_mouse_event
            on:mouseup=handle_mouse_event
            on:keydown=handle_key_down
            on:keyup=handle_key_up
        >
            {children()}
        </div>
    }
}

/// Collect browser fingerprint data
fn collect_browser_fingerprint(
    window: &Window,
    config: &BehavioralCollectorConfig,
) -> BrowserFingerprintData {
    let navigator = window.navigator();

    // User agent
    let user_agent = navigator.user_agent().unwrap_or_default();

    // Screen resolution
    let screen_resolution = if let Ok(screen) = window.screen() {
        format!(
            "{}x{}",
            screen.width().unwrap_or(0),
            screen.height().unwrap_or(0)
        )
    } else {
        String::new()
    };

    // Timezone
    let timezone = js_sys::Reflect::get(
        &js_sys::Intl::DateTimeFormat::new(&js_sys::Array::new(), &js_sys::Object::new())
            .resolved_options(),
        &JsValue::from_str("timeZone"),
    )
    .ok()
    .and_then(|v| v.as_string())
    .unwrap_or_default();

    // Language
    let language = navigator.language().unwrap_or_default();

    // Plugins (limited in modern browsers) - disabled for compatibility
    let plugins = Vec::<String>::new();

    // Canvas fingerprint
    let canvas_fingerprint = if config.enable_canvas_fingerprinting {
        generate_canvas_fingerprint(window)
    } else {
        None
    };

    // WebGL fingerprint
    let webgl_fingerprint = if config.enable_webgl_fingerprinting {
        generate_webgl_fingerprint(window)
    } else {
        None
    };

    BrowserFingerprintData {
        user_agent,
        screen_resolution,
        timezone,
        language,
        plugins,
        canvas_fingerprint,
        webgl_fingerprint,
    }
}

/// Generate canvas fingerprint
fn generate_canvas_fingerprint(window: &Window) -> Option<String> {
    let document = window.document()?;
    let canvas = document.create_element("canvas").ok()?;
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into().ok()?;

    canvas.set_width(200);
    canvas.set_height(50);

    let context = canvas.get_context("2d").ok()??;
    let context: web_sys::CanvasRenderingContext2d = context.dyn_into().ok()?;

    // Draw some text and shapes for fingerprinting
    context.set_text_baseline("top");
    context.set_font("14px Arial");
    context.set_fill_style_str("#f60");
    context.fill_rect(125.0, 1.0, 62.0, 20.0);

    context.set_fill_style_str("#069");
    context
        .fill_text("BrowserLeaks,com <canvas> 1.0", 2.0, 15.0)
        .ok()?;

    context.set_fill_style_str("rgba(102, 204, 0, 0.2)");
    context
        .fill_text("BrowserLeaks,com <canvas> 1.0", 4.0, 17.0)
        .ok()?;

    // Get canvas data URL as fingerprint
    canvas.to_data_url().ok()
}

/// Generate WebGL fingerprint
fn generate_webgl_fingerprint(window: &Window) -> Option<String> {
    let document = window.document()?;
    let canvas = document.create_element("canvas").ok()?;
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into().ok()?;

    let context = canvas.get_context("webgl").ok()??;
    let gl: web_sys::WebGlRenderingContext = context.dyn_into().ok()?;

    // Get WebGL parameters for fingerprinting
    let mut fingerprint_parts = Vec::new();

    if let Ok(vendor) = gl
        .get_parameter(web_sys::WebGlRenderingContext::VENDOR)
    {
        fingerprint_parts.push(format!("vendor:{}", vendor.as_string().unwrap_or_default()));
    }

    if let Ok(renderer) = gl
        .get_parameter(web_sys::WebGlRenderingContext::RENDERER)
    {
        fingerprint_parts.push(format!(
            "renderer:{}",
            renderer.as_string().unwrap_or_default()
        ));
    }

    if let Ok(version) = gl
        .get_parameter(web_sys::WebGlRenderingContext::VERSION)
    {
        fingerprint_parts.push(format!(
            "version:{}",
            version.as_string().unwrap_or_default()
        ));
    }

    Some(fingerprint_parts.join("|"))
}

// Helper functions for interval management
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = setInterval)]
    fn set_interval_with_handle(handler: &js_sys::Function, timeout: i32) -> i32;

    #[wasm_bindgen(js_name = clearInterval)]
    fn clear_interval(handle: i32);
}

fn set_interval<F>(mut f: F, duration: std::time::Duration) -> i32
where
    F: FnMut() + 'static,
{
    let closure = Closure::wrap(Box::new(f) as Box<dyn FnMut()>);
    let handle = set_interval_with_handle(
        closure.as_ref().unchecked_ref(),
        duration.as_millis() as i32,
    );
    closure.forget(); // Keep closure alive
    handle
}
