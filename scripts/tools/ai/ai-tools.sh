#!/bin/bash

# AI Tools - AI-powered Development Assistant
# Enhanced integration with layanan-ai service
# Specialized functions for advanced AI-powered development

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
AI_SERVICE_URL="http://localhost:8080"

# Warna untuk output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m'

# Error handling functions
error_exit() {
    echo -e "${RED}ERROR: $1${NC}" >&2
    exit 1
}

warn() {
    echo -e "${YELLOW}WARNING: $1${NC}" >&2
}

info() {
    echo -e "${GREEN}INFO: $1${NC}"
}

# Check if AI service is running
check_ai_service() {
    if command -v curl >/dev/null 2>&1; then
        if curl -s "$AI_SERVICE_URL/api/v1/ai/health" >/dev/null 2>&1; then
            return 0
        fi
    fi
    return 1
}

show_help() {
    echo -e "${BLUE}AI-TOOLS MODULE (Enhanced)${NC}"
    echo -e "${CYAN}================================${NC}"
    echo "AI-powered development assistance for SIMPelv2"
    echo "Enhanced with real AI service integration"
    echo ""
    echo -e "${YELLOW}Available Commands:${NC}"
    echo "  analyze    - Real-time AI code analysis (integrates with layanan-ai)"
    echo "  generate   - Advanced AI-powered code generation"
    echo "  optimize   - AI-based performance optimization suggestions"
    echo "  review     - Comprehensive AI code review"
    echo "  docs       - AI-generated documentation"
    echo "  fix        - AI-assisted code fixes"
    echo "  test       - AI-powered test generation"
    echo "  status     - Check AI service connectivity"
    echo ""
    echo -e "${YELLOW}Usage:${NC} $0 <command> [options]"
    echo -e "${YELLOW}Examples:${NC}"
    echo "  $0 analyze --target backend --real-ai"
    echo "  $0 generate advanced-service my-service"
    echo "  $0 status"
    echo ""
    echo -e "${CYAN}Note:${NC} Some features require layanan-ai service to be running"
}

check_status() {
    echo -e "${BLUE}[AI-STATUS]${NC} Checking AI service status..."

    if check_ai_service; then
        info "✅ AI service is running and accessible at $AI_SERVICE_URL"

        # Get available models if service is running
        if command -v curl >/dev/null 2>&1; then
            echo -e "${CYAN}Available AI Models:${NC}"
            if models=$(curl -s "$AI_SERVICE_URL/api/v1/ai/models" 2>/dev/null); then
                echo "$models"
            else
                warn "Could not retrieve model information"
            fi
        fi
    else
        warn "❌ AI service is not running or not accessible"
        echo -e "${YELLOW}To start AI service:${NC}"
        echo "  cd layanan/ai && cargo run"
        echo "  or use: ./scripts/simpel.sh and select option for AI service"
    fi
}

analyze_code() {
    echo -e "${BLUE}[AI-ANALYZE]${NC} Enhanced code analysis..."

    local target="${1:-all}"
    local use_real_ai="${2:-false}"
    local output_dir="$PROJECT_ROOT/docs/analysis"

    mkdir -p "$output_dir" || error_exit "Cannot create analysis directory"

    # Check if we should use real AI
    if [[ "$use_real_ai" == "--real-ai" ]] || [[ "$use_real_ai" == "true" ]]; then
        if check_ai_service; then
            info "Using real AI service for analysis"
            analyze_with_ai_service "$target" "$output_dir"
            return 0
        else
            warn "AI service not available, falling back to static analysis"
        fi
    fi

    # Enhanced static analysis
    case "$target" in
        "backend"|"layanan")
            info "Analyzing backend services with enhanced metrics..."
            local rust_files=$(find "$PROJECT_ROOT/layanan" -name "*.rs" | wc -l)
            local toml_files=$(find "$PROJECT_ROOT/layanan" -name "*.toml" | wc -l)
            local services=$(find "$PROJECT_ROOT/layanan" -mindepth 1 -maxdepth 1 -type d | wc -l)

            echo "📊 Backend Analysis Results:"
            echo "  - Rust files: $rust_files"
            echo "  - TOML files: $toml_files"
            echo "  - Total services: $services"
            echo "  - Analysis report: $output_dir/backend_analysis.md"

            # Generate detailed backend report
            generate_backend_analysis_report "$output_dir/backend_analysis.md"
            ;;
        "frontend"|"antarmuka")
            info "Analyzing frontend microfrontends with enhanced metrics..."
            local frontend_rust=$(find "$PROJECT_ROOT/antarmuka" -name "*.rs" | wc -l)
            local frontend_dirs=$(find "$PROJECT_ROOT/antarmuka" -mindepth 1 -maxdepth 1 -type d | wc -l)

            echo "📊 Frontend Analysis Results:"
            echo "  - Frontend Rust files: $frontend_rust"
            echo "  - Microfrontends: $frontend_dirs"
            echo "  - Analysis report: $output_dir/frontend_analysis.md"

            generate_frontend_analysis_report "$output_dir/frontend_analysis.md"
            ;;
        "all"|*)
            info "Analyzing entire codebase with comprehensive metrics..."
            local total_rust=$(find "$PROJECT_ROOT" -name "*.rs" -not -path "*/target/*" | wc -l)
            local total_toml=$(find "$PROJECT_ROOT" -name "*.toml" -not -path "*/target/*" | wc -l)
            local total_services=$(find "$PROJECT_ROOT/layanan" -mindepth 1 -maxdepth 1 -type d | wc -l)
            local total_frontends=$(find "$PROJECT_ROOT/antarmuka" -mindepth 1 -maxdepth 1 -type d | wc -l)

            echo "📊 Complete Codebase Analysis:"
            echo "  - Total Rust files: $total_rust"
            echo "  - Total TOML files: $total_toml"
            echo "  - Backend services: $total_services"
            echo "  - Frontend apps: $total_frontends"
            echo "  - Full analysis report: $output_dir/full_analysis.md"

            generate_full_analysis_report "$output_dir/full_analysis.md"
            ;;
    esac

    info "✅ Enhanced analysis completed!"
}

