use crate::{
    config::SimplConfig,
    utils::output::SimplOutput,
};
use anyhow::Result;
use handlebars::Handlebars;
use serde_json::json;
use std::collections::HashMap;

pub async fn handle_make_command(
    command: &crate::commands::MakeCommands,
    config: &SimplConfig,
    output: &SimplOutput,
) -> Result<()> {
    match command {
        crate::commands::MakeCommands::Service { name, service_type } => {
            make_service(name, service_type, config, output).await
        }
        crate::commands::MakeCommands::Frontend { name, frontend_type } => {
            make_frontend(name, frontend_type, config, output).await
        }
        crate::commands::MakeCommands::Migration { name, service } => {
            make_migration(name, service, config, output).await
        }
        crate::commands::MakeCommands::Model { name, service } => {
            make_model(name, service, config, output).await
        }
        crate::commands::MakeCommands::Handler { name, service } => {
            make_handler(name, service, config, output).await
        }
        crate::commands::MakeCommands::Test { name, service } => {
            make_test(name, service, config, output).await
        }
    }
}

async fn make_service(
    name: &str,
    service_type: &str,
    config: &SimplConfig,
    output: &SimplOutput,
) -> Result<()> {
    output.info(&format!("Creating new {} service: {}", service_type, name))?;

    let service_path = format!("layanan/{}", name);
    
    // Check if service already exists
    if tokio::fs::metadata(&service_path).await.is_ok() {
        output.error(&format!("Service {} already exists", name))?;
        return Ok(());
    }

    let progress = output.progress("Creating service...", 6)?;

    // Create service directory
    progress.set_message("Creating directory structure...");
    create_service_directory(&service_path, output).await?;
    progress.inc(1);

    // Create Cargo.toml
    progress.set_message("Creating Cargo.toml...");
    create_service_cargo_toml(name, &service_path, service_type, output).await?;
    progress.inc(1);

    // Create main.rs
    progress.set_message("Creating main.rs...");
    create_service_main(name, &service_path, service_type, output).await?;
    progress.inc(1);

    // Create lib.rs
    progress.set_message("Creating lib.rs...");
    create_service_lib(name, &service_path, service_type, output).await?;
    progress.inc(1);

    // Create handlers
    progress.set_message("Creating handlers...");
    create_service_handlers(name, &service_path, service_type, output).await?;
    progress.inc(1);

    // Create Dockerfile
    progress.set_message("Creating Dockerfile...");
    create_service_dockerfile(name, &service_path, service_type, output).await?;
    progress.inc(1);

    progress.finish_with_message("Service created successfully!");
    output.success(&format!("Service {} created at {}", name, service_path))?;

    // Show next steps
    show_service_next_steps(name, &service_path, output).await?;

    Ok(())
}

async fn make_frontend(
    name: &str,
    frontend_type: &str,
    config: &SimplConfig,
    output: &SimplOutput,
) -> Result<()> {
    output.info(&format!("Creating new {} frontend: {}", frontend_type, name))?;

    let frontend_path = format!("antarmuka/{}", name);
    
    // Check if frontend already exists
    if tokio::fs::metadata(&frontend_path).await.is_ok() {
        output.error(&format!("Frontend {} already exists", name))?;
        return Ok(());
    }

    let progress = output.progress("Creating frontend...", 5)?;

    // Create frontend directory
    progress.set_message("Creating directory structure...");
    create_frontend_directory(&frontend_path, output).await?;
    progress.inc(1);

    // Create Cargo.toml
    progress.set_message("Creating Cargo.toml...");
    create_frontend_cargo_toml(name, &frontend_path, frontend_type, output).await?;
    progress.inc(1);

    // Create main.rs
    progress.set_message("Creating main.rs...");
    create_frontend_main(name, &frontend_path, frontend_type, output).await?;
    progress.inc(1);

    // Create Trunk.toml
    progress.set_message("Creating Trunk.toml...");
    create_frontend_trunk_toml(name, &frontend_path, output).await?;
    progress.inc(1);

    // Create index.html
    progress.set_message("Creating index.html...");
    create_frontend_index_html(name, &frontend_path, output).await?;
    progress.inc(1);

    progress.finish_with_message("Frontend created successfully!");
    output.success(&format!("Frontend {} created at {}", name, frontend_path))?;

    // Show next steps
    show_frontend_next_steps(name, &frontend_path, output).await?;

    Ok(())
}

