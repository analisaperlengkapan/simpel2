use anyhow::Result;
use colored::*;
use std::fs;

pub async fn generate_cicd_pipeline(platform: &str, template: Option<&str>) -> Result<()> {
    println!("{}", "🚀 Generating CI/CD pipeline...".green());

    match platform {
        "github" => generate_github_actions(template.unwrap_or("minimal"))?,
        "gitlab" => generate_gitlab_ci(template.unwrap_or("minimal"))?,
        "jenkins" => generate_jenkinsfile(template.unwrap_or("minimal"))?,
        _ => {
            println!(
                "{}",
                "❌ Unsupported platform. Use: github, gitlab, or jenkins".red()
            );
            return Ok(());
        }
    }

    println!("{}", "✅ CI/CD pipeline generated successfully!".green());
    Ok(())
}

fn generate_github_actions(template: &str) -> Result<()> {
    fs::create_dir_all(".github/workflows")?;

    let workflow_content = match template {
        "minimal" => create_github_minimal_workflow(),
        "full" => create_github_full_workflow(),
        _ => create_github_minimal_workflow(),
    };

    let filename = format!("ci-{}.yml", template);
    let path = format!(".github/workflows/{}", filename);
    fs::write(&path, workflow_content)?;

    println!("📁 Created: {}", path);
    Ok(())
}

fn generate_gitlab_ci(template: &str) -> Result<()> {
    let content = match template {
        "minimal" => create_gitlab_minimal_ci(),
        "full" => create_gitlab_full_ci(),
        _ => create_gitlab_minimal_ci(),
    };

    fs::write(".gitlab-ci.yml", content)?;
    println!("📁 Created: .gitlab-ci.yml");
    Ok(())
}

fn generate_jenkinsfile(template: &str) -> Result<()> {
    let content = match template {
        "minimal" => create_jenkins_minimal_pipeline(),
        "full" => create_jenkins_full_pipeline(),
        _ => create_jenkins_minimal_pipeline(),
    };

    fs::write("Jenkinsfile", content)?;
    println!("📁 Created: Jenkinsfile");
    Ok(())
}

fn create_github_minimal_workflow() -> &'static str {
    r#"name: CI/CD Pipeline

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    runs-on: ubuntu-latest

    steps:
    - uses: actions/checkout@v4

    - name: Setup Rust
      uses: dtolnay/rust-toolchain@stable

    - name: Cache dependencies
      uses: Swatinem/rust-cache@v2

    - name: Run tests
      run: cargo test --all

    - name: Check formatting
      run: cargo fmt --check

    - name: Run clippy
      run: cargo clippy --all -- -D warnings

  build:
    runs-on: ubuntu-latest
    needs: test

    steps:
    - uses: actions/checkout@v4

    - name: Setup Rust
      uses: dtolnay/rust-toolchain@stable

    - name: Cache dependencies
      uses: Swatinem/rust-cache@v2

    - name: Build release
      run: cargo build --release

    - name: Upload artifacts
      uses: actions/upload-artifact@v4
      with:
        name: simpel-binary
        path: target/release/simpel
"#
}

fn create_github_full_workflow() -> &'static str {
    r#"name: Full CI/CD Pipeline

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]
  schedule:
    - cron: '0 2 * * 1'

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        rust:
          - stable
          - beta
          - nightly

    steps:
    - uses: actions/checkout@v4

    - name: Setup Rust ${{ matrix.rust }}
      uses: dtolnay/rust-toolchain@master
      with:
        toolchain: ${{ matrix.rust }}

    - name: Cache dependencies
      uses: Swatinem/rust-cache@v2

    - name: Run tests
      run: cargo test --all --verbose

    - name: Generate coverage
      if: matrix.rust == 'stable'
      run: |
        cargo install cargo-tarpaulin
        cargo tarpaulin --out Xml

    - name: Upload coverage
      if: matrix.rust == 'stable'
      uses: codecov/codecov-action@v3
      with:
        file: cobertura.xml

  lint:
    runs-on: ubuntu-latest

    steps:
    - uses: actions/checkout@v4

    - name: Setup Rust
      uses: dtolnay/rust-toolchain@stable

    - name: Check formatting
      run: cargo fmt --all --check

    - name: Run clippy
      run: cargo clippy --all -- -D warnings

    - name: Check documentation
      run: cargo doc --no-deps

  security:
    runs: ubuntu-latest

    steps:
    - uses: actions/checkout@v4

    - name: Setup Rust
      uses: dtolnay/rust-toolchain@stable

    - name: Run cargo audit
      run: |
        cargo install cargo-audit
        cargo audit

  build:
    runs-on: ubuntu-latest
    needs: [test, lint, security]

    steps:
    - uses: actions/checkout@v4

    - name: Setup Rust
      uses: dtolnay/rust-toolchain@stable

    - name: Cache dependencies
      uses: Swatinem/rust-cache@v2

    - name: Build release
      run: cargo build --release

    - name: Upload artifacts
      uses: actions/upload-artifact@v4
      with:
        name: simpel-binary
        path: target/release/simpel

  docker:
    runs-on: ubuntu-latest
    needs: build

    steps:
    - uses: actions/checkout@v4

    - name: Download binary
      uses: actions/download-artifact@v4
      with:
        name: simpel-binary
        path: target/release/

    - name: Build Docker image
      run: |
        chmod +x target/release/simpel
        docker build -t simpelv2:latest .

    - name: Push to registry
      if: github.ref == 'refs/heads/main'
      run: |
        echo ${{ secrets.DOCKER_PASSWORD }} | docker login -u ${{ secrets.DOCKER_USERNAME }} --password-stdin
        docker tag simpelv2:latest ${{ secrets.DOCKER_USERNAME }}/simpelv2:latest
        docker push ${{ secrets.DOCKER_USERNAME }}/simpelv2:latest