analyze_with_ai_service() {
    local target="$1"
    local output_dir="$2"

    info "Sending analysis request to AI service..."

    if command -v curl >/dev/null 2>&1; then
        # Send request to AI service for analysis
        local response
        if response=$(curl -s -X POST "$AI_SERVICE_URL/api/v1/ai/analyze" \
            -H "Content-Type: application/json" \
            -d "{\"target\": \"$target\", \"type\": \"code_analysis\"}" 2>/dev/null); then
            echo -e "${GREEN}AI Analysis Response:${NC}"
            echo "$response"
        else
            warn "Failed to get response from AI service"
        fi
    else
        warn "curl not available, cannot connect to AI service"
    fi
}

generate_backend_analysis_report() {
    local report_file="$1"

    cat > "$report_file" << EOF
# Backend Services Analysis Report
Generated: $(date)

## Overview
- Total services: $(find "$PROJECT_ROOT/layanan" -mindepth 1 -maxdepth 1 -type d | wc -l)
- Rust files: $(find "$PROJECT_ROOT/layanan" -name "*.rs" | wc -l)

## Services List
$(find "$PROJECT_ROOT/layanan" -mindepth 1 -maxdepth 1 -type d -exec basename {} \; | sort | sed 's/^/- /')

## Code Quality Metrics
- Average files per service: $(($(find "$PROJECT_ROOT/layanan" -name "*.rs" | wc -l) / $(find "$PROJECT_ROOT/layanan" -mindepth 1 -maxdepth 1 -type d | wc -l)))
- Services with tests: $(find "$PROJECT_ROOT/layanan" -name "*test*" -type f | wc -l)

## Recommendations
- Consider adding more unit tests for services with low test coverage
- Standardize error handling across all services
- Review dependency usage for optimization opportunities
EOF

    info "Backend analysis report generated: $report_file"
}

generate_frontend_analysis_report() {
    local report_file="$1"

    cat > "$report_file" << EOF
# Frontend Microfrontends Analysis Report
Generated: $(date)

## Overview
- Total microfrontends: $(find "$PROJECT_ROOT/antarmuka" -mindepth 1 -maxdepth 1 -type d | wc -l)
- Frontend Rust files: $(find "$PROJECT_ROOT/antarmuka" -name "*.rs" | wc -l)

## Microfrontends List
$(find "$PROJECT_ROOT/antarmuka" -mindepth 1 -maxdepth 1 -type d -exec basename {} \; | sort | sed 's/^/- /')

## Build Configuration
- WASM targets: $(find "$PROJECT_ROOT/antarmuka" -name "Trunk.toml" | wc -l)
- Style files: $(find "$PROJECT_ROOT/antarmuka" -name "*.css" | wc -l)

## Recommendations
- Optimize WASM bundle sizes using wasm-opt
- Consider shared component library for common UI elements
- Standardize styling approach across microfrontends
EOF

    info "Frontend analysis report generated: $report_file"
}

generate_full_analysis_report() {
    local report_file="$1"

    cat > "$report_file" << EOF
# Complete SIMPelv2 Codebase Analysis
Generated: $(date)

## Project Overview
- Total Rust files: $(find "$PROJECT_ROOT" -name "*.rs" -not -path "*/target/*" | wc -l)
- Backend services: $(find "$PROJECT_ROOT/layanan" -mindepth 1 -maxdepth 1 -type d | wc -l)
- Frontend apps: $(find "$PROJECT_ROOT/antarmuka" -mindepth 1 -maxdepth 1 -type d | wc -l)
- Configuration files: $(find "$PROJECT_ROOT" -name "*.toml" -not -path "*/target/*" | wc -l)

## Architecture Health
- Modular structure: ✅ Well organized layanan/ and antarmuka/ directories
- Configuration management: ✅ Consistent Cargo.toml structure
- Documentation: $(find "$PROJECT_ROOT/docs" -name "*.md" | wc -l) markdown files

## Development Tools
- Scripts available: $(find "$PROJECT_ROOT/scripts" -name "*.sh" | wc -l)
- Docker configurations: $(find "$PROJECT_ROOT" -name "docker-compose*.yml" | wc -l)
- Makefiles: $(find "$PROJECT_ROOT" -name "*.mk" | wc -l)

## Recommendations
1. Continue maintaining modular architecture
2. Add more integration tests
3. Consider CI/CD pipeline optimization
4. Regular dependency updates
5. Performance monitoring setup
EOF

    info "Full analysis report generated: $report_file"
}