async fn make_migration(
    name: &str,
    service: &str,
    config: &SimplConfig,
    output: &SimplOutput,
) -> Result<()> {
    output.info(&format!("Creating migration {} for service {}", name, service))?;

    let service_path = format!("layanan/{}", service);
    
    // Check if service exists
    if tokio::fs::metadata(&service_path).await.is_err() {
        output.error(&format!("Service {} does not exist", service))?;
        return Ok(());
    }

    let migrations_dir = format!("{}/migrations", service_path);
    tokio::fs::create_dir_all(&migrations_dir).await?;

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let migration_file = format!("{}/{}_{}.sql", migrations_dir, timestamp, name);

    let migration_content = create_migration_template(name)?;
    tokio::fs::write(&migration_file, migration_content).await?;

    output.success(&format!("Migration created: {}", migration_file))?;

    Ok(())
}

async fn make_model(
    name: &str,
    service: &str,
    config: &SimplConfig,
    output: &SimplOutput,
) -> Result<()> {
    output.info(&format!("Creating model {} for service {}", name, service))?;

    let service_path = format!("layanan/{}", service);
    let models_dir = format!("{}/src/models", service_path);
    
    // Check if service exists
    if tokio::fs::metadata(&service_path).await.is_err() {
        output.error(&format!("Service {} does not exist", service))?;
        return Ok(());
    }

    tokio::fs::create_dir_all(&models_dir).await?;

    let model_file = format!("{}/{}.rs", models_dir, name.to_lowercase());
    let model_content = create_model_template(name)?;
    
    tokio::fs::write(&model_file, model_content).await?;

    // Update models/mod.rs
    update_models_mod_file(&models_dir, name).await?;

    output.success(&format!("Model created: {}", model_file))?;

    Ok(())
}

async fn make_handler(
    name: &str,
    service: &str,
    config: &SimplConfig,
    output: &SimplOutput,
) -> Result<()> {
    output.info(&format!("Creating handler {} for service {}", name, service))?;

    let service_path = format!("layanan/{}", service);
    let handlers_dir = format!("{}/src/handlers", service_path);
    
    // Check if service exists
    if tokio::fs::metadata(&service_path).await.is_err() {
        output.error(&format!("Service {} does not exist", service))?;
        return Ok(());
    }

    tokio::fs::create_dir_all(&handlers_dir).await?;

    let handler_file = format!("{}/{}.rs", handlers_dir, name.to_lowercase());
    let handler_content = create_handler_template(name)?;
    
    tokio::fs::write(&handler_file, handler_content).await?;

    // Update handlers/mod.rs
    update_handlers_mod_file(&handlers_dir, name).await?;

    output.success(&format!("Handler created: {}", handler_file))?;

    Ok(())
}

async fn make_test(
    name: &str,
    service: &str,
    config: &SimplConfig,
    output: &SimplOutput,
) -> Result<()> {
    output.info(&format!("Creating test {} for service {}", name, service))?;

    let service_path = format!("layanan/{}", service);
    let tests_dir = format!("{}/tests", service_path);
    
    // Check if service exists
    if tokio::fs::metadata(&service_path).await.is_err() {
        output.error(&format!("Service {} does not exist", service))?;
        return Ok(());
    }

    tokio::fs::create_dir_all(&tests_dir).await?;

    let test_file = format!("{}/{}.rs", tests_dir, name.to_lowercase());
    let test_content = create_test_template(name, service)?;
    
    tokio::fs::write(&test_file, test_content).await?;

    output.success(&format!("Test created: {}", test_file))?;

    Ok(())
}

async fn create_service_directory(service_path: &str, output: &SimplOutput) -> Result<()> {
    let directories = [
        service_path,
        &format!("{}/src", service_path),
        &format!("{}/src/handlers", service_path),
        &format!("{}/src/models", service_path),
        &format!("{}/src/services", service_path),
        &format!("{}/src/utils", service_path),
        &format!("{}/tests", service_path),
        &format!("{}/migrations", service_path),
    ];

    for dir in &directories {
        tokio::fs::create_dir_all(dir).await?;
    }

    Ok(())
}