"#
}

fn create_gitlab_minimal_ci() -> &'static str {
    r#"stages:
  - test
  - build
  - deploy

variables:
  CARGO_HOME: $CI_PROJECT_DIR/.cargo
  RUST_BACKTRACE: 1

cache:
  paths:
    - .cargo/
    - target/

test:
  stage: test
  image: rust:latest
  before_script:
    - rustc --version && cargo --version
  script:
    - cargo test --all
    - cargo fmt --check
    - cargo clippy --all -- -D warnings
  artifacts:
    reports:
      coverage_report:
        coverage_format: cobertura
        path: cobertura.xml
    expire_in: 1 week

build:
  stage: build
  image: rust:latest
  script:
    - cargo build --release
  artifacts:
    paths:
      - target/release/simpel
    expire_in: 1 week
  only:
    - main
    - develop

deploy_staging:
  stage: deploy
  image: docker:latest
  script:
    - docker build -t simpelv2:$CI_COMMIT_SHA .
    - docker tag simpelv2:$CI_COMMIT_SHA registry.example.com/simpelv2:staging
    - docker push registry.example.com/simpelv2:staging
  only:
    - develop

deploy_production:
  stage: deploy
  image: docker:latest
  script:
    - docker build -t simpelv2:$CI_COMMIT_SHA .
    - docker tag simpelv2:$CI_COMMIT_SHA registry.example.com/simpelv2:latest
    - docker push registry.example.com/simpelv2:latest
  only:
    - main
  when: manual
"#
}

fn create_gitlab_full_ci() -> &'static str {
    r#"stages:
  - validate
  - test
  - build
  - security
  - deploy
  - cleanup

variables:
  CARGO_HOME: $CI_PROJECT_DIR/.cargo
  RUST_BACKTRACE: 1
  DOCKER_DRIVER: overlay2

cache:
  key: ${CI_COMMIT_REF_SLUG}
  paths:
    - .cargo/
    - target/

validate:
  stage: validate
  image: rust:latest
  before_script:
    - rustc --version && cargo --version
  script:
    - cargo check --all
    - cargo fmt --all --check
    - cargo clippy --all -- -D warnings
  allow_failure: false

test:
  stage: test
  image: rust:latest
  services:
    - postgres:13
    - redis:6
  variables:
    POSTGRES_DB: simpel_test
    POSTGRES_USER: postgres
    POSTGRES_PASSWORD: postgres
    DATABASE_URL: postgresql://postgres:postgres@postgres:5432/simpel_test
  before_script:
    - apt-get update && apt-get install -y postgresql-client
    - rustc --version && cargo --version
    - cargo install cargo-tarpaulin
  script:
    - cargo test --all --verbose
    - cargo tarpaulin --out Xml --output-dir reports/
  coverage: '/TOTAL.*\s+(\d+%)$/'
  artifacts:
    reports:
      coverage_report:
        coverage_format: cobertura
        path: reports/cobertura.xml
    paths:
      - reports/
    expire_in: 1 week

security_scan:
  stage: security
  image: rust:latest
  before_script:
    - cargo install cargo-audit
  script:
    - cargo audit --format json || true
  artifacts:
    paths:
      - cargo-audit.json
    expire_in: 1 week
  allow_failure: true

build:
  stage: build
  image: rust:latest
  script:
    - cargo build --release
  artifacts:
    paths:
      - target/release/simpel
    expire_in: 1 week
  only:
    - main
    - develop
    - merge_requests

docker_build:
  stage: build
  image: docker:latest
  services:
    - docker:dind
  script:
    - docker build -t $CI_REGISTRY_IMAGE:$CI_COMMIT_SHA .
    - docker push $CI_REGISTRY_IMAGE:$CI_COMMIT_SHA
  only:
    - main
    - develop

deploy_staging:
  stage: deploy
  image: alpine:latest
  before_script:
    - apk add --no-cache openssh-client
    - eval $(ssh-agent -s)
    - echo "$SSH_PRIVATE_KEY" | tr -d '\r' | ssh-add -
    - mkdir -p ~/.ssh
    - chmod 700 ~/.ssh
    - ssh-keyscan -H $STAGING_HOST >> ~/.ssh/known_hosts
  script:
    - |
      ssh $STAGING_USER@$STAGING_HOST << EOF
        cd /opt/simpelv2
        docker pull $CI_REGISTRY_IMAGE:$CI_COMMIT_SHA
        docker tag $CI_REGISTRY_IMAGE:$CI_COMMIT_SHA simpelv2:latest
        docker-compose down
        docker-compose up -d
        docker system prune -f
      EOF
  environment:
    name: staging
    url: https://staging.simpelv2.com
  only:
    - develop
  dependencies:
    - docker_build

