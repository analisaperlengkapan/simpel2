//! # SIMPelv2 Datun - Criminal Prosecution System
//!
//! Server entry point for the Datun criminal prosecution microfrontend.
//! This file is only compiled for non-WASM targets.

#[cfg(not(target_arch = "wasm32"))]
use axum::{response::Html, routing::get, Router};

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() {
    println!("⚖️ Starting Datun SIMPelv2 Server...");

    // Build the application router
    let app = Router::new()
        .route("/", get(render_app))
        .route("/api/health", get(health_check));

    // Start the server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8082")
        .await
        .expect("Failed to bind to address");

    println!("🚀 Datun server running on http://127.0.0.1:8082");
    println!("⚖️ Criminal Prosecution Management System Ready!");

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}

#[cfg(not(target_arch = "wasm32"))]
/// Render the main application
async fn render_app() -> Html<String> {
    let html = r#"<!DOCTYPE html>
<html lang="id">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Datun SIMPelv2 - Direktorat Tindak Pidana Umum Kejaksaan RI</title>
    <meta name="description" content="Sistem Informasi Direktorat Tindak Pidana Umum Kejaksaan Agung Republik Indonesia">
    <link rel="stylesheet" href="/styles/output.css">
    <link rel="icon" type="image/png" href="/assets/favicon.png">
</head>
<body>
    <div id="app">Loading...</div>
    <script type="module" src="/assets/datun.js"></script>
</body>
</html>"#.to_string();

    Html(html)
}

#[cfg(not(target_arch = "wasm32"))]
/// Health check endpoint
async fn health_check() -> &'static str {
    "⚖️ Datun SIMPelv2 - Criminal Prosecution System Healthy!"
}

// For WASM builds, we don't need a main function
#[cfg(target_arch = "wasm32")]
fn main() {
    // Empty main for WASM builds - actual entry point is in lib.rs
}
