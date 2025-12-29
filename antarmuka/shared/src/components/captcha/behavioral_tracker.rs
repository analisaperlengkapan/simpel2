//! Simple Behavioral Tracking Component
//!
//! Lightweight behavioral data collection for CAPTCHA

use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use web_sys::{KeyboardEvent, MouseEvent as WebMouseEvent};

/// Simple mouse tracking data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleMouseData {
    pub x: f64,
    pub y: f64,
    pub timestamp: u64,
    pub event_type: String,
}

/// Simple keystroke tracking data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleKeystrokeData {
    pub key: String,
    pub timestamp: u64,
    pub event_type: String,
}

/// Simple behavioral metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleBehavioralMetrics {
    pub session_id: String,
    pub mouse_events: Vec<SimpleMouseData>,
    pub keystroke_events: Vec<SimpleKeystrokeData>,
    pub interaction_start: u64,
    pub total_interactions: u32,
    pub mouse_velocity_avg: f64,
    pub typing_rhythm_score: f64,
}

/// Simple behavioral tracker component
#[component]
pub fn SimpleBehavioralTracker(
    /// Session ID for tracking
    session_id: String,
    /// Callback when data is collected
    #[prop(optional)]
    on_data_update: Option<Callback<SimpleBehavioralMetrics>>,
    /// Children components
    children: Children,
) -> impl IntoView {
    let _ = session_id;
    let _ = on_data_update;

    // Tracking state
    let (_mouse_events, set_mouse_events) = signal(VecDeque::<SimpleMouseData>::new());
    let (_keystroke_events, set_keystroke_events) = signal(VecDeque::<SimpleKeystrokeData>::new());
    let (_interaction_start, _set_interaction_start) = signal(js_sys::Date::now() as u64);
    let (_total_interactions, set_total_interactions) = signal(0u32);

    // Mouse event handler
    let handle_mouse_event = move |event: WebMouseEvent| {
        let now = js_sys::Date::now() as u64;
        let mouse_data = SimpleMouseData {
            x: event.client_x() as f64,
            y: event.client_y() as f64,
            timestamp: now,
            event_type: event.type_(),
        };

        set_mouse_events.update(|events| {
            events.push_back(mouse_data);
            if events.len() > 50 {
                // Keep last 50 events
                events.pop_front();
            }
        });

        set_total_interactions.update(|count| *count += 1);
    };

    // Keyboard event handler
    let handle_keyboard_event = move |event: KeyboardEvent| {
        let now = js_sys::Date::now() as u64;
        let keystroke_data = SimpleKeystrokeData {
            key: event.key(),
            timestamp: now,
            event_type: event.type_(),
        };

        set_keystroke_events.update(|events| {
            events.push_back(keystroke_data);
            if events.len() > 30 {
                // Keep last 30 events
                events.pop_front();
            }
        });

        set_total_interactions.update(|count| *count += 1);
    };

    // Periodic data analysis and callback - disabled due to closure issues
    /*
    Effect::new(move |_| {
        let interval_handle = gloo_timers::callback::Interval::new(2000, move || {
            // Calculate metrics
            let mouse_events_vec = mouse_events.get_untracked().into_iter().collect::<Vec<_>>();
            let keystroke_events_vec = keystroke_events
                .get_untracked()
                .into_iter()
                .collect::<Vec<_>>();

            // Calculate average mouse velocity
            let mouse_velocity_avg = if mouse_events_vec.len() > 1 {
                let velocities: Vec<f64> = mouse_events_vec
                    .windows(2)
                    .map(|pair| {
                        let dx = pair[1].x - pair[0].x;
                        let dy = pair[1].y - pair[0].y;
                        let dt = (pair[1].timestamp - pair[0].timestamp) as f64 / 1000.0;
                        if dt > 0.0 {
                            ((dx * dx + dy * dy).sqrt()) / dt
                        } else {
                            0.0
                        }
                    })
                    .collect();

                if !velocities.is_empty() {
                    velocities.iter().sum::<f64>() / velocities.len() as f64
                } else {
                    0.0
                }
            } else {
                0.0
            };

            // Calculate typing rhythm score
            let typing_rhythm_score = if keystroke_events_vec.len() > 2 {
                let intervals: Vec<f64> = keystroke_events_vec
                    .windows(2)
                    .filter(|pair| {
                        pair[0].event_type == "keydown" && pair[1].event_type == "keydown"
                    })
                    .map(|pair| (pair[1].timestamp - pair[0].timestamp) as f64)
                    .collect();

                if intervals.len() > 1 {
                    let mean = intervals.iter().sum::<f64>() / intervals.len() as f64;
                    let variance = intervals.iter().map(|x| (x - mean).powi(2)).sum::<f64>()
                        / intervals.len() as f64;

                    // Higher score for more consistent timing (lower variance)
                    if variance > 0.0 {
                        1.0 / (1.0 + variance / 10000.0)
                    } else {
                        1.0
                    }
                } else {
                    0.5
                }
            } else {
                0.0
            };

            let metrics = SimpleBehavioralMetrics {
                session_id: session_id.clone(),
                mouse_events: mouse_events_vec,
                keystroke_events: keystroke_events_vec,
                interaction_start: interaction_start.get_untracked(),
                total_interactions: total_interactions.get_untracked(),
                mouse_velocity_avg,
                typing_rhythm_score,
            };

            if let Some(callback) = on_data_update {
                // callback(metrics);
            }
        });

        on_cleanup(move || {
            drop(interval_handle);
        });
    });
    */

    view! {
        <div
            class="behavioral-tracker"
            on:mousemove=handle_mouse_event
            on:mousedown=handle_mouse_event
            on:mouseup=handle_mouse_event
            on:click=handle_mouse_event
            on:keydown=handle_keyboard_event
            on:keyup=handle_keyboard_event
        >
            {children()}
        </div>
    }
}

/// Behavioral metrics display component
#[component]
pub fn BehavioralMetricsDisplay(
    metrics: ReadSignal<Option<SimpleBehavioralMetrics>>,
    show_details: bool,
) -> impl IntoView {
    view! {
        {move || {
            if let Some(metrics_data) = metrics.get() {
                view! {
                    <div class="behavioral-metrics text-xs space-y-1">
                        <div class="flex items-center space-x-2">
                            <span class="w-2 h-2 bg-blue-500 rounded-full"></span>
                            <span class="text-gray-600 dark:text-gray-400">
                                "Interactions: " {metrics_data.total_interactions}
                            </span>
                        </div>

                        {if show_details {
                            view! {
                                <div class="space-y-1 text-gray-500 dark:text-gray-500">
                                    <div>"Mouse events: " {metrics_data.mouse_events.len()}</div>
                                    <div>"Keystroke events: " {metrics_data.keystroke_events.len()}</div>
                                    <div>"Avg velocity: " {format!("{:.1}", metrics_data.mouse_velocity_avg)} " px/s"</div>
                                    <div>"Rhythm score: " {format!("{:.2}", metrics_data.typing_rhythm_score)}</div>
                                </div>
                            }.into_any()
                        } else {
                            view! {}.into_any()
                        }}
                    </div>
                }.into_any()
            } else {
                view! {
                    <div class="behavioral-metrics text-xs text-gray-500 dark:text-gray-500">
                        "Initializing behavioral analysis..."
                    </div>
                }.into_any()
            }
        }}
    }
}
