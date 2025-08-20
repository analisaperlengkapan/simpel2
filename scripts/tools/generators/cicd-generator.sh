#!/bin/bash

# 🚀 SIMPelv2 CI/CD Module  
# Continuous Integration and Continuous Deployment automation

set -euo pipefail

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
PURPLE='\033[0;35m'
NC='\033[0m'

WORKSPACE_ROOT="/var/www/simpelv2"
CI_CONFIG_DIR="$WORKSPACE_ROOT/.ci"

echo -e "${CYAN}🚀 SIMPelv2 CI/CD Module${NC}"

# Generate CI/CD pipeline configurations
generate_pipeline() {
    local platform="${1:-github}"
    
    echo -e "\n${GREEN}⚙️  Generating CI/CD Pipeline${NC}"
    echo "============================"
    
    cd "$WORKSPACE_ROOT"
    
    case "$platform" in
        "github"|"gh")
            echo "🐙 Creating GitHub Actions workflow..."
            
            mkdir -p .github/workflows
            
            cat > .github/workflows/ci.yml << 'EOF'
name: SIMPelv2 CI/CD Pipeline

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  test:
    name: Test Suite
    runs-on: ubuntu-latest
    
    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_USER: test
          POSTGRES_PASSWORD: test
          POSTGRES_DB: simpelv2_test
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 5432:5432
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Install Rust toolchain
      uses: dtolnay/rust-toolchain@stable
      with:
        components: clippy, rustfmt
        
    - name: Cache cargo dependencies
      uses: actions/cache@v3
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
        
    - name: Check formatting
      run: cargo fmt --all -- --check
      
    - name: Run clippy
      run: cargo clippy --all-targets --all-features -- -D warnings
      
    - name: Run tests
      run: cargo test --all-features
      env:
        DATABASE_URL: postgresql://test:test@localhost/simpelv2_test
        
    - name: Run security audit
      run: |
        cargo install cargo-audit
        cargo audit

  build-backend:
    name: Build Backend Services
    runs-on: ubuntu-latest
    needs: test
    
    strategy:
      matrix:
        service: [dasbor, keamanan, laporan]
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Install Rust toolchain
      uses: dtolnay/rust-toolchain@stable
      
    - name: Cache cargo dependencies
      uses: actions/cache@v3
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
        
    - name: Build service
      run: cargo build --release --bin ${{ matrix.service }}
      working-directory: layanan/${{ matrix.service }}
      
    - name: Build Docker image
      run: |
        docker build -t simpelv2/${{ matrix.service }}:${{ github.sha }} \
          -f layanan/${{ matrix.service }}/Dockerfile .
        
    - name: Save Docker image
      run: |
        docker save simpelv2/${{ matrix.service }}:${{ github.sha }} | \
          gzip > ${{ matrix.service }}-image.tar.gz
        
    - name: Upload image artifact
      uses: actions/upload-artifact@v3
      with:
        name: ${{ matrix.service }}-image
        path: ${{ matrix.service }}-image.tar.gz

  build-frontend:
    name: Build Frontend Applications
    runs-on: ubuntu-latest
    needs: test
    
    strategy:
      matrix:
        frontend: [portal, badiklat, intel]
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Install Rust toolchain
      uses: dtolnay/rust-toolchain@stable
      with:
        targets: wasm32-unknown-unknown
        
    - name: Install trunk
      run: cargo install trunk
      
    - name: Cache cargo dependencies
      uses: actions/cache@v3
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
        
    - name: Build frontend
      run: trunk build --release
      working-directory: antarmuka/${{ matrix.frontend }}
      
    - name: Upload frontend artifact
      uses: actions/upload-artifact@v3
      with:
        name: ${{ matrix.frontend }}-frontend
        path: antarmuka/${{ matrix.frontend }}/dist/

  deploy-staging:
    name: Deploy to Staging
    runs-on: ubuntu-latest
    needs: [build-backend, build-frontend]
    if: github.ref == 'refs/heads/develop'
    
    environment: staging
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Download artifacts
      uses: actions/download-artifact@v3
      
    - name: Deploy to staging
      run: |
        echo "🚀 Deploying to staging environment..."
        # Add your deployment commands here
        
  deploy-production:
    name: Deploy to Production
    runs-on: ubuntu-latest
    needs: [build-backend, build-frontend]
    if: github.ref == 'refs/heads/main'
    
    environment: production
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Download artifacts
      uses: actions/download-artifact@v3
      
    - name: Deploy to production
      run: |
        echo "🚀 Deploying to production environment..."
        # Add your deployment commands here
