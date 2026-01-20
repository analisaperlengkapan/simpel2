//! Behavioral Analysis Components
//!
//! Components for collecting and analyzing user behavioral data

use super::types::*;
use gloo_timers::callback::Interval;
use leptos::ev;
use leptos::prelude::*;
use std::collections::HashMap;
use std::collections::VecDeque;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

/// Wrapper to make types Send + Sync for StoredValue in WASM
struct SendWrapper<T>(T);
unsafe impl<T> Send for SendWrapper<T> {}
unsafe impl<T> Sync for SendWrapper<T> {}

/// Window event listener wrapper for manual event management
struct WindowListener {
    event_name: &'static str,
    closure: Option<Closure<dyn FnMut(web_sys::MouseEvent)>>,
}

impl WindowListener {
    fn new(event_name: &'static str, callback: impl FnMut(web_sys::MouseEvent) + 'static) -> Self {
        let closure = Closure::wrap(Box::new(callback) as Box<dyn FnMut(_)>);
        if let Some(window) = web_sys::window() {
            let _ = window
                .add_event_listener_with_callback(event_name, closure.as_ref().unchecked_ref());
        }

        Self {
            event_name,
            closure: Some(closure),
        }
    }
}

impl Drop for WindowListener {
    fn drop(&mut self) {
        if let Some(closure) = &self.closure {
            if let Some(window) = web_sys::window() {
                let _ = window.remove_event_listener_with_callback(
                    self.event_name,
                    closure.as_ref().unchecked_ref(),
                );
            }
        }
    }
}

struct SendWindowListener(WindowListener);
unsafe impl Send for SendWindowListener {}
unsafe impl Sync for SendWindowListener {}

/// Helper to collect browser fingerprint
fn collect_fingerprint() -> Option<String> {
    if let Some(window) = web_sys::window() {
        let ua = window.navigator().user_agent().unwrap_or_default();
        let lang = window.navigator().language().unwrap_or_default();
        let screen = window.screen().ok();
        let resolution = if let Some(s) = screen {
            format!("{}x{}", s.width().unwrap_or(0), s.height().unwrap_or(0))
        } else {
            "unknown".to_string()
        };
        Some(format!("UA:{};Lang:{};Res:{}", ua, lang, resolution))
    } else {
        None
    }
}

/// Internal state for behavioral collector
#[derive(Clone)]
struct CollectorState {
    mouse_movements: VecDeque<MouseEvent>,
    keystroke_timings: VecDeque<KeystrokeEvent>,
    interaction_start: u64,
    last_mouse_move: u64,
}

impl Default for CollectorState {
    fn default() -> Self {
        Self {
            mouse_movements: VecDeque::new(),
            keystroke_timings: VecDeque::new(),
            interaction_start: js_sys::Date::now() as u64,
            last_mouse_move: 0,
        }
    }
}

/// Behavioral data collector hook
pub fn use_behavioral_collector() -> (ReadSignal<BehavioralData>, WriteSignal<BehavioralData>) {
    let (behavioral_data, set_behavioral_data) = signal(BehavioralData {
        mouse_movements: vec![],
        keystroke_timings: vec![],
        interaction_duration: 0,
        browser_fingerprint: collect_fingerprint(),
    });

    let state = StoredValue::new(CollectorState::default());

    // Initialize mouse tracking listeners
    Effect::new(move |_| {
        let _ = window_event_listener(leptos::ev::mousemove, move |ev| {
            state.update_value(|s| {
                let now = js_sys::Date::now() as u64;
                // Throttle updates to ~20fps (50ms)
                if now - s.last_mouse_move > 50 {
                    s.last_mouse_move = now;
                    s.mouse_movements.push_back(MouseEvent {
                        x: ev.client_x() as f64,
                        y: ev.client_y() as f64,
                        timestamp: now,
                        event_type: "mousemove".to_string(),
                    });
                    // Keep last 100 events
                    if s.mouse_movements.len() > 100 {
                        s.mouse_movements.pop_front();
                    }
                }
            });
        });

        let _ = window_event_listener(leptos::ev::click, move |ev| {
            state.update_value(|s| {
                let now = js_sys::Date::now() as u64;
                s.mouse_movements.push_back(MouseEvent {
                    x: ev.client_x() as f64,
                    y: ev.client_y() as f64,
                    timestamp: now,
                    event_type: "click".to_string(),
                });
                if s.mouse_movements.len() > 100 {
                    s.mouse_movements.pop_front();
                }
            });
        });
    });

    // Initialize keystroke tracking listeners
    Effect::new(move |_| {
        let _ = window_event_listener(leptos::ev::keydown, move |ev| {
            state.update_value(|s| {
                let now = js_sys::Date::now() as u64;
                s.keystroke_timings.push_back(KeystrokeEvent {
                    key: ev.key(),
                    timestamp: now,
                    duration: 0, // Duration would require keyup matching
                });
                if s.keystroke_timings.len() > 50 {
                    s.keystroke_timings.pop_front();
                }
            });
        });

        let _ = window_event_listener(leptos::ev::keyup, move |ev| {
            state.update_value(|s| {
                let now = js_sys::Date::now() as u64;
                // For now just record keyup as an event, or we could update duration of previous keydown
                // Simplified: just log it
                s.keystroke_timings.push_back(KeystrokeEvent {
                    key: ev.key(),
                    timestamp: now,
                    duration: 0,
                });
                if s.keystroke_timings.len() > 50 {
                    s.keystroke_timings.pop_front();
                }
            });
        });
    });

    // Periodic synchronization to signal
    Effect::new(move |_| {
        let _handle = StoredValue::new(SendWrapper(Interval::new(1000, move || {
            state.with_value(|s| {
                let now = js_sys::Date::now() as u64;
                set_behavioral_data.update(|data| {
                    data.mouse_movements = s.mouse_movements.iter().cloned().collect();
                    data.keystroke_timings = s.keystroke_timings.iter().cloned().collect();
                    data.interaction_duration = now - s.interaction_start;
                });
            });
        })));
    });

    (behavioral_data, set_behavioral_data)
}