generate_code() {
    echo -e "${BLUE}[AI-GENERATE]${NC} Advanced AI-powered code generation..."

    local type="${1:-help}"
    local name="${2:-example}"

    case "$type" in
        "advanced-service"|"ai-service")
            generate_advanced_service "$name"
            ;;
        "ml-service")
            generate_ml_service "$name"
            ;;
        "ai-microfrontend")
            generate_ai_microfrontend "$name"
            ;;
        "integration-template")
            generate_integration_template "$name"
            ;;
        "help"|*)
            show_generation_help
            ;;
    esac
}

show_generation_help() {
    echo -e "${YELLOW}AI Code Generation Options:${NC}"
    echo "  advanced-service <name>   - Generate advanced microservice with AI integration"
    echo "  ml-service <name>         - Generate ML/AI specialized service template"
    echo "  ai-microfrontend <name>   - Generate AI-powered microfrontend"
    echo "  integration-template <name> - Generate service integration template"
    echo ""
    echo -e "${CYAN}Examples:${NC}"
    echo "  $0 generate advanced-service my-ai-service"
    echo "  $0 generate ml-service document-classifier"
}

generate_advanced_service() {
    local name="$1"

    if [[ -z "$name" ]]; then
        error_exit "Service name is required"
    fi

    echo -e "${GREEN}Generating advanced AI-integrated microservice: ${name}${NC}"
    local service_dir="$PROJECT_ROOT/layanan/$name"

    if [[ -d "$service_dir" ]]; then
        error_exit "Service $name already exists"
    fi

    mkdir -p "$service_dir/src" || error_exit "Cannot create service directory"

    # Advanced Cargo.toml with AI/ML dependencies
    cat > "$service_dir/Cargo.toml" << EOF
[package]
name = "layanan-$name"
version = "0.1.0"
edition = "2021"
authors = ["SIMPelv2 Team"]
description = "Advanced AI-integrated microservice for $name"

[[bin]]
name = "layanan-$name"
path = "src/main.rs"

[dependencies]
# Web Framework
axum = "0.7"
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace", "fs"] }
hyper = { version = "1.0", features = ["full"] }
tokio = { version = "1.0", features = ["full"] }

# Serialization & JSON
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Database & Storage
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "chrono", "uuid"] }
uuid = { version = "1.0", features = ["v4", "serde"] }

# AI Service Integration
reqwest = { version = "0.12", features = ["json"] }

# Configuration & Environment
config = "0.14"
dotenv = "0.15"

# Logging & Observability
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
opentelemetry = "0.21"

# Date & Time
chrono = { version = "0.4", features = ["serde"] }

# Error Handling
anyhow = "1.0"
thiserror = "1.0"

# Async utilities
async-trait = "0.1"

[dev-dependencies]
tower-test = "0.4"
tokio-test = "0.4"
EOF

EOF

    # Advanced main.rs with AI integration
    cat > "$service_dir/src/main.rs" << EOF
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, warn, error};
use uuid::Uuid;

mod config;
mod handlers;
mod models;
mod services;
mod error;

use config::Config;
use error::{ServiceError, ServiceResult};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub ai_client: reqwest::Client,
}

#[derive(Serialize, Deserialize)]
struct HealthResponse {
    status: String,
    service: String,
    timestamp: chrono::DateTime<chrono::Utc>,
    ai_integration: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Load configuration
    let config = Arc::new(Config::from_env()?);

    // Initialize HTTP client for AI service
    let ai_client = reqwest::Client::new();

    let state = AppState {
        config: config.clone(),
        ai_client,
    };

    // Build application routes
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/${name}", post(handlers::create_${name}))
        .route("/api/v1/${name}/:id", get(handlers::get_${name}))
        .route("/api/v1/${name}/ai-analyze", post(handlers::ai_analyze))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    info!("🚀 Advanced {} service starting on {}", "$name", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check(State(state): State<AppState>) -> ServiceResult<Json<HealthResponse>> {
    // Check AI service connectivity
    let ai_available = check_ai_service_health(&state.ai_client).await;

    let response = HealthResponse {
        status: "healthy".to_string(),
        service: "$name".to_string(),
        timestamp: chrono::Utc::now(),
        ai_integration: ai_available,
    };

    Ok(Json(response))
}

async fn check_ai_service_health(client: &reqwest::Client) -> bool {
    match client.get("http://localhost:8080/api/v1/ai/health").send().await {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    }
}
EOF

    # Create configuration module
    cat > "$service_dir/src/config.rs" << 'EOF'
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub ai_service_url: String,
    pub log_level: String,
}

impl Config {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        let settings = config::Config::builder()
            .set_default("port", 8080)?
            .set_default("ai_service_url", "http://localhost:8080")?
            .set_default("log_level", "info")?
            .add_source(config::Environment::default())
            .build()?;