async fn create_service_cargo_toml(
    name: &str,
    service_path: &str,
    service_type: &str,
    output: &SimplOutput,
) -> Result<()> {
    let mut handlebars = Handlebars::new();
    
    let template = r#"[package]
name = "layanan-{{name}}"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "layanan-{{name}}"
path = "src/main.rs"

[dependencies]
{{#if is_web_service}}
axum = "0.7"
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace"] }
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
uuid = { version = "1.0", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1.0"
thiserror = "1.0"
{{/if}}
{{#if has_database}}
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "chrono", "uuid"] }
{{/if}}
{{#if is_ai_service}}
candle = "0.3"
tokenizers = "0.15"
{{/if}}"#;

    handlebars.register_template_string("cargo_toml", template)?;

    let is_web_service = service_type == "backend" || service_type == "api";
    let has_database = service_type == "backend" || service_type == "api";
    let is_ai_service = name.contains("ai") || service_type == "ai";

    let data = json!({
        "name": name,
        "is_web_service": is_web_service,
        "has_database": has_database,
        "is_ai_service": is_ai_service
    });

    let content = handlebars.render("cargo_toml", &data)?;
    let cargo_path = format!("{}/Cargo.toml", service_path);
    
    tokio::fs::write(&cargo_path, content).await?;

    Ok(())
}

async fn create_service_main(
    name: &str,
    service_path: &str,
    service_type: &str,
    output: &SimplOutput,
) -> Result<()> {
    let content = if service_type == "backend" || service_type == "api" {
        create_web_service_main_template(name)?
    } else {
        create_generic_service_main_template(name)?
    };

    let main_path = format!("{}/src/main.rs", service_path);
    tokio::fs::write(&main_path, content).await?;

    Ok(())
}

async fn create_service_lib(
    name: &str,
    service_path: &str,
    service_type: &str,
    output: &SimplOutput,
) -> Result<()> {
    let content = format!(
        r#"//! {} Service
//! 
//! This service provides functionality for {}.

pub mod handlers;
pub mod models;
pub mod services;
pub mod utils;

pub use handlers::*;
pub use models::*;
pub use services::*;
"#,
        name.to_uppercase(),
        name
    );

    let lib_path = format!("{}/src/lib.rs", service_path);
    tokio::fs::write(&lib_path, content).await?;

    Ok(())
}

async fn create_service_handlers(
    name: &str,
    service_path: &str,
    service_type: &str,
    output: &SimplOutput,
) -> Result<()> {
    // Create mod.rs
    let mod_content = r#"pub mod health;

pub use health::*;
"#;
    let mod_path = format!("{}/src/handlers/mod.rs", service_path);
    tokio::fs::write(&mod_path, mod_content).await?;

    // Create health handler
    let health_content = create_health_handler_template()?;
    let health_path = format!("{}/src/handlers/health.rs", service_path);
    tokio::fs::write(&health_path, health_content).await?;

    Ok(())
}

async fn create_service_dockerfile(
    name: &str,
    service_path: &str,
    service_type: &str,
    output: &SimplOutput,
) -> Result<()> {
    let content = format!(
        r#"FROM rust:1.75 as builder

WORKDIR /app
COPY . .
RUN cargo build --release --bin layanan-{}

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/layanan-{} /usr/local/bin/layanan-{}

EXPOSE 3000

CMD ["layanan-{}"]
"#,
        name, name, name, name
    );

    let dockerfile_path = format!("{}/Dockerfile", service_path);
    tokio::fs::write(&dockerfile_path, content).await?;

    Ok(())
}

async fn create_frontend_directory(frontend_path: &str, output: &SimplOutput) -> Result<()> {
    let directories = [
        frontend_path,
        &format!("{}/src", frontend_path),
        &format!("{}/src/components", frontend_path),
        &format!("{}/src/pages", frontend_path),
        &format!("{}/styles", frontend_path),
        &format!("{}/static", frontend_path),
    ];

    for dir in &directories {
        tokio::fs::create_dir_all(dir).await?;
    }

    Ok(())
}

async fn create_frontend_cargo_toml(
    name: &str,
    frontend_path: &str,
    frontend_type: &str,
    output: &SimplOutput,
) -> Result<()> {
    let content = format!(
        r#"[package]
name = "antarmuka-{}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
leptos = {{ version = "0.6", features = ["csr"] }}
leptos_router = {{ version = "0.6", features = ["csr"] }}
leptos_meta = {{ version = "0.6", features = ["csr"] }}
wasm-bindgen = "0.2"
console_error_panic_hook = "0.1"
console_log = "1.0"
log = "0.4"
serde = {{ version = "1.0", features = ["derive"] }}
serde-wasm-bindgen = "0.6"
gloo-net = "0.5"
web-sys = "0.3"
"#,
        name
    );

    let cargo_path = format!("{}/Cargo.toml", frontend_path);
    tokio::fs::write(&cargo_path, content).await?;

    Ok(())
}

async fn create_frontend_main(
    name: &str,
    frontend_path: &str,
    frontend_type: &str,
    output: &SimplOutput,
) -> Result<()> {
    let content = create_leptos_main_template(name)?;
    let main_path = format!("{}/src/main.rs", frontend_path);
    tokio::fs::write(&main_path, content).await?;

    Ok(())
}

async fn create_frontend_trunk_toml(
    name: &str,
    frontend_path: &str,
    output: &SimplOutput,
) -> Result<()> {
    let content = format!(
        r#"[build]
target = "index.html"
dist = "dist"

[watch]
watch = ["src"]

[serve]
address = "0.0.0.0"
port = 8080
open = false
"#
    );

    let trunk_path = format!("{}/Trunk.toml", frontend_path);
    tokio::fs::write(&trunk_path, content).await?;

    Ok(())
}

async fn create_frontend_index_html(
    name: &str,
    frontend_path: &str,
    output: &SimplOutput,
) -> Result<()> {
    let content = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>{} - SIMPelv2</title>
    <link data-trunk rel="css" href="styles/main.css">
</head>
<body>
    <div id="app"></div>
</body>
</html>
"#,
        name.to_uppercase()
    );

    let index_path = format!("{}/index.html", frontend_path);
    tokio::fs::write(&index_path, content).await?;

    // Create basic CSS
    let css_content = r#"/* Main styles for the frontend */
body {
    margin: 0;
    padding: 0;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
}

#app {
    min-height: 100vh;
}
"#;
    let css_path = format!("{}/styles/main.css", frontend_path);
    tokio::fs::write(&css_path, css_content).await?;

    Ok(())
}

// Template functions
fn create_web_service_main_template(name: &str) -> Result<String> {
    Ok(format!(
        r#"use axum::{{
    routing::{{get, post}},
    Router,
}};
use tower_http::cors::CorsLayer;
use tracing_subscriber;

mod handlers;
mod models;
mod services;

#[tokio::main]
async fn main() -> anyhow::Result<()> {{
    tracing_subscriber::init();

    let app = Router::new()
        .route("/health", get(handlers::health_check))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    
    tracing::info!("Server starting on http://0.0.0.0:3000");
    
    axum::serve(listener, app).await?;
    
    Ok(())
}}
"#
    ))
}

fn create_generic_service_main_template(name: &str) -> Result<String> {
    Ok(format!(
        r#"use tracing_subscriber;

mod services;

#[tokio::main]
async fn main() -> anyhow::Result<()> {{
    tracing_subscriber::init();

    tracing::info!("{} service starting...");

    // Add your service logic here

    Ok(())
}}
"#,
        name.to_uppercase()
    ))
}

fn create_health_handler_template() -> Result<String> {
    Ok(r#"use axum::{http::StatusCode, Json};
use serde_json::{json, Value};

pub async fn health_check() -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "status": "ok",
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}
"#.to_string())
}

fn create_leptos_main_template(name: &str) -> Result<String> {
    Ok(format!(
        r#"use leptos::*;
use leptos_router::*;

#[component]
fn App() -> impl IntoView {{
    view! {{
        <Router>
            <Routes>
                <Route path="/" view=HomePage/>
            </Routes>
        </Router>
    }}
}}

#[component]
fn HomePage() -> impl IntoView {{
    view! {{
        <div class="container">
            <h1>"{} - SIMPelv2"</h1>
            <p>"Welcome to the {} module of SIMPelv2"</p>
        </div>
    }}
}}

fn main() {{
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).expect("error initializing logger");

    mount_to_body(|| view! {{ <App/> }})
}}
"#,
        name.to_uppercase(),
        name
    ))
}