deploy_production:
  stage: deploy
  image: alpine:latest
  before_script:
    - apk add --no-cache openssh-client
    - eval $(ssh-agent -s)
    - echo "$SSH_PRIVATE_KEY" | tr -d '\r' | ssh-add -
    - mkdir -p ~/.ssh
    - chmod 700 ~/.ssh
    - ssh-keyscan -H $PRODUCTION_HOST >> ~/.ssh/known_hosts
  script:
    - |
      ssh $PRODUCTION_USER@$PRODUCTION_HOST << EOF
        cd /opt/simpelv2
        docker pull $CI_REGISTRY_IMAGE:$CI_COMMIT_SHA
        docker tag $CI_REGISTRY_IMAGE:$CI_COMMIT_SHA simpelv2:latest
        docker-compose -f docker-compose.prod.yml down
        docker-compose -f docker-compose.prod.yml up -d --scale web=3
        docker system prune -f
      EOF
  environment:
    name: production
    url: https://simpelv2.com
  only:
    - main
  when: manual
  dependencies:
    - docker_build

cleanup:
  stage: cleanup
  image: alpine:latest
  script:
    - docker system prune -f
    - docker image prune -f
  only:
    - schedules
  when: manual
"#
}

fn create_jenkins_minimal_pipeline() -> &'static str {
    r#"pipeline {
    agent any

    environment {
        CARGO_HOME = '${WORKSPACE}/.cargo'
        RUST_BACKTRACE = '1'
    }

    stages {
        stage('Checkout') {
            steps {
                checkout scm
            }
        }

        stage('Setup Rust') {
            steps {
                sh 'curl --proto \'=https\' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y'
                sh 'source ~/.cargo/env'
                sh 'rustc --version && cargo --version'
            }
        }

        stage('Test') {
            steps {
                sh 'source ~/.cargo/env'
                sh 'cargo test --all'
            }
        }

        stage('Build') {
            steps {
                sh 'source ~/.cargo/env'
                sh 'cargo build --release'
            }
        }

        stage('Archive') {
            steps {
                archiveArtifacts artifacts: 'target/release/simpel', fingerprint: true
            }
        }
    }

    post {
        always {
            cleanWs()
        }
        success {
            echo 'Pipeline succeeded!'
        }
        failure {
            echo 'Pipeline failed!'
        }
    }
}
"#
}

fn create_jenkins_full_pipeline() -> &'static str {
    r#"pipeline {
    agent any

    environment {
        CARGO_HOME = '${WORKSPACE}/.cargo'
        RUST_BACKTRACE = '1'
        DOCKER_IMAGE = 'simpelv2'
        REGISTRY = 'registry.example.com'
    }

    stages {
        stage('Checkout') {
            steps {
                checkout scm
            }
        }

        stage('Setup Environment') {
            steps {
                sh 'curl --proto \'=https\' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y'
                sh 'source ~/.cargo/env'
                sh 'rustc --version && cargo --version'
                sh 'cargo install cargo-audit cargo-tarpaulin'
            }
        }

        stage('Validate') {
            steps {
                sh 'source ~/.cargo/env'
                sh 'cargo check --all'
                sh 'cargo fmt --all --check'
                sh 'cargo clippy --all -- -D warnings'
            }
        }

        stage('Test') {
            steps {
                sh 'source ~/.cargo/env'
                sh 'cargo test --all --verbose'
                sh 'cargo tarpaulin --out Xml --output-dir reports/'
            }
            post {
                always {
                    publishCoverage adapters: [coberturaAdapter('reports/cobertura.xml')]
                }
            }
        }

        stage('Security Scan') {
            steps {
                sh 'source ~/.cargo/env'
                sh 'cargo audit --format json > cargo-audit.json || true'
            }
            post {
                always {
                    archiveArtifacts artifacts: 'cargo-audit.json', allowEmptyArchive: true
                }
            }
        }

        stage('Build') {
            steps {
                sh 'source ~/.cargo/env'
                sh 'cargo build --release'
            }
            post {
                success {
                    archiveArtifacts artifacts: 'target/release/simpel', fingerprint: true
                }
            }
        }

        stage('Docker Build') {
            steps {
                script {
                    docker.build("${DOCKER_IMAGE}:${BUILD_NUMBER}")
                }
            }
        }

        stage('Docker Push') {
            steps {
                script {
                    docker.withRegistry("https://${REGISTRY}", 'registry-credentials') {
                        docker.image("${DOCKER_IMAGE}:${BUILD_NUMBER}").push()
                        docker.image("${DOCKER_IMAGE}:${BUILD_NUMBER}").push('latest')
                    }
                }
            }
        }

        stage('Deploy to Staging') {
            when {
                branch 'develop'
            }
            steps {
                script {
                    sh """
                        docker pull ${REGISTRY}/${DOCKER_IMAGE}:${BUILD_NUMBER}
                        docker tag ${REGISTRY}/${DOCKER_IMAGE}:${BUILD_NUMBER} ${DOCKER_IMAGE}:staging
                        docker-compose -f docker-compose.staging.yml down || true
                        docker-compose -f docker-compose.staging.yml up -d
                        docker system prune -f
                    """
                }
            }
        }

        stage('Deploy to Production') {
            when {
                branch 'main'
            }
            steps {
                input message: 'Deploy to Production?', ok: 'Deploy'
                script {
                    sh """
                        docker pull ${REGISTRY}/${DOCKER_IMAGE}:${BUILD_NUMBER}
                        docker tag ${REGISTRY}/${DOCKER_IMAGE}:${BUILD_NUMBER} ${DOCKER_IMAGE}:latest
                        docker-compose -f docker-compose.prod.yml down || true
                        docker-compose -f docker-compose.prod.yml up -d --scale web=3
                        docker system prune -f
                    """
                }
            }
        }
    }

    post {
        always {
            cleanWs()
            sh 'docker system prune -f || true'
        }
        success {
            echo 'Pipeline completed successfully!'
            slackSend channel: '#ci-cd',
                      color: 'good',
                      message: "Build ${BUILD_NUMBER} succeeded!"
        }
        failure {
            echo 'Pipeline failed!'
            slackSend channel: '#ci-cd',
                      color: 'danger',
                      message: "Build ${BUILD_NUMBER} failed!"
        }
    }
}
"#
}