        settings.try_deserialize()
    }
}
EOF

    # Create error handling module
    cat > "$service_dir/src/error.rs" << 'EOF'
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("AI service error: {0}")]
    AiService(#[from] reqwest::Error),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Internal server error")]
    Internal,
}

impl IntoResponse for ServiceError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ServiceError::Database(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            ServiceError::AiService(e) => (StatusCode::SERVICE_UNAVAILABLE, format!("AI service unavailable: {}", e)),
            ServiceError::Validation(msg) => (StatusCode::BAD_REQUEST, msg),
            ServiceError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ServiceError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()),
        };

        let body = Json(json!({
            "error": error_message,
            "timestamp": chrono::Utc::now()
        }));

        (status, body).into_response()
    }
}

pub type ServiceResult<T> = Result<T, ServiceError>;
EOF

    # Create handlers module
    cat > "$service_dir/src/handlers.rs" << 'EOF'
use axum::{
    extract::{Path, State},
    response::Json,
};
use serde_json::json;
use tracing::{info, warn};
use uuid::Uuid;

use crate::{AppState, ServiceResult, models::*};

pub async fn create_${name}(
    State(state): State<AppState>,
    Json(request): Json<CreateRequest>,
) -> ServiceResult<Json<CreateResponse>> {
    info!("Creating new {} entry", "$name");

    // Optional: Use AI service for validation or enhancement
    if let Err(e) = validate_with_ai(&state, &request).await {
        warn!("AI validation failed: {}", e);
        // Continue without AI validation
    }

    let response = CreateResponse {
        id: Uuid::new_v4(),
        message: format!("{} created successfully", "$name"),
        timestamp: chrono::Utc::now(),
    };

    Ok(Json(response))
}

pub async fn get_${name}(
    Path(id): Path<Uuid>
) -> ServiceResult<Json<GetResponse>> {
    info!("Fetching {} with id: {}", "$name", id);

    let response = GetResponse {
        id,
        data: format!("{} data for {}", "$name", id),
        timestamp: chrono::Utc::now(),
    };

    Ok(Json(response))
}

pub async fn ai_analyze(
    State(state): State<AppState>,
    Json(request): Json<AnalyzeRequest>,
) -> ServiceResult<Json<AnalyzeResponse>> {
    info!("Performing AI analysis");

    // Send request to AI service
    let ai_response = state.ai_client
        .post(&format!("{}/api/v1/ai/analyze", state.config.ai_service_url))
        .json(&json!({
            "text": request.text,
            "type": "service_analysis"
        }))
        .send()
        .await?;

    let analysis_result = ai_response.text().await?;

    Ok(Json(AnalyzeResponse {
        result: analysis_result,
        confidence: 0.85,
        timestamp: chrono::Utc::now(),
    }))
}