EOF
            
            echo "✅ GitHub Actions workflow created: .github/workflows/ci.yml"
            ;;
            
        "gitlab"|"gl")
            echo "🦊 Creating GitLab CI/CD pipeline..."
            
            cat > .gitlab-ci.yml << 'EOF'
# SIMPelv2 GitLab CI/CD Pipeline

stages:
  - test
  - build
  - deploy

variables:
  CARGO_HOME: ${CI_PROJECT_DIR}/.cargo
  RUST_BACKTRACE: "1"

cache:
  paths:
    - .cargo/
    - target/

before_script:
  - apt-get update -qq && apt-get install -y -qq git curl
  - curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  - source ~/.cargo/env
  - rustup component add clippy rustfmt

test:
  stage: test
  services:
    - postgres:15
  variables:
    POSTGRES_DB: simpelv2_test
    POSTGRES_USER: test
    POSTGRES_PASSWORD: test
    DATABASE_URL: postgresql://test:test@postgres/simpelv2_test
  script:
    - cargo fmt --all -- --check
    - cargo clippy --all-targets --all-features -- -D warnings
    - cargo test --all-features
    - cargo install cargo-audit
    - cargo audit

build:backend:
  stage: build
  parallel:
    matrix:
      - SERVICE: [dasbor, keamanan, laporan]
  script:
    - cargo build --release --bin $SERVICE
    - docker build -t $CI_REGISTRY_IMAGE/$SERVICE:$CI_COMMIT_SHA -f layanan/$SERVICE/Dockerfile .
    - docker push $CI_REGISTRY_IMAGE/$SERVICE:$CI_COMMIT_SHA
  only:
    - main
    - develop

