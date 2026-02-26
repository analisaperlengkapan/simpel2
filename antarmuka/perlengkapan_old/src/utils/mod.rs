//! Utility modules for the Perlengkapan microfrontend

pub mod websocket;

pub use websocket::{
    ConnectionState, DashboardUpdate, DashboardWebSocket, use_dashboard_websocket,
};