// Docker Compose generation (migrated from Python)
pub async fn generate_docker_compose(environment: &str, services: &[&str]) -> Result<()> {
    println!(
        "{}",
        "🐳 Generating Docker Compose configuration...".bright_blue()
    );

    let compose_content = match environment {
        "dev" => generate_dev_compose(services),
        "staging" => generate_staging_compose(services),
        "prod" => generate_prod_compose(services),
        _ => generate_dev_compose(services),
    };

    let filename = format!("docker-compose.{}.yml", environment);
    fs::write(&filename, compose_content)?;

    println!("📁 Created: {}", filename);
    Ok(())
}

fn generate_dev_compose(services: &[&str]) -> String {
    let mut compose = String::from(
        r#"version: '3.8'

services:
"#,
    );

    for service in services {
        compose.push_str(&format!(
            r#"
  {}:
    build:
      context: .
      dockerfile: layanan/{}/Dockerfile
    ports:
      - "{}:8080"
    environment:
      - RUST_LOG=debug
      - DATABASE_URL=postgresql://user:pass@postgres:5432/simpel_{}
    depends_on:
      - postgres
      - vault
    networks:
      - simpel-network
"#,
            service,
            service,
            get_service_port(service),
            service
        ));
    }

    compose.push_str(
        r#"
  postgres:
    image: postgres:15
    environment:
      POSTGRES_DB: simpel_dev
      POSTGRES_USER: user
      POSTGRES_PASSWORD: pass
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
    networks:
      - simpel-network

  vault:
    image: hashicorp/vault:latest
    cap_add:
      - IPC_LOCK
    environment:
      VAULT_DEV_ROOT_TOKEN_ID: root
      VAULT_DEV_LISTEN_ADDRESS: 0.0.0.0:8200
    ports:
      - "8200:8200"
    networks:
      - simpel-network

  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./infra/nginx/dev.conf:/etc/nginx/nginx.conf
    depends_on:
"#,
    );

    for service in services {
        compose.push_str(&format!("      - {}\n", service));
    }

    compose.push_str(
        r#"    networks:
      - simpel-network

volumes:
  postgres_data:

networks:
  simpel-network:
    driver: bridge
"#,
    );

    compose
}

fn generate_staging_compose(services: &[&str]) -> String {
    // Similar to dev but with staging configurations
    let mut compose = String::from(
        r#"version: '3.8'

services:
"#,
    );

    for service in services {
        compose.push_str(&format!(
            r#"
  {}:
    image: registry.gitlab.com/simpelv2/{}:staging
    environment:
      - RUST_LOG=info
      - DATABASE_URL=${{DATABASE_URL}}
      - VAULT_ADDR=${{VAULT_ADDR}}
      - VAULT_TOKEN=${{VAULT_TOKEN}}
    networks:
      - simpel-network
"#,
            service, service
        ));
    }

    compose.push_str(
        r#"
networks:
  simpel-network:
    external: true
"#,
    );

    compose
}

fn generate_prod_compose(services: &[&str]) -> String {
    // Production configuration with high availability
    let mut compose = String::from(
        r#"version: '3.8'

services:
"#,
    );

    for service in services {
        compose.push_str(&format!(
            r#"
  {}:
    image: registry.gitlab.com/simpelv2/{}:latest
    deploy:
      replicas: 3
      restart_policy:
        condition: on-failure
        delay: 5s
        max_attempts: 3
      resources:
        limits:
          cpus: '1.0'
          memory: 1G
        reservations:
          cpus: '0.5'
          memory: 512M
    environment:
      - RUST_LOG=warn
      - DATABASE_URL=${{DATABASE_URL}}
      - VAULT_ADDR=${{VAULT_ADDR}}
      - VAULT_TOKEN=${{VAULT_TOKEN}}
    networks:
      - simpel-network
    secrets:
      - db_password
      - vault_token
"#,
            service, service
        ));
    }

    compose.push_str(
        r#"
networks:
  simpel-network:
    external: true

secrets:
  db_password:
    external: true
  vault_token:
    external: true
"#,
    );

    compose
}

// Kubernetes manifests generation (migrated from Python)
#[allow(dead_code)]
pub async fn generate_k8s_manifests(namespace: &str, services: &[&str]) -> Result<()> {
    println!("{}", "☸️  Generating Kubernetes manifests...".bright_cyan());

    let k8s_dir = format!("infra/k8s/{}", namespace);
    fs::create_dir_all(&k8s_dir)?;

    // Generate namespace
    generate_k8s_namespace(namespace, &k8s_dir)?;

    // Generate configmap
    generate_k8s_configmap(namespace, &k8s_dir)?;

    // Generate secrets
    generate_k8s_secrets(namespace, &k8s_dir)?;

    // Generate services and deployments
    for service in services {
        generate_k8s_service(service, namespace, &k8s_dir)?;
        generate_k8s_deployment(service, namespace, &k8s_dir)?;
    }

    // Generate ingress
    generate_k8s_ingress(namespace, services, &k8s_dir)?;

    // Generate kustomization
    generate_k8s_kustomization(services, &k8s_dir)?;

    println!("✅ Kubernetes manifests generated in: {}", k8s_dir);
    Ok(())
}