build:frontend:
  stage: build
  parallel:
    matrix:
      - FRONTEND: [portal, badiklat, intel]
  before_script:
    - rustup target add wasm32-unknown-unknown
    - cargo install trunk
  script:
    - cd antarmuka/$FRONTEND
    - trunk build --release
  artifacts:
    paths:
      - antarmuka/*/dist/
    expire_in: 1 hour
  only:
    - main
    - develop

deploy:staging:
  stage: deploy
  script:
    - echo "🚀 Deploying to staging..."
    - # Add staging deployment commands
  environment:
    name: staging
    url: https://staging.simpelv2.example.com
  only:
    - develop

deploy:production:
  stage: deploy
  script:
    - echo "🚀 Deploying to production..."
    - # Add production deployment commands
  environment:
    name: production
    url: https://simpelv2.example.com
  when: manual
  only:
    - main
EOF
            
            echo "✅ GitLab CI/CD pipeline created: .gitlab-ci.yml"
            ;;
            
        "jenkins")
            echo "🏗️  Creating Jenkinsfile..."
            
            cat > Jenkinsfile << 'EOF'
// SIMPelv2 Jenkins Pipeline

pipeline {
    agent any
    
    environment {
        RUST_BACKTRACE = '1'
        CARGO_HOME = "${WORKSPACE}/.cargo"
    }
    
    stages {
        stage('Setup') {
            steps {
                sh '''
                    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
                    source ~/.cargo/env
                    rustup component add clippy rustfmt
                    rustup target add wasm32-unknown-unknown
                '''
            }
        }
        
        stage('Test') {
            steps {
                sh '''
                    source ~/.cargo/env
                    cargo fmt --all -- --check
                    cargo clippy --all-targets --all-features -- -D warnings
                    cargo test --all-features
                '''
            }
        }
        
        stage('Build Backend') {
            parallel {
                stage('Build Dasbor') {
                    steps {
                        sh 'cargo build --release --bin dasbor'
                    }
                }
                stage('Build Keamanan') {
                    steps {
                        sh 'cargo build --release --bin keamanan'
                    }
                }
                stage('Build Laporan') {
                    steps {
                        sh 'cargo build --release --bin laporan'
                    }
                }
            }
        }
        
        stage('Build Frontend') {
            steps {
                sh '''
                    source ~/.cargo/env
                    cargo install trunk
                    
                    for frontend in portal badiklat intel; do
                        if [ -d "antarmuka/$frontend" ]; then
                            cd "antarmuka/$frontend"
                            trunk build --release
                            cd "../.."
                        fi
                    done
                '''
            }
        }
        
        stage('Deploy') {
            when {
                branch 'main'
            }
            steps {
                echo '🚀 Deploying to production...'
                // Add deployment steps
            }
        }
    }
    
    post {
        always {
            cleanWs()
        }
    }
}
EOF
            
            echo "✅ Jenkins pipeline created: Jenkinsfile"
            ;;
            
        *)
            echo "❌ Unknown platform: $platform"
            echo "Supported platforms: github, gitlab, jenkins"
            return 1
            ;;
    esac
}

# Run CI pipeline locally
run_local_ci() {
    echo -e "\n${BLUE}🔧 Running Local CI Pipeline${NC}"
    echo "=========================="
    
    cd "$WORKSPACE_ROOT"
    
    echo "🔍 Step 1: Code formatting check..."
    if cargo fmt --all -- --check; then
        echo "✅ Formatting check passed"
    else
        echo "❌ Formatting check failed"
        return 1
    fi
    
    echo ""
    echo "📎 Step 2: Linting with clippy..."
    if cargo clippy --all-targets --all-features -- -D warnings; then
        echo "✅ Clippy check passed"
    else
        echo "❌ Clippy check failed"
        return 1
    fi
    
    echo ""
    echo "🧪 Step 3: Running tests..."
    if cargo test --all-features; then
        echo "✅ Tests passed"
    else
        echo "❌ Tests failed"
        return 1
    fi
    
    echo ""
    echo "🔒 Step 4: Security audit..."
    if command -v cargo-audit >/dev/null 2>&1; then
        if cargo audit; then
            echo "✅ Security audit passed"
        else
            echo "⚠️  Security audit found issues"
        fi
    else
        echo "📦 Installing cargo-audit..."
        cargo install cargo-audit
        cargo audit
    fi
    
    echo ""
    echo "🏗️  Step 5: Building backend services..."
    local services=($(find layanan -maxdepth 1 -type d ! -name layanan | head -3))
    
    for service_dir in "${services[@]}"; do
        service_name=$(basename "$service_dir")
        echo "  📦 Building $service_name..."
        
        cd "$service_dir"
        if cargo build --release; then
            echo "    ✅ $service_name built successfully"
        else
            echo "    ❌ $service_name build failed"
            return 1
        fi
        cd "$WORKSPACE_ROOT"
    done
    
    echo ""
    echo "🌐 Step 6: Building frontend applications..."
    local frontends=($(find antarmuka -maxdepth 1 -type d ! -name antarmuka -a ! -name shared | head -3))
    
    if ! command -v trunk >/dev/null 2>&1; then
        echo "📦 Installing trunk..."
        cargo install trunk
    fi
    
    for frontend_dir in "${frontends[@]}"; do
        frontend_name=$(basename "$frontend_dir")
        echo "  🌐 Building $frontend_name..."
        
        cd "$frontend_dir"
        if [[ -f "Trunk.toml" ]]; then
            if trunk build --release; then
                echo "    ✅ $frontend_name built successfully"
            else
                echo "    ❌ $frontend_name build failed"
                return 1
            fi
        else
            echo "    ⚠️  No Trunk.toml found for $frontend_name"
        fi
        cd "$WORKSPACE_ROOT"
    done
    
    echo ""
    echo "🎉 Local CI pipeline completed successfully!"
}

# Deploy to environment
deploy_env() {
    local environment="${1:-staging}"
    local version="${2:-latest}"
    
    echo -e "\n${PURPLE}🚀 Deploying to $environment${NC}"
    echo "======================="
    
    cd "$WORKSPACE_ROOT"
    
    case "$environment" in
        "staging"|"dev")
            echo "🛠️  Deploying to staging environment..."
            
            # Use development compose file
            if [[ -f "docker-compose.dev.yml" ]]; then
                echo "  🐳 Starting staging services..."
                docker-compose -f docker-compose.dev.yml up -d
                
                echo "  ⏳ Waiting for services to be ready..."
                sleep 10
                
                # Health check
                if curl -f http://localhost:8080/health >/dev/null 2>&1; then
                    echo "  ✅ Staging deployment successful"
                else
                    echo "  ⚠️  Services may still be starting up"
                fi
            else
                echo "❌ No staging configuration found"
                return 1
            fi
            ;;
            
        "production"|"prod")
            echo "🌟 Deploying to production environment..."
            
            # Confirmation for production deployment
            echo "⚠️  This will deploy to PRODUCTION. Continue? (y/N)"
            read -r confirm
            if [[ ! "$confirm" =~ ^[Yy]$ ]]; then
                echo "❌ Production deployment cancelled"
                return 1
            fi
            
            # Use production compose file
            if [[ -f "docker-compose.prod.yml" ]]; then
                echo "  🐳 Starting production services..."
                docker-compose -f docker-compose.prod.yml up -d
                
                echo "  ⏳ Waiting for services to be ready..."
                sleep 15
                
                # Health check
                if curl -f https://simpelv2.example.com/health >/dev/null 2>&1; then
                    echo "  ✅ Production deployment successful"
                else
                    echo "  ⚠️  Services may still be starting up"
                fi
            else
                echo "❌ No production configuration found"
                return 1
            fi
            ;;
            
        *)
            echo "❌ Unknown environment: $environment"
            echo "Supported environments: staging, production"
            return 1
            ;;
    esac
}

# Monitor deployment status
monitor_deployment() {
    local environment="${1:-staging}"
    
    echo -e "\n${YELLOW}📊 Monitoring Deployment${NC}"
    echo "======================="
    
    echo "🔍 Checking $environment deployment status..."
    
    # Check Docker containers
    echo ""
    echo "🐳 Container status:"
    docker ps --format "table {{.Names}}\t{{.Image}}\t{{.Status}}\t{{.Ports}}" | head -10
    
    # Health checks
    echo ""
    echo "🩺 Health checks:"
    
    local endpoints=("http://localhost:8080/health" "http://localhost:3000/health")
    
    for endpoint in "${endpoints[@]}"; do
        echo "  🔍 Checking $endpoint..."
        if curl -f "$endpoint" >/dev/null 2>&1; then
            echo "    ✅ $endpoint is healthy"
        else
            echo "    ❌ $endpoint is not responding"
        fi
    done
    
    # Resource usage
    echo ""
    echo "💻 Resource usage:"
    docker stats --no-stream --format "table {{.Container}}\t{{.CPUPerc}}\t{{.MemUsage}}" | head -5
}

# Main function
main() {
    case "${1:-help}" in
        "generate"|"init")
            generate_pipeline "${2:-github}"
            ;;
        "run"|"local")
            run_local_ci
            ;;
        "deploy")
            deploy_env "${2:-staging}" "${3:-latest}"
            ;;
        "monitor"|"status")
            monitor_deployment "${2:-staging}"
            ;;
        "help"|"--help"|"-h")
            echo ""
            echo "SIMPelv2 CI/CD Module"
            echo ""
            echo "Usage: $0 [command] [options]"
            echo ""
            echo "Commands:"
            echo "  generate <platform>     - Generate CI/CD pipeline configuration"
            echo "    Platforms: github, gitlab, jenkins"
            echo "  run                     - Run CI pipeline locally"
            echo "  deploy <env> [version]  - Deploy to environment"
            echo "    Environments: staging, production"
            echo "  monitor <env>           - Monitor deployment status"
            echo "  help                    - Show this help"
            echo ""
            echo "Examples:"
            echo "  $0 generate github      # Create GitHub Actions workflow"
            echo "  $0 run                  # Run local CI pipeline"
            echo "  $0 deploy staging       # Deploy to staging"
            echo "  $0 monitor production   # Monitor production deployment"
            ;;
        *)
            echo -e "${RED}❌ Unknown command: $1${NC}"
            echo "Use '$0 help' for available commands"
            exit 1
            ;;
    esac
}

main "$@"