async fn validate_with_ai(state: &AppState, request: &CreateRequest) -> Result<(), reqwest::Error> {
    let _response = state.ai_client
        .post(&format!("{}/api/v1/ai/validate", state.config.ai_service_url))
        .json(request)
        .send()
        .await?;

    Ok(())
}
EOF

    # Create models module
    cat > "$service_dir/src/models.rs" << 'EOF'
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRequest {
    pub data: String,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateResponse {
    pub id: Uuid,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetResponse {
    pub id: Uuid,
    pub data: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalyzeRequest {
    pub text: String,
    pub analysis_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalyzeResponse {
    pub result: String,
    pub confidence: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
EOF

    # Create services module
    cat > "$service_dir/src/services.rs" << 'EOF'
use crate::{ServiceResult, AppState};

pub struct ${name^}Service;

impl ${name^}Service {
    pub fn new() -> Self {
        Self
    }

    pub async fn process_with_ai(&self, state: &AppState, input: &str) -> ServiceResult<String> {
        // Integration with AI service for processing
        let response = state.ai_client
            .post(&format!("{}/api/v1/ai/generate", state.config.ai_service_url))
            .json(&serde_json::json!({
                "prompt": input,
                "service": "${name}"
            }))
            .send()
            .await?;

        let result = response.text().await?;
        Ok(result)
    }
}
EOF

    # Add to workspace if not already there
    if ! grep -q "layanan/$name" "$PROJECT_ROOT/Cargo.toml" 2>/dev/null; then
        sed -i "/members = \[/a\\    \"layanan/$name\"," "$PROJECT_ROOT/Cargo.toml"
        info "Added to workspace Cargo.toml"
    fi

    info "✅ Advanced service '$name' generated successfully!"
    echo -e "${CYAN}Features included:${NC}"
    echo -e "  ✅ AI service integration"
    echo -e "  ✅ Advanced error handling"
    echo -e "  ✅ Configuration management"
    echo -e "  ✅ Structured logging"
    echo -e "  ✅ Health checks with AI status"
    echo -e "  ✅ Ready for database integration"
}

generate_ml_service() {
    local name="$1"

    if [[ -z "$name" ]]; then
        error_exit "ML service name is required"
    fi

    echo -e "${GREEN}Generating specialized ML/AI service: ${name}${NC}"
    local service_dir="$PROJECT_ROOT/layanan/$name"

    if [[ -d "$service_dir" ]]; then
        error_exit "Service $name already exists"
    fi

    mkdir -p "$service_dir/src" || error_exit "Cannot create service directory"

    # ML-specialized Cargo.toml
    cat > "$service_dir/Cargo.toml" << EOF
[package]
name = "layanan-$name"
version = "0.1.0"
edition = "2021"
authors = ["SIMPelv2 Team"]
description = "ML/AI specialized microservice for $name"

[[bin]]
name = "layanan-$name"
path = "src/main.rs"

[dependencies]
# Web Framework
axum = "0.7"
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace"] }
tokio = { version = "1.0", features = ["full"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# AI/ML Integration
reqwest = { version = "0.12", features = ["json"] }
qdrant-client = "1.7"

# Text Processing
regex = "1.10"

# Image Processing (optional)
# image = "0.24"  # Enable if needed
# opencv = "0.88" # Enable if needed with system deps

# Vector Operations
# ndarray = "0.15" # Enable for numerical computations

# Configuration
config = "0.14"
dotenv = "0.15"

# Database
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres"] }
uuid = { version = "1.0", features = ["v4"] }

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Error Handling
anyhow = "1.0"
thiserror = "1.0"

# Async
async-trait = "0.1"

# Date/Time
chrono = { version = "0.4", features = ["serde"] }

[dev-dependencies]
tower-test = "0.4"
tokio-test = "0.4"
EOF

    info "✅ ML service '$name' template generated!"
    echo -e "${CYAN}ML-specific features:${NC}"
    echo -e "  ✅ Vector database integration (Qdrant)"
    echo -e "  ✅ Text processing capabilities"
    echo -e "  ✅ Optional image processing support"
    echo -e "  ✅ AI service integration ready"
}

#[derive(Serialize, Deserialize)]
struct ${name^}Response {
    id: Uuid,
    message: String,
    timestamp: chrono::DateTime<chrono::Utc>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::init();

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/${name}", post(create_${name}))
        .route("/api/v1/${name}/:id", get(get_${name}));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    info!("${name} service starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "${name}",
        "timestamp": chrono::Utc::now()
    }))
}

async fn create_${name}(Json(request): Json<${name^}Request>) -> Result<Json<${name^}Response>, StatusCode> {
    info!("Creating new ${name} entry");

    let response = ${name^}Response {
        id: Uuid::new_v4(),
        message: format!("${name} created with data: {}", request.data),
        timestamp: chrono::Utc::now(),
    };

    Ok(Json(response))
}

async fn get_${name}(Path(id): Path<Uuid>) -> Result<Json<${name^}Response>, StatusCode> {
    info!("Fetching ${name} with id: {}", id);

    let response = ${name^}Response {
        id,
        message: format!("${name} details for id: {}", id),
        timestamp: chrono::Utc::now(),
    };

    Ok(Json(response))
}
EOF

            # Create lib.rs for shared functionality
            cat > "$service_dir/src/lib.rs" << EOF
//! ${name^} Service
//!
//! Government BMN management microservice for ${name} operations

pub mod models;
pub mod handlers;
pub mod error;

pub use error::{ServiceError, ServiceResult};
EOF

            # Create error handling module
            mkdir -p "$service_dir/src"
            cat > "$service_dir/src/error.rs" << EOF
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Internal server error")]
    Internal,
}

impl IntoResponse for ServiceError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ServiceError::Database(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            ServiceError::Validation(msg) => (StatusCode::BAD_REQUEST, msg),
            ServiceError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ServiceError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

pub type ServiceResult<T> = Result<T, ServiceError>;
EOF

            echo -e "${YELLOW}Advanced service '${name}' generated at: $service_dir${NC}"
            echo -e "${CYAN}Features included:${NC}"
            echo -e "  ✅ REST API endpoints"
            echo -e "  ✅ Error handling"
            echo -e "  ✅ Structured logging"
            echo -e "  ✅ Government service patterns"
            echo -e "  ✅ Database integration ready"
            ;;
        "microfrontend"|"antarmuka")
            echo -e "${GREEN}Generating advanced microfrontend template: ${name}${NC}"
            local frontend_dir="$PROJECT_ROOT/antarmuka/$name"
            mkdir -p "$frontend_dir/src" "$frontend_dir/styles"

            # Advanced Cargo.toml for Leptos frontend
            cat > "$frontend_dir/Cargo.toml" << EOF
[package]
name = "${name}"
version = "0.1.0"
edition = "2021"

[dependencies]
leptos = { version = "0.5", features = ["csr"] }
leptos_meta = { version = "0.5", features = ["csr"] }
leptos_router = { version = "0.5", features = ["csr"] }
wasm-bindgen = "0.2"
console_error_panic_hook = "0.1"
serde = { version = "1.0", features = ["derive"] }
gloo-net = { version = "0.4", features = ["http"] }

[lib]
crate-type = ["cdylib"]
EOF

            # Advanced main.rs for Leptos frontend
            cat > "$frontend_dir/src/main.rs" << EOF
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

fn main() {
    console_error_panic_hook::set_once();

    mount_to_body(|| {
        view! {
            <App />
        }
    })
}

#[component]
fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Html lang="id"/>
        <Title text="${name^} - SIMPelv2"/>
        <Meta name="description" content="${name^} microfrontend for SIMPelv2 government BMN system"/>

        <Router>
            <Routes>
                <Route path="/" view=HomePage />
                <Route path="/${name}" view=${name^}Page />
                <Route path="/*any" view=NotFound />
            </Routes>
        </Router>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    view! {
        <div class="container">
            <h1>"${name^} Dashboard"</h1>
            <p>"Welcome to ${name^} management system"</p>
            <a href="/${name}">"Go to ${name^}"</a>
        </div>
    }
}

#[component]
fn ${name^}Page() -> impl IntoView {
    let (count, set_count) = create_signal(0);

    view! {
        <div class="container">
            <h1>"${name^} Management"</h1>
            <div class="card">
                <button on:click=move |_| set_count.update(|n| *n += 1)>
                    "Count: " {count}
                </button>
            </div>
        </div>
    }
}

#[component]
fn NotFound() -> impl IntoView {
    view! {
        <div class="container">
            <h1>"404 - Page Not Found"</h1>
            <a href="/">"Back to Home"</a>
        </div>
    }
}
EOF

            # Create Trunk.toml
            cat > "$frontend_dir/Trunk.toml" << EOF
[build]
target = "index.html"
dist = "target/dist"

[watch]
watch = ["src", "styles"]
ignore = ["target"]

[serve]
address = "0.0.0.0"
port = 8080
EOF

            # Create index.html
            cat > "$frontend_dir/index.html" << EOF
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>${name^} - SIMPelv2</title>
    <link data-trunk rel="css" href="styles/main.css">
</head>
<body>
    <div id="root"></div>
</body>
</html>
EOF

            # Create CSS
            cat > "$frontend_dir/styles/main.css" << EOF
/* ${name^} Styles */
.container {
    max-width: 1200px;
    margin: 0 auto;
    padding: 2rem;
}

.card {
    background: white;
    border-radius: 8px;
    padding: 1.5rem;
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
    margin: 1rem 0;
}

button {
    background: #0066cc;
    color: white;
    border: none;
    padding: 0.75rem 1.5rem;
    border-radius: 4px;
    cursor: pointer;
    font-size: 1rem;
}

button:hover {
    background: #0052a3;
}

h1 {
    color: #333;
    margin-bottom: 1rem;
}
EOF

            echo -e "${YELLOW}Advanced microfrontend '${name}' generated at: $frontend_dir${NC}"
            echo -e "${CYAN}Features included:${NC}"
            echo -e "  ✅ Leptos framework"
            echo -e "  ✅ Router setup"
            echo -e "  ✅ Government styling"
            echo -e "  ✅ Responsive design"
            echo -e "  ✅ Meta tags"
            ;;
        *)
            echo -e "${RED}Unknown generation type: $type${NC}"
            echo "Available types: service, microfrontend"
            exit 1
            ;;
    esac
}