#[allow(dead_code)]
fn generate_k8s_namespace(namespace: &str, k8s_dir: &str) -> Result<()> {
    let content = format!(
        r#"apiVersion: v1
kind: Namespace
metadata:
  name: {}
  labels:
    name: {}
    project: simpelv2
"#,
        namespace, namespace
    );

    fs::write(format!("{}/namespace.yaml", k8s_dir), content)?;
    Ok(())
}

#[allow(dead_code)]
fn generate_k8s_configmap(namespace: &str, k8s_dir: &str) -> Result<()> {
    let content = format!(
        r#"apiVersion: v1
kind: ConfigMap
metadata:
  name: simpel-config
  namespace: {}
data:
  RUST_LOG: "info"
  DATABASE_HOST: "postgres-service"
  DATABASE_PORT: "5432"
  VAULT_ADDR: "http://vault-service:8200"
  REDIS_URL: "redis://redis-service:6379"
"#,
        namespace
    );

    fs::write(format!("{}/configmap.yaml", k8s_dir), content)?;
    Ok(())
}

#[allow(dead_code)]
fn generate_k8s_secrets(namespace: &str, k8s_dir: &str) -> Result<()> {
    let content = format!(
        r#"apiVersion: v1
kind: Secret
metadata:
  name: simpel-secrets
  namespace: {}
type: Opaque
data:
  # Base64 encoded values - replace with actual values
  DATABASE_PASSWORD: cGFzc3dvcmQ=
  VAULT_TOKEN: dG9rZW4=
  JWT_SECRET: c2VjcmV0
"#,
        namespace
    );

    fs::write(format!("{}/secrets.yaml", k8s_dir), content)?;
    Ok(())
}

#[allow(dead_code)]
fn generate_k8s_service(service: &str, namespace: &str, k8s_dir: &str) -> Result<()> {
    let port = get_service_port(service);
    let content = format!(
        r#"apiVersion: v1
kind: Service
metadata:
  name: {}-service
  namespace: {}
  labels:
    app: {}
spec:
  selector:
    app: {}
  ports:
    - protocol: TCP
      port: 80
      targetPort: {}
  type: ClusterIP
"#,
        service, namespace, service, service, port
    );

    fs::write(format!("{}/{}-service.yaml", k8s_dir, service), content)?;
    Ok(())
}

#[allow(dead_code)]
fn generate_k8s_deployment(service: &str, namespace: &str, k8s_dir: &str) -> Result<()> {
    let port = get_service_port(service);
    let content = format!(
        r#"apiVersion: apps/v1
kind: Deployment
metadata:
  name: {}-deployment
  namespace: {}
  labels:
    app: {}
spec:
  replicas: 2
  selector:
    matchLabels:
      app: {}
  template:
    metadata:
      labels:
        app: {}
    spec:
      containers:
      - name: {}
        image: registry.gitlab.com/simpelv2/layanan-{}:latest
        ports:
        - containerPort: {}
        env:
        - name: RUST_LOG
          valueFrom:
            configMapKeyRef:
              name: simpel-config
              key: RUST_LOG
        - name: DATABASE_URL
          value: postgresql://$(DATABASE_USER):$(DATABASE_PASSWORD)@$(DATABASE_HOST):$(DATABASE_PORT)/simpel_{}
        - name: DATABASE_USER
          value: "simpel"
        - name: DATABASE_PASSWORD
          valueFrom:
            secretKeyRef:
              name: simpel-secrets
              key: DATABASE_PASSWORD
        - name: DATABASE_HOST
          valueFrom:
            configMapKeyRef:
              name: simpel-config
              key: DATABASE_HOST
        - name: DATABASE_PORT
          valueFrom:
            configMapKeyRef:
              name: simpel-config
              key: DATABASE_PORT
        - name: VAULT_ADDR
          valueFrom:
            configMapKeyRef:
              name: simpel-config
              key: VAULT_ADDR
        - name: VAULT_TOKEN
          valueFrom:
            secretKeyRef:
              name: simpel-secrets
              key: VAULT_TOKEN
        resources:
          limits:
            cpu: 500m
            memory: 512Mi
          requests:
            cpu: 250m
            memory: 256Mi
        livenessProbe:
          httpGet:
            path: /health
            port: {}
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: {}
          initialDelaySeconds: 5
          periodSeconds: 5
"#,
        service, namespace, service, service, service, service, service, port, service, port, port
    );

    fs::write(format!("{}/{}-deployment.yaml", k8s_dir, service), content)?;
    Ok(())
}

#[allow(dead_code)]
fn generate_k8s_ingress(namespace: &str, services: &[&str], k8s_dir: &str) -> Result<()> {
    let mut content = format!(
        r#"apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: simpel-ingress
  namespace: {}
  annotations:
    nginx.ingress.kubernetes.io/rewrite-target: /
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
    cert-manager.io/cluster-issuer: "letsencrypt-prod"
spec:
  tls:
  - hosts:
    - simpel.local
    secretName: simpel-tls
  rules:
  - host: simpel.local
    http:
      paths:
"#,
        namespace
    );

    for service in services {
        content.push_str(&format!(
            r#"      - path: /api/{}
        pathType: Prefix
        backend:
          service:
            name: {}-service
            port:
              number: 80
"#,
            service, service
        ));
    }

    fs::write(format!("{}/ingress.yaml", k8s_dir), content)?;
    Ok(())
}

