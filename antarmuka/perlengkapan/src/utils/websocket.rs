//! WebSocket utilities for real-time dashboard updates

use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CloseEvent, ErrorEvent, MessageEvent, WebSocket as WebSocketSys};

/// Dashboard update message types (matches backend DashboardUpdate enum)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DashboardUpdate {
    /// Metrics update for perlengkapan dashboard
    MetricsUpdate {
        tahun_anggaran: i32,
        timestamp: String,
        data: serde_json::Value,
    },
    /// Workflow status change notification
    WorkflowUpdate {
        entity_id: String,
        entity_type: String,
        new_status: String,
        timestamp: String,
    },
    /// Gap analysis update
    GapAnalysisUpdate {
        satker_id: String,
        timestamp: String,
        data: serde_json::Value,
    },
    /// Heartbeat/ping message
    Ping {
        timestamp: String,
    },
    /// Connection confirmation
    Connected {
        client_id: String,
        timestamp: String,
    },
    /// Lag warning
    LagWarning {
        skipped_messages: u32,
        timestamp: String,
    },
    /// Subscription acknowledgment
    Subscribed {
        timestamp: String,
    },
    /// Pong response
    Pong {
        timestamp: String,
    },
}

/// WebSocket connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Connecting,
    Connected,
    Disconnected,
    Error,
}

/// WebSocket manager for dashboard updates
#[derive(Clone)]
pub struct DashboardWebSocket {
    ws: Option<WebSocketSys>,
    url: String,
    state: RwSignal<ConnectionState>,
    reconnect_attempts: RwSignal<u32>,
    max_reconnect_attempts: u32,
    reconnect_delay_ms: u32,
}

impl DashboardWebSocket {
    /// Create a new WebSocket connection
    pub fn new(url: String) -> Self {
        Self {
            ws: None,
            url,
            state: RwSignal::new(ConnectionState::Disconnected),
            reconnect_attempts: RwSignal::new(0),
            max_reconnect_attempts: 5,
            reconnect_delay_ms: 2000,
        }
    }