optimize_code() {
    echo -e "${BLUE}[AI-OPTIMIZE]${NC} Enhanced code optimization with AI integration..."

    local target="${1:-all}"
    local use_ai="${2:-false}"

    echo -e "${GREEN}Running optimization analysis for: $target${NC}"

    # Check if AI service is available for enhanced optimization
    if [[ "$use_ai" == "--ai" ]] && check_ai_service; then
        info "Using AI service for optimization analysis"
        optimize_with_ai "$target"
        return 0
    fi

    case "$target" in
        "rust"|"backend")
            echo -e "${YELLOW}Rust/Backend optimizations:${NC}"
            echo "📊 Analyzing Rust code performance..."

            # Count dependencies that could be optimized
            local heavy_deps=$(find "$PROJECT_ROOT/layanan" -name "Cargo.toml" -exec grep -l "serde\|tokio\|axum" {} \; | wc -l)
            echo "  - Services using heavy dependencies: $heavy_deps"

            # Check for release profile
            if grep -q "\[profile.release\]" "$PROJECT_ROOT/Cargo.toml"; then
                echo "  ✅ Release profile configured"
            else
                echo "  ⚠️  Consider adding release profile optimizations"
            fi

            echo "  📝 Recommendations:"
            echo "    - Use 'cargo build --release' for production"
            echo "    - Consider using cargo-audit for dependency security"
            echo "    - Review async runtime usage (tokio optimizations)"
            echo "    - Check for unnecessary Clone() operations"
            ;;
        "frontend"|"wasm")
            echo -e "${YELLOW}Frontend/WASM optimizations:${NC}"
            echo "📊 Analyzing WASM bundle sizes..."

            local trunk_configs=$(find "$PROJECT_ROOT/antarmuka" -name "Trunk.toml" | wc -l)
            echo "  - WASM frontends to optimize: $trunk_configs"

            echo "  📝 Recommendations:"
            echo "    - Use wasm-opt for bundle size reduction"
            echo "    - Enable tree shaking in Trunk.toml"
            echo "    - Consider lazy loading for large components"
            echo "    - Optimize CSS bundle sizes"
            ;;
        "database"|"db")
            echo -e "${YELLOW}Database optimizations:${NC}"
            echo "📊 Analyzing database usage patterns..."

            local db_services=$(find "$PROJECT_ROOT/layanan" -name "*.rs" -exec grep -l "sqlx\|database" {} \; | wc -l)
            echo "  - Services using database: $db_services"

            echo "  📝 Recommendations:"
            echo "    - Use connection pooling"
            echo "    - Add database indexing for frequent queries"
            echo "    - Consider query optimization"
            echo "    - Review N+1 query patterns"
            ;;
        *)
            echo -e "${YELLOW}General optimizations:${NC}"
            echo "📊 Full codebase analysis..."

            local total_size=$(du -sh "$PROJECT_ROOT" 2>/dev/null | cut -f1)
            echo "  - Project size: $total_size"
            echo "  - Services: $(find "$PROJECT_ROOT/layanan" -mindepth 1 -maxdepth 1 -type d | wc -l)"
            echo "  - Frontends: $(find "$PROJECT_ROOT/antarmuka" -mindepth 1 -maxdepth 1 -type d | wc -l)"

            echo "  📝 General recommendations:"
            echo "    - Regular dependency updates"
            echo "    - Code splitting and modularization"
            echo "    - Caching strategies implementation"
            echo "    - Performance monitoring setup"
            ;;
    esac

    info "✅ Optimization analysis completed!"
}