#[allow(dead_code)]
fn generate_k8s_kustomization(services: &[&str], k8s_dir: &str) -> Result<()> {
    let mut content = String::from(
        r#"apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization

resources:
  - namespace.yaml
  - configmap.yaml
  - secrets.yaml
  - ingress.yaml
"#,
    );

    for service in services {
        content.push_str(&format!("  - {}-service.yaml\n", service));
        content.push_str(&format!("  - {}-deployment.yaml\n", service));
    }

    content.push_str(
        r#"
commonLabels:
  project: simpelv2

images:
"#,
    );

    for service in services {
        content.push_str(&format!(
            r#"  - name: registry.gitlab.com/simpelv2/layanan-{}
    newTag: latest
"#,
            service
        ));
    }

    fs::write(format!("{}/kustomization.yaml", k8s_dir), content)?;
    Ok(())
}

// GitLab CI advanced generation (migrated from Python)
#[allow(dead_code)]
pub async fn generate_advanced_gitlab_ci(features: &[&str]) -> Result<()> {
    println!(
        "{}",
        "🚀 Generating advanced GitLab CI configuration...".bright_green()
    );

    let mut content = String::from(
        r#"# Advanced GitLab CI configuration for SIMPelv2
# Generated by SIMPel CLI

variables:
  DOCKER_DRIVER: overlay2
  DOCKER_TLS_CERTDIR: "/certs"
  CARGO_HOME: $CI_PROJECT_DIR/.cargo
  REGISTRY: $CI_REGISTRY_IMAGE
  RUST_BACKTRACE: 1

stages:
  - validate
  - test
  - security
  - build
  - package
  - deploy

# Templates
.rust_cache: &rust_cache
  cache:
    key: "${CI_JOB_NAME}-${CI_COMMIT_REF_SLUG}"
    paths:
      - .cargo/
      - target/

.docker_setup: &docker_setup
  image: docker:latest
  services:
    - docker:dind
  before_script:
    - docker login -u $CI_REGISTRY_USER -p $CI_REGISTRY_PASSWORD $CI_REGISTRY

"#,
    );

    // Add validation stage
    if features.contains(&"validation") {
        content.push_str(
            r#"
# Validation Jobs
validate:yaml:
  stage: validate
  image: alpine:latest
  before_script:
    - apk add --no-cache yamllint
  script:
    - yamllint -d relaxed .gitlab-ci.yml
    - find . -name "*.yaml" -o -name "*.yml" | xargs yamllint -d relaxed
  only:
    changes:
      - "**/*.yml"
      - "**/*.yaml"

validate:toml:
  stage: validate
  image: rust:latest
  script:
    - cargo check --workspace
  <<: *rust_cache

"#,
        );
    }

    // Add comprehensive testing
    if features.contains(&"testing") {
        content.push_str(
            r#"
# Testing Jobs
test:unit:
  stage: test
  image: rust:latest
  script:
    - cargo test --workspace --lib
  coverage: '/^\d+\.\d+% coverage/'
  artifacts:
    reports:
      coverage_report:
        coverage_format: cobertura
        path: coverage.xml
  <<: *rust_cache

test:integration:
  stage: test
  image: rust:latest
  services:
    - postgres:13
    - redis:alpine
  variables:
    DATABASE_URL: "postgresql://postgres:password@postgres:5432/test_db"
    REDIS_URL: "redis://redis:6379"
  script:
    - cargo test --workspace --test '*'
  <<: *rust_cache

test:performance:
  stage: test
  image: rust:latest
  script:
    - cargo bench --workspace
  artifacts:
    paths:
      - target/criterion/
    expire_in: 1 week
  <<: *rust_cache

"#,
        );
    }

    // Add security scanning
    if features.contains(&"security") {
        content.push_str(
            r#"
# Security Jobs
security:cargo-audit:
  stage: security
  image: rust:latest
  before_script:
    - cargo install cargo-audit
  script:
    - cargo audit
  allow_failure: true

security:dependency-check:
  stage: security
  image: owasp/dependency-check:latest
  script:
    - /usr/share/dependency-check/bin/dependency-check.sh
      --project "SIMPelv2"
      --scan .
      --format XML
      --out reports/
  artifacts:
    paths:
      - reports/
    expire_in: 1 week
  allow_failure: true

security:container-scan:
  stage: security
  image: docker:stable
  services:
    - docker:stable-dind
  script:
    - docker build -t $CI_PROJECT_NAME:$CI_COMMIT_SHA .
    - docker run --rm -v /var/run/docker.sock:/var/run/docker.sock
      -v $PWD:/tmp/clair2-db aquasec/trivy:latest
      image --format template --template "@contrib/html.tpl"
      -o /tmp/clair2-db/report.html $CI_PROJECT_NAME:$CI_COMMIT_SHA
  artifacts:
    paths:
      - report.html
    expire_in: 1 week
  allow_failure: true

"#,
        );
    }

    // Add build and packaging
    content.push_str(
        r#"
# Build Jobs
build:debug:
  stage: build
  image: rust:latest
  script:
    - cargo build --workspace
  artifacts:
    paths:
      - target/debug/
    expire_in: 1 hour
  <<: *rust_cache
  except:
    - main
    - tags

build:release:
  stage: build
  image: rust:latest
  script:
    - cargo build --workspace --release
  artifacts:
    paths:
      - target/release/
    expire_in: 1 day
  <<: *rust_cache
  only:
    - main
    - tags

# Package Jobs
package:docker:
  stage: package
  <<: *docker_setup
  script:
    - docker build -t $REGISTRY:$CI_COMMIT_SHA .
    - docker push $REGISTRY:$CI_COMMIT_SHA
    - |
      if [ "$CI_COMMIT_BRANCH" == "main" ]; then
        docker tag $REGISTRY:$CI_COMMIT_SHA $REGISTRY:latest
        docker push $REGISTRY:latest
      fi
  only:
    - main
    - tags

package:helm:
  stage: package
  image: alpine/helm:latest
  script:
    - helm package infra/helm/simpelv2
    - helm repo index . --url $CI_PROJECT_URL/packages/helm
  artifacts:
    paths:
      - "*.tgz"
      - index.yaml
  only:
    - main
    - tags

"#,
    );

    // Add deployment stages
    if features.contains(&"deployment") {
        content.push_str(
            r#"
# Deployment Jobs
deploy:staging:
  stage: deploy
  image: bitnami/kubectl:latest
  environment:
    name: staging
    url: https://staging.simpel.local
  script:
    - kubectl config use-context staging
    - helm upgrade --install simpelv2-staging infra/helm/simpelv2
      --namespace staging
      --values infra/helm/values-staging.yaml
      --set image.tag=$CI_COMMIT_SHA
  only:
    - main

deploy:production:
  stage: deploy
  image: bitnami/kubectl:latest
  environment:
    name: production
    url: https://simpel.local
  script:
    - kubectl config use-context production
    - helm upgrade --install simpelv2 infra/helm/simpelv2
      --namespace production
      --values infra/helm/values-production.yaml
      --set image.tag=$CI_COMMIT_SHA
  when: manual
  only:
    - tags

"#,
        );
    }

    fs::write(".gitlab-ci.yml", content)?;
    println!("📁 Created: .gitlab-ci.yml");
    Ok(())
}