fn create_migration_template(name: &str) -> Result<String> {
    Ok(format!(
        r#"-- Migration: {}
-- Created at: {}

-- Up
-- Add your migration SQL here

-- Down
-- Add your rollback SQL here
"#,
        name,
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    ))
}

fn create_model_template(name: &str) -> Result<String> {
    Ok(format!(
        r#"use serde::{{Deserialize, Serialize}};
use uuid::Uuid;
use chrono::{{DateTime, Utc}};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {} {{
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Add your fields here
}}

impl {} {{
    pub fn new() -> Self {{
        let now = Utc::now();
        Self {{
            id: Uuid::new_v4(),
            created_at: now,
            updated_at: now,
        }}
    }}
}}
"#,
        name,
        name
    ))
}

fn create_handler_template(name: &str) -> Result<String> {
    Ok(format!(
        r#"use axum::{{
    extract::{{Path, Query}},
    http::StatusCode,
    Json,
}};
use serde::{{Deserialize, Serialize}};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct {}Query {{
    // Add query parameters here
}}

#[derive(Debug, Serialize)]
pub struct {}Response {{
    // Add response fields here
}}

pub async fn get_{}(
    Query(query): Query<{}Query>,
) -> Result<Json<{}Response>, StatusCode> {{
    // Add your handler logic here
    
    let response = {}Response {{}};
    
    Ok(Json(response))
}}

pub async fn create_{}(
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<{}Response>, StatusCode> {{
    // Add your create logic here
    
    let response = {}Response {{}};
    
    Ok(Json(response))
}}
"#,
        name, name, name, name, name, name, name, name, name
    ))
}