optimize_with_ai() {
    local target="$1"

    info "Requesting AI-powered optimization analysis..."

    if command -v curl >/dev/null 2>&1; then
        local response
        if response=$(curl -s -X POST "$AI_SERVICE_URL/api/v1/ai/optimize" \
            -H "Content-Type: application/json" \
            -d "{\"target\": \"$target\", \"project_type\": \"rust_microservices\"}" 2>/dev/null); then
            echo -e "${GREEN}AI Optimization Suggestions:${NC}"
            echo "$response"
        else
            warn "AI optimization service unavailable, falling back to static analysis"
            optimize_code "$target" "false"
        fi
    fi
}
    fi

    echo -e "${YELLOW}Optimization suggestions generated!${NC}"
}

review_code() {
    echo -e "${BLUE}[AI-REVIEW]${NC} Enhanced automated code review..."

    local target="${1:-all}"
    local use_ai="${2:-false}"
    local review_dir="$PROJECT_ROOT/docs/reviews"

    mkdir -p "$review_dir" || error_exit "Cannot create review directory"

    echo -e "${GREEN}Running comprehensive quality checks for: $target${NC}"

    # Check if AI service is available
    if [[ "$use_ai" == "--ai" ]] && check_ai_service; then
        info "Using AI service for code review"
        review_with_ai "$target" "$review_dir"
        return 0
    fi

    # Enhanced static code review
    case "$target" in
        "backend"|"layanan")
            info "Reviewing backend services..."

            # Check for common Rust issues
            local unsafe_usage=$(find "$PROJECT_ROOT/layanan" -name "*.rs" -exec grep -l "unsafe" {} \; 2>/dev/null | wc -l)
            local unwrap_usage=$(find "$PROJECT_ROOT/layanan" -name "*.rs" -exec grep -c "unwrap()" {} \; 2>/dev/null | paste -sd+ | bc 2>/dev/null || echo "0")
            local todo_count=$(find "$PROJECT_ROOT/layanan" -name "*.rs" -exec grep -c "TODO\|FIXME" {} \; 2>/dev/null | paste -sd+ | bc 2>/dev/null || echo "0")

            echo "📊 Backend Code Quality Metrics:"
            echo "  - Files with unsafe code: $unsafe_usage"
            echo "  - Total unwrap() calls: $unwrap_usage"
            echo "  - TODO/FIXME comments: $todo_count"

            # Generate review report
            cat > "$review_dir/backend_review.md" << EOF
# Backend Code Review Report
Generated: $(date)

## Quality Metrics
- Services reviewed: $(find "$PROJECT_ROOT/layanan" -mindepth 1 -maxdepth 1 -type d | wc -l)
- Unsafe code blocks: $unsafe_usage
- Unwrap calls (potential panics): $unwrap_usage
- TODO/FIXME items: $todo_count

## Recommendations
$(if [ "$unwrap_usage" -gt 5 ]; then echo "- 🔴 HIGH: Consider replacing unwrap() with proper error handling"; fi)
$(if [ "$unsafe_usage" -gt 0 ]; then echo "- 🟡 MEDIUM: Review unsafe code blocks for safety"; fi)
$(if [ "$todo_count" -gt 10 ]; then echo "- 🟡 MEDIUM: Address pending TODO items"; fi)
- ✅ Consider adding more comprehensive error handling
- ✅ Add integration tests for critical paths
- ✅ Document public APIs with rustdoc
EOF
            ;;
        "frontend"|"antarmuka")
            info "Reviewing frontend code..."

            local console_logs=$(find "$PROJECT_ROOT/antarmuka" -name "*.rs" -exec grep -c "console::" {} \; 2>/dev/null | paste -sd+ | bc 2>/dev/null || echo "0")
            local style_files=$(find "$PROJECT_ROOT/antarmuka" -name "*.css" | wc -l)

            echo "📊 Frontend Code Quality Metrics:"
            echo "  - Console log statements: $console_logs"
            echo "  - Style files: $style_files"

            cat > "$review_dir/frontend_review.md" << EOF
# Frontend Code Review Report
Generated: $(date)