// Configuration generators
#[allow(dead_code)]
pub async fn generate_config_files(config_type: &str, environment: &str) -> Result<()> {
    println!(
        "{}",
        format!(
            "⚙️  Generating {} configuration for {}...",
            config_type, environment
        )
        .bright_yellow()
    );

    match config_type {
        "nginx" => generate_nginx_config(environment)?,
        "vault" => generate_vault_config_file(environment)?,
        "postgres" => generate_postgres_config(environment)?,
        "monitoring" => generate_monitoring_config(environment)?,
        _ => {
            println!("❌ Unknown configuration type: {}", config_type);
            return Ok(());
        }
    }

    println!("✅ Configuration files generated successfully!");
    Ok(())
}

#[allow(dead_code)]
fn generate_nginx_config(environment: &str) -> Result<()> {
    let config_dir = format!("infra/nginx/{}", environment);
    fs::create_dir_all(&config_dir)?;

    let content = match environment {
        "dev" => include_str!("templates/nginx-dev.conf"),
        "staging" => include_str!("templates/nginx-staging.conf"),
        "prod" => include_str!("templates/nginx-prod.conf"),
        _ => include_str!("templates/nginx-dev.conf"),
    };

    fs::write(format!("{}/nginx.conf", config_dir), content)?;
    println!("📁 Created: {}/nginx.conf", config_dir);
    Ok(())
}

#[allow(dead_code)]
fn generate_vault_config_file(environment: &str) -> Result<()> {
    let config_dir = format!("infra/vault/{}", environment);
    fs::create_dir_all(&config_dir)?;

    let content = match environment {
        "dev" => crate::handlers::vault::generate_dev_config(),
        "staging" => crate::handlers::vault::generate_staging_config(),
        "prod" => crate::handlers::vault::generate_prod_config(),
        _ => crate::handlers::vault::generate_dev_config(),
    };

    fs::write(format!("{}/vault.hcl", config_dir), content)?;
    println!("📁 Created: {}/vault.hcl", config_dir);
    Ok(())
}

#[allow(dead_code)]
fn generate_postgres_config(environment: &str) -> Result<()> {
    let config_dir = format!("infra/postgres/{}", environment);
    fs::create_dir_all(&config_dir)?;

    let content = match environment {
        "dev" => create_postgres_dev_config(),
        "staging" => create_postgres_staging_config(),
        "prod" => create_postgres_prod_config(),
        _ => create_postgres_dev_config(),
    };

    fs::write(format!("{}/postgresql.conf", config_dir), content)?;
    println!("📁 Created: {}/postgresql.conf", config_dir);
    Ok(())
}

#[allow(dead_code)]
fn generate_monitoring_config(environment: &str) -> Result<()> {
    let config_dir = format!("infra/monitoring/{}", environment);
    fs::create_dir_all(&config_dir)?;

    // Generate Prometheus config
    let prometheus_config = create_prometheus_config(environment);
    fs::write(format!("{}/prometheus.yml", config_dir), prometheus_config)?;

    // Generate Grafana config
    let grafana_config = create_grafana_config(environment);
    fs::write(format!("{}/grafana.ini", config_dir), grafana_config)?;

    println!("📁 Created monitoring configs in: {}", config_dir);
    Ok(())
}

// Helper functions
fn get_service_port(service: &str) -> u16 {
    match service {
        "keamanan" => 8001,
        "dasbor" => 8002,
        "aset" => 8003,
        "audit" => 8004,
        "ai" => 8005,
        "dokumen" => 8006,
        "laporan" => 8007,
        "integrasi" => 8008,
        "konfigurasi" => 8009,
        "bantuan" => 8010,
        _ => 8080,
    }
}