    /// Connect to WebSocket server
    pub fn connect<F>(&mut self, on_message: F) -> Result<(), JsValue>
    where
        F: Fn(DashboardUpdate) + 'static,
    {
        // Close existing connection if any
        if let Some(ws) = &self.ws {
            let _ = ws.close();
        }

        // Create new WebSocket connection
        let ws = WebSocketSys::new(&self.url)?;
        ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

        let state = self.state;
        let reconnect_attempts = self.reconnect_attempts;

        // Set up onopen handler
        let onopen_callback = Closure::wrap(Box::new(move |_| {
            leptos::logging::log!("WebSocket connected");
            state.set(ConnectionState::Connected);
            reconnect_attempts.set(0);
        }) as Box<dyn FnMut(JsValue)>);
        ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
        onopen_callback.forget();

        // Set up onmessage handler
        let on_message = Rc::new(on_message);
        let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
            if let Ok(text) = e.data().dyn_into::<js_sys::JsString>() {
                let text_str: String = text.into();
                leptos::logging::log!("WebSocket message received: {}", text_str);

                match serde_json::from_str::<DashboardUpdate>(&text_str) {
                    Ok(update) => {
                        on_message(update);
                    }
                    Err(e) => {
                        leptos::logging::error!("Failed to parse WebSocket message: {}", e);
                    }
                }
            }
        }) as Box<dyn FnMut(MessageEvent)>);
        ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();

        // Set up onerror handler
        let state_error = self.state;
        let onerror_callback = Closure::wrap(Box::new(move |e: ErrorEvent| {
            leptos::logging::error!("WebSocket error: {:?}", e);
            state_error.set(ConnectionState::Error);
        }) as Box<dyn FnMut(ErrorEvent)>);
        ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
        onerror_callback.forget();

        // Set up onclose handler with reconnection logic
        let state_close = self.state;
        let reconnect_attempts_close = self.reconnect_attempts;
        let max_attempts = self.max_reconnect_attempts;
        let delay_ms = self.reconnect_delay_ms;
        let url_clone = self.url.clone();

        let onclose_callback = Closure::wrap(Box::new(move |e: CloseEvent| {
            leptos::logging::warn!("WebSocket closed: code={}, reason={}", e.code(), e.reason());
            state_close.set(ConnectionState::Disconnected);

            // Attempt reconnection
            let current_attempts = reconnect_attempts_close.get();
            if current_attempts < max_attempts {
                reconnect_attempts_close.set(current_attempts + 1);
                leptos::logging::log!(
                    "Attempting to reconnect ({}/{})",
                    current_attempts + 1,
                    max_attempts
                );

                // Schedule reconnection
                let url_reconnect = url_clone.clone();
                let state_reconnect = state_close;
                let _reconnect_attempts_reconnect = reconnect_attempts_close;

                let timeout_callback = Closure::once(Box::new(move || {
                    state_reconnect.set(ConnectionState::Connecting);
                    // Note: Actual reconnection would need to be triggered from outside
                    // This is a limitation of the closure-based approach
                    leptos::logging::log!("Reconnection timeout triggered for {}", url_reconnect);
                }) as Box<dyn FnOnce()>);

                let window = web_sys::window().expect("no global window exists");
                let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                    timeout_callback.as_ref().unchecked_ref(),
                    delay_ms as i32,
                );
                timeout_callback.forget();
            } else {
                leptos::logging::error!("Max reconnection attempts reached");
            }
        }) as Box<dyn FnMut(CloseEvent)>);
        ws.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
        onclose_callback.forget();

        self.state.set(ConnectionState::Connecting);
        self.ws = Some(ws);

        Ok(())
    }

    /// Send a message to the server
    pub fn send(&self, message: &str) -> Result<(), JsValue> {
        if let Some(ws) = &self.ws {
            ws.send_with_str(message)?;
        }
        Ok(())
    }

    /// Send a ping message
    pub fn send_ping(&self) -> Result<(), JsValue> {
        let ping_msg = serde_json::json!({
            "type": "ping",
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        self.send(&ping_msg.to_string())
    }

    /// Subscribe to specific updates (future enhancement)
    pub fn subscribe(&self, filters: serde_json::Value) -> Result<(), JsValue> {
        let subscribe_msg = serde_json::json!({
            "type": "subscribe",
            "filters": filters,
        });
        self.send(&subscribe_msg.to_string())
    }

    /// Get current connection state
    pub fn state(&self) -> RwSignal<ConnectionState> {
        self.state
    }

    /// Close the WebSocket connection
    pub fn close(&mut self) {
        if let Some(ws) = &self.ws {
            let _ = ws.close();
        }
        self.ws = None;
        self.state.set(ConnectionState::Disconnected);
    }
}

impl Drop for DashboardWebSocket {
    fn drop(&mut self) {
        self.close();
    }
}

/// Hook for using WebSocket in components
pub fn use_dashboard_websocket<F>(
    url: String,
    on_message: F,
) -> (RwSignal<ConnectionState>, impl Fn())
where
    F: Fn(DashboardUpdate) + 'static + Clone,
{
    let ws = StoredValue::new_local(None::<DashboardWebSocket>);
    let state = RwSignal::new(ConnectionState::Disconnected);

    // Connect on mount
    let on_message_for_effect = on_message.clone();
    Effect::new(move || {
        let mut ws_manager = DashboardWebSocket::new(url.clone());
        let ws_state = ws_manager.state();

        // Sync state
        Effect::new(move || {
            state.set(ws_state.get());
        });

        // Connect
        if let Err(e) = ws_manager.connect(on_message_for_effect.clone()) {
            leptos::logging::error!("Failed to connect WebSocket: {:?}", e);
        }

        ws.set_value(Some(ws_manager));
    });

    // Cleanup on unmount
    on_cleanup(move || {
        if let Some(mut ws_manager) = ws.get_value() {
            ws_manager.close();
        }
    });

    // Reconnect function
    let reconnect = move || {
        if let Some(mut ws_manager) = ws.get_value() {
            ws_manager.close();
            if let Err(e) = ws_manager.connect(on_message.clone()) {
                leptos::logging::error!("Failed to reconnect WebSocket: {:?}", e);
            }
        }
    };

    (state, reconnect)
}