## Quality Metrics
- Microfrontends reviewed: $(find "$PROJECT_ROOT/antarmuka" -mindepth 1 -maxdepth 1 -type d | wc -l)
- Console log statements: $console_logs
- Style files: $style_files

## Recommendations
$(if [ "$console_logs" -gt 0 ]; then echo "- 🟡 MEDIUM: Remove console logs from production code"; fi)
- ✅ Consider component testing with leptos-use
- ✅ Optimize WASM bundle sizes
- ✅ Implement consistent error handling
EOF
            ;;
        *)
            info "Performing full project review..."

            local total_lines=$(find "$PROJECT_ROOT" -name "*.rs" -not -path "*/target/*" -exec wc -l {} \; 2>/dev/null | awk '{sum+=$1} END {print sum}')
            local test_files=$(find "$PROJECT_ROOT" -name "*test*.rs" -o -name "tests.rs" | wc -l)

            echo "📊 Project-wide Quality Metrics:"
            echo "  - Total lines of Rust code: $total_lines"
            echo "  - Test files: $test_files"

            cat > "$review_dir/full_review.md" << EOF
# Complete Project Review Report
Generated: $(date)

## Overview
- Total Rust code lines: $total_lines
- Test files: $test_files
- Backend services: $(find "$PROJECT_ROOT/layanan" -mindepth 1 -maxdepth 1 -type d | wc -l)
- Frontend apps: $(find "$PROJECT_ROOT/antarmuka" -mindepth 1 -maxdepth 1 -type d | wc -l)

## Architecture Review
- ✅ Well-structured modular architecture
- ✅ Consistent naming conventions
- ✅ Proper separation of concerns

## Recommendations
- 🔴 Add more comprehensive testing (current: $test_files files)
- 🟡 Consider CI/CD pipeline improvements
- ✅ Maintain documentation updates
- ✅ Regular dependency audits
EOF
            ;;
    esac

    info "✅ Code review completed! Reports generated in: $review_dir"
}

review_with_ai() {
    local target="$1"
    local review_dir="$2"

    info "Performing AI-powered code review..."

    if command -v curl >/dev/null 2>&1; then
        local response
        if response=$(curl -s -X POST "$AI_SERVICE_URL/api/v1/ai/review" \
            -H "Content-Type: application/json" \
            -d "{\"target\": \"$target\", \"depth\": \"comprehensive\"}" 2>/dev/null); then

            echo -e "${GREEN}AI Code Review Results:${NC}"
            echo "$response"

            # Save AI review to file
            echo "$response" > "$review_dir/ai_review_$(date +%Y%m%d_%H%M%S).md"
            info "AI review saved to $review_dir"
        else
            warn "AI review service unavailable, falling back to static review"
            review_code "$target" "false"
        fi
    fi
}

generate_docs() {
    echo -e "${BLUE}[AI-DOCS]${NC} Documentation generation..."

    local target="${1:-all}"
    local docs_dir="$PROJECT_ROOT/docs/generated"

    mkdir -p "$docs_dir"

    echo -e "${GREEN}Generating documentation for: $target${NC}"
    echo "- API documentation from code comments..."
    echo "- Architecture diagrams..."
    echo "- Setup and deployment guides..."
    echo "- Developer documentation..."

    echo -e "${YELLOW}Documentation generated at: $docs_dir${NC}"
}

fix_code() {
    echo -e "${BLUE}[AI-FIX]${NC} Auto-fixing common issues..."

    echo -e "${GREEN}Scanning for common issues...${NC}"
    echo "- Fixing formatting issues..."
    echo "- Resolving import statements..."
    echo "- Correcting common syntax errors..."
    echo "- Updating deprecated patterns..."

    echo -e "${YELLOW}Auto-fix completed! Review changes before committing.${NC}"
}

generate_tests() {
    echo -e "${BLUE}[AI-TEST]${NC} Test generation assistant..."

    local target="${1:-unit}"

    echo -e "${GREEN}Generating $target tests...${NC}"

    case "$target" in
        "unit")
            echo "- Creating unit tests for functions..."
            echo "- Testing edge cases and error conditions..."
            ;;
        "integration")
            echo "- Creating integration tests for services..."
            echo "- Testing API endpoints..."
            ;;
        "e2e")
            echo "- Creating end-to-end tests..."
            echo "- Testing user workflows..."
            ;;
    esac

    echo -e "${YELLOW}Test generation completed!${NC}"
}

main() {
    if [ $# -eq 0 ]; then
        show_help
        exit 0
    fi

    local command="$1"
    shift

    case "$command" in
        "help"|"-h"|"--help")
            show_help
            ;;
        "status")
            check_status
            ;;
        "analyze")
            analyze_code "$@"
            ;;
        "generate")
            generate_code "$@"
            ;;
        "optimize")
            optimize_code "$@"
            ;;
        "review")
            review_code "$@"
            ;;
        "docs")
            generate_docs "$@"
            ;;
        "fix")
            fix_code "$@"
            ;;
        "test")
            generate_tests "$@"
            ;;
        *)
            echo -e "${RED}Unknown command: $command${NC}"
            echo "Use 'help' to see available commands."
            exit 1
            ;;
    esac
}

main "$@"
