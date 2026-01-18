//! Behavioral Analysis Components
//!
//! Components for collecting and analyzing user behavioral data

use super::types::*;
use leptos::prelude::*;
use std::collections::VecDeque;
use gloo_timers::callback::Interval;

/// Wrapper to make types Send + Sync for StoredValue in WASM
struct SendWrapper<T>(T);
unsafe impl<T> Send for SendWrapper<T> {}
unsafe impl<T> Sync for SendWrapper<T> {}

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
    let mouse_events = StoredValue::new(VecDeque::<MouseEvent>::new());
    let last_update = StoredValue::new(0u64);

    Effect::new(move |_| {
        let _ = window_event_listener(leptos::ev::mousemove, move |ev| {
            let now = js_sys::Date::now() as u64;
            let mut should_update = false;

            last_update.update_value(|last| {
                if now - *last > 50 {
                    *last = now;
                    should_update = true;
                }
            });

            if should_update {
                mouse_events.update_value(|events| {
                    events.push_back(MouseEvent {
                        x: ev.client_x() as f64,
                        y: ev.client_y() as f64,
                        timestamp: now,
                        event_type: "mousemove".to_string(),
                    });
                    if events.len() > 100 {
                        events.pop_front();
                    }
                });
            }
        });

        let _ = window_event_listener(leptos::ev::click, move |ev| {
            mouse_events.update_value(|events| {
                events.push_back(MouseEvent {
                    x: ev.client_x() as f64,
                    y: ev.client_y() as f64,
                    timestamp: js_sys::Date::now() as u64,
                    event_type: "click".to_string(),
                });
                 if events.len() > 100 {
                    events.pop_front();
                }
            });
        });
    });

    // Periodically report data
    Effect::new(move |_| {
        let _handle = StoredValue::new(SendWrapper(Interval::new(1000, move || {
            mouse_events.with_value(|events| {
                if !events.is_empty() {
                    on_data_collected.run(events.iter().cloned().collect());
                }
            });
        })));
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

/// Keystroke dynamics analyzer
#[component]
pub fn KeystrokeAnalyzer(on_data_collected: Callback<Vec<KeystrokeEvent>>) -> impl IntoView {
    let keystroke_events = StoredValue::new(VecDeque::<KeystrokeEvent>::new());

    Effect::new(move |_| {
        let _ = window_event_listener(leptos::ev::keydown, move |ev| {
            keystroke_events.update_value(|events| {
                events.push_back(KeystrokeEvent {
                    key: ev.key(),
                    timestamp: js_sys::Date::now() as u64,
                    duration: 0,
                });
                if events.len() > 50 {
                    events.pop_front();
                }
            });
        });

         let _ = window_event_listener(leptos::ev::keyup, move |ev| {
            keystroke_events.update_value(|events| {
                events.push_back(KeystrokeEvent {
                    key: ev.key(),
                    timestamp: js_sys::Date::now() as u64,
                    duration: 0,
                });
                if events.len() > 50 {
                    events.pop_front();
                }
            });
        });
    });

    Effect::new(move |_| {
        let _handle = StoredValue::new(SendWrapper(Interval::new(1000, move || {
            keystroke_events.with_value(|events| {
                if !events.is_empty() {
                    on_data_collected.run(events.iter().cloned().collect());
                }
            });
        })));
    });

    view! {
        <div class="keystroke-analyzer">
            // Component for keystroke analysis
        </div>
    }
}