#[allow(dead_code)]
fn create_postgres_dev_config() -> String {
    r#"# PostgreSQL Development Configuration
listen_addresses = '*'
port = 5432
max_connections = 100
shared_buffers = 128MB
effective_cache_size = 4GB
maintenance_work_mem = 64MB
checkpoint_completion_target = 0.9
wal_buffers = 16MB
default_statistics_target = 100
random_page_cost = 1.1
effective_io_concurrency = 200
work_mem = 4MB
min_wal_size = 1GB
max_wal_size = 4GB
log_destination = 'stderr'
logging_collector = on
log_directory = 'logs'
log_filename = 'postgresql-%a.log'
log_truncate_on_rotation = on
log_rotation_age = 1d
log_rotation_size = 10MB
log_line_prefix = '%t [%p]: [%l-1] user=%u,db=%d,app=%a,client=%h '
log_checkpoints = on
log_connections = on
log_disconnections = on
log_lock_waits = on
log_temp_files = 0
log_autovacuum_min_duration = 0
log_error_verbosity = default
"#
    .to_string()
}

#[allow(dead_code)]
fn create_postgres_staging_config() -> String {
    r#"# PostgreSQL Staging Configuration
listen_addresses = '*'
port = 5432
max_connections = 200
shared_buffers = 256MB
effective_cache_size = 8GB
maintenance_work_mem = 128MB
checkpoint_completion_target = 0.9
wal_buffers = 32MB
default_statistics_target = 100
random_page_cost = 1.1
effective_io_concurrency = 200
work_mem = 8MB
min_wal_size = 2GB
max_wal_size = 8GB
log_destination = 'stderr'
logging_collector = on
log_directory = 'logs'
log_filename = 'postgresql-%Y-%m-%d_%H%M%S.log'
log_truncate_on_rotation = off
log_rotation_age = 1d
log_rotation_size = 100MB
log_line_prefix = '%t [%p]: [%l-1] user=%u,db=%d,app=%a,client=%h '
log_checkpoints = on
log_connections = off
log_disconnections = off
log_lock_waits = on
log_temp_files = 0
log_autovacuum_min_duration = 0
log_error_verbosity = default
"#
    .to_string()
}

#[allow(dead_code)]
fn create_postgres_prod_config() -> String {
    r#"# PostgreSQL Production Configuration
listen_addresses = '*'
port = 5432
max_connections = 500
shared_buffers = 1GB
effective_cache_size = 16GB
maintenance_work_mem = 256MB
checkpoint_completion_target = 0.9
wal_buffers = 64MB
default_statistics_target = 100
random_page_cost = 1.1
effective_io_concurrency = 300
work_mem = 16MB
min_wal_size = 4GB
max_wal_size = 16GB
log_destination = 'stderr'
logging_collector = on
log_directory = 'logs'
log_filename = 'postgresql-%Y-%m-%d_%H%M%S.log'
log_truncate_on_rotation = off
log_rotation_age = 1d
log_rotation_size = 1GB
log_line_prefix = '%t [%p]: [%l-1] user=%u,db=%d,app=%a,client=%h '
log_checkpoints = off
log_connections = off
log_disconnections = off
log_lock_waits = on
log_temp_files = 0
log_autovacuum_min_duration = -1
log_error_verbosity = terse
archive_mode = on
archive_command = 'cp %p /var/lib/postgresql/archive/%f'
"#
    .to_string()
}

#[allow(dead_code)]
fn create_prometheus_config(environment: &str) -> String {
    format!(
        r#"global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files:
  - "rules/*.yml"

alerting:
  alertmanagers:
    - static_configs:
        - targets:
          - alertmanager:9093

scrape_configs:
  - job_name: 'prometheus'
    static_configs:
      - targets: ['localhost:9090']

  - job_name: 'node-exporter'
    static_configs:
      - targets: ['node-exporter:9100']

  - job_name: 'simpelv2-services'
    static_configs:
      - targets:
        - 'keamanan-service:8001'
        - 'dasbor-service:8002'
        - 'aset-service:8003'
        - 'ai-service:8005'
    metrics_path: '/metrics'
    scrape_interval: 30s

  - job_name: 'postgres-exporter'
    static_configs:
      - targets: ['postgres-exporter:9187']

  - job_name: 'vault'
    static_configs:
      - targets: ['vault:8200']
    metrics_path: '/v1/sys/metrics'
    params:
      format: ['prometheus']
    bearer_token_file: /etc/prometheus/vault_token

environment: {}
"#,
        environment
    )
}

#[allow(dead_code)]
fn create_grafana_config(environment: &str) -> String {
    format!(
        r#"[server]
protocol = http
http_port = 3000
domain = grafana.{}.simpel.local
root_url = http://grafana.{}.simpel.local:3000

[database]
type = postgres
host = postgres:5432
name = grafana
user = grafana
password = $__env{{GRAFANA_DB_PASSWORD}}

[security]
admin_user = admin
admin_password = $__env{{GRAFANA_ADMIN_PASSWORD}}
secret_key = $__env{{GRAFANA_SECRET_KEY}}

[auth]
disable_login_form = false

[auth.basic]
enabled = true

[auth.anonymous]
enabled = false

[log]
mode = console
level = info

[metrics]
enabled = true

[alerting]
enabled = true
execute_alerts = true

[unified_alerting]
enabled = true
"#,
        environment, environment
    )
}