fn create_test_template(name: &str, service: &str) -> Result<String> {
    Ok(format!(
        r#"use layanan_{}::*;

#[tokio::test]
async fn test_{}() {{
    // Add your test here
    assert_eq!(2 + 2, 4);
}}

#[tokio::test]
async fn test_{}_integration() {{
    // Add integration test here
}}
"#,
        service, name, name
    ))
}

async fn update_models_mod_file(models_dir: &str, name: &str) -> Result<()> {
    let mod_file = format!("{}/mod.rs", models_dir);
    let module_line = format!("pub mod {};\n", name.to_lowercase());
    
    if tokio::fs::metadata(&mod_file).await.is_ok() {
        let mut content = tokio::fs::read_to_string(&mod_file).await?;
        if !content.contains(&module_line) {
            content.push_str(&module_line);
            tokio::fs::write(&mod_file, content).await?;
        }
    } else {
        tokio::fs::write(&mod_file, module_line).await?;
    }
    
    Ok(())
}

async fn update_handlers_mod_file(handlers_dir: &str, name: &str) -> Result<()> {
    let mod_file = format!("{}/mod.rs", handlers_dir);
    let module_line = format!("pub mod {};\n", name.to_lowercase());
    
    if tokio::fs::metadata(&mod_file).await.is_ok() {
        let mut content = tokio::fs::read_to_string(&mod_file).await?;
        if !content.contains(&module_line) {
            content.push_str(&module_line);
            tokio::fs::write(&mod_file, content).await?;
        }
    } else {
        tokio::fs::write(&mod_file, module_line).await?;
    }
    
    Ok(())
}

async fn show_service_next_steps(name: &str, service_path: &str, output: &SimplOutput) -> Result<()> {
    output.info("Next steps:")?;
    println!("  1. Add your service to the workspace Cargo.toml");
    println!("  2. Update docker-compose.yml with your service configuration");
    println!("  3. Create Kubernetes manifests in infra/k8s/{}.yaml", name);
    println!("  4. Add service configuration to your environment files");
    println!("  5. Build and test your service:");
    println!("     simpel build backend --service {}", name);
    println!("     simpel test service {}", name);
    
    Ok(())
}

async fn show_frontend_next_steps(name: &str, frontend_path: &str, output: &SimplOutput) -> Result<()> {
    output.info("Next steps:")?;
    println!("  1. Add your frontend to the workspace Cargo.toml");
    println!("  2. Configure your frontend port in the configuration");
    println!("  3. Start development server:");
    println!("     simpel dev up --services {}", name);
    println!("  4. Build your frontend:");
    println!("     simpel build frontend --frontend {}", name);
    
    Ok(())
}