/// Mouse tracking component
#[component]
pub fn MouseTracker(on_data_collected: Callback<Vec<MouseEvent>>) -> impl IntoView {
    let (_mouse_events, set_mouse_events) = signal(Vec::<MouseEvent>::new());

    Effect::new(move |_| {
        // Mouse move handler
        let set_mouse_events_clone = set_mouse_events;
        let on_data_collected_clone = on_data_collected;
        let listener_move = SendWindowListener(WindowListener::new(
            "mousemove",
            move |e: web_sys::MouseEvent| {
                let event = MouseEvent {
                    x: e.client_x() as f64,
                    y: e.client_y() as f64,
                    timestamp: js_sys::Date::now() as u64,
                    event_type: "mousemove".to_string(),
                };

                set_mouse_events_clone.update(|events| {
                    if events.len() >= 100 {
                        events.remove(0);
                    }
                    events.push(event);
                    on_data_collected_clone.run(events.clone());
                });
            },
        ));

        // Click handler
        let listener_click = SendWindowListener(WindowListener::new(
            "click",
            move |e: web_sys::MouseEvent| {
                let event = MouseEvent {
                    x: e.client_x() as f64,
                    y: e.client_y() as f64,
                    timestamp: js_sys::Date::now() as u64,
                    event_type: "click".to_string(),
                };

                set_mouse_events.update(|events| {
                    if events.len() >= 100 {
                        events.remove(0);
                    }
                    events.push(event);
                    on_data_collected.run(events.clone());
                });
            },
        ));

        on_cleanup(move || {
            drop(listener_move);
            drop(listener_click);
        });
    });

    view! {
        <div
            class="mouse-tracker"
            style="position: absolute; top: 0; left: 0; width: 100%; height: 100%; pointer-events: none;"
        >
            // Invisible overlay
        </div>
    }
}

/// Keystroke dynamics analyzer with duration tracking
#[component]
pub fn KeystrokeAnalyzer(on_data_collected: Callback<Vec<KeystrokeEvent>>) -> impl IntoView {
    let (keystroke_events, set_keystroke_events) = signal(Vec::<KeystrokeEvent>::new());
    let pending_keys = StoredValue::new(HashMap::<String, u64>::new());

    // Keydown listener to capture press start time
    let on_keydown = move |ev: web_sys::KeyboardEvent| {
        let key = ev.key();
        let timestamp = js_sys::Date::now() as u64;

        pending_keys.update_value(|map| {
            map.entry(key).or_insert(timestamp);
        });
    };

    // Keyup listener to calculate duration and record event
    let on_keyup = move |ev: web_sys::KeyboardEvent| {
        let key = ev.key();
        let timestamp = js_sys::Date::now() as u64;

        let mut start_time = None;
        pending_keys.update_value(|map| {
            start_time = map.remove(&key);
        });

        if let Some(start) = start_time {
            let duration = timestamp.saturating_sub(start);
            let event = KeystrokeEvent {
                key,
                timestamp: start,
                duration,
            };

            set_keystroke_events.update(|events| events.push(event));
            on_data_collected.run(keystroke_events.get_untracked());
        }
    };

    // Attach listeners to window
    // leptos::window_event_listener automatically handles cleanup when component is dropped
    let _ = window_event_listener(ev::keydown, on_keydown);
    let _ = window_event_listener(ev::keyup, on_keyup);

    view! {
        <div class="keystroke-analyzer" style="display: none;">
            // Component for keystroke analysis (invisible)
        </div>
    }
}
