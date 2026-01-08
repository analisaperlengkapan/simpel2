# 🚀 Deployment Guide - SIMPelv2 Frontend

Panduan lengkap untuk deployment portal dan microfrontends SIMPelv2.

## 📖 Table of Contents

1. [Overview](#overview)
2. [Prerequisites](#prerequisites)
3. [Build Process](#build-process)
4. [Local Development](#local-development)
5. [Staging Deployment](#staging-deployment)
6. [Production Deployment](#production-deployment)
7. [Infrastructure Setup](#infrastructure-setup)
8. [Monitoring](#monitoring)
9. [Rollback Procedures](#rollback-procedures)
10. [Troubleshooting](#troubleshooting)

---

## Overview

SIMPelv2 frontend consists of:
- **Portal**: Main gateway and authentication (port 8080)
- **11 Microfrontends**: Domain-specific applications
- **Shared Library**: Common components and utilities

### Deployment Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Load Balancer                         │
│                  (SSL Termination)                       │
└────────────────────┬────────────────────────────────────┘
                     │
         ┌───────────┴───────────┐
         │                       │
         ▼                       ▼
┌─────────────────┐    ┌─────────────────┐
│  Nginx (Port 80)│    │  Nginx (Port 80)│
│  - Portal       │    │  - Portal       │
│  - Microfrontends│   │  - Microfrontends│
└────────┬────────┘    └────────┬────────┘
         │                       │
         └───────────┬───────────┘
                     │
         ┌───────────┴───────────┐
         │                       │
         ▼                       ▼
┌─────────────────┐    ┌─────────────────┐
│  Envoy Gateway  │    │  Envoy Gateway  │
│  (API Routing)  │    │  (API Routing)  │
└─────────────────┘    └─────────────────┘
```

---

## Prerequisites

### Development Machine

```bash
# Rust toolchain
rustup --version  # 1.75+

# WASM target
rustup target add wasm32-unknown-unknown

# Trunk (WASM bundler)
cargo install trunk

# wasm-bindgen-cli
cargo install wasm-bindgen-cli

# wasm-opt (optional, for optimization)
cargo install wasm-opt
```

### Server Requirements

- **OS**: Ubuntu 22.04 LTS or RHEL 8+
- **RAM**: Minimum 4GB (8GB recommended)
- **CPU**: 2 cores minimum (4 cores recommended)
- **Disk**: 20GB minimum
- **Network**: Public IP with ports 80, 443 open

### Software Stack

- **Nginx**: 1.24+
- **Docker**: 24.0+
- **Kubernetes**: 1.28+ (MicroK8s)
- **Cert-Manager**: For SSL certificates

---

## Build Process

### 1. Build Portal

```bash
cd antarmuka/portal

# Development build
trunk build

# Production build
trunk build --release

# Output in dist/
ls dist/
# index.html
# portal-*.wasm
# portal-*.js
# styles/
```

### 2. Build All Microfrontends

```bash
# Build script for all microfrontends
#!/bin

MODULES=(
    "badiklat"
    "datun"
    "intel"
    "pidum"
    "pidsus"
    "pidmil"
    "pengawasan"
    "pemulihan_aset
    "pembinaan/keuangan"
    "pembinaan/perencanaan"
    "pembinaan/perlengkapan"
)

for module in "${MODULES[@]}"; do
    echo "Building $module..."
    cd "antarmuka/$module"
    trunk build --release
    cd ../..
done

echo "All builds complete!"
```

### 3. Optimize WASM

```bash
# Optimize WASM files for production
for file in dist/*.wasm; do
    wasm-opt -Oz "$file" -o "${file%.wasm}.opt.wasm"
    mv "${file%.wasm}.opt.wasm" "$file"
done
```

### 4. Generate Source Maps

```bash
# Enable source maps in Trunk.toml
[build]
release = true
minify = "on_release"
source_maps = true
```

---

## Local Development

### 1. Start Portal

```bash
cd antarmuka/portal
trunk serve --port 8080 --open
```

### 2. Start Microfrontend

```bash
cd antarmuka/badiklat
trunk serve --port 8081 --open
```

### 3. Start Backend Services

```bash
# Start Authenc
cd infra/authenc
cargo run --release

# Start API Gateway
cd infra/gerbang
docker-compose up -d
```

### 4. Development Workflow

```bash
# Watch for changes and rebuild
trunk watch

# Run tests
cargo test

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy --all-targets
```

---

## Staging Deployment

### 1. Build for Staging

```bash
# Set staging environment
export PORTAL_URL=https://portal-staging.simpelv2.kejaksaan.go.id
export API_URL=https://api-staging.simpelv2.kejaksaan.go.id

# Build all
make build-all-fe
```

### 2. Deploy to Staging Server

```bash
# Copy files to staging server
rsync -avz --delete \
    antarmuka/portal/dist/ \
    deploy@staging:/var/www/portal/

# Deploy microfrontends
for module in badiklat datun intel pidum pidsus pidmil pengawasan pemulihan_aset; do
    rsync -avz --delete \
        "antarmuka/$module/dist/" \
        "deploy@staging:/var/www/$module/"
done
```

### 3. Restart Nginx

```bash
ssh deploy@staging "sudo systemctl reload nginx"
```

### 4. Verify Deployment

```bash
# Check portal
curl -I https://portal-staging.simpelv2.kejaksaan.go.id

# Check microfrontend
curl -I https://badiklat-staging.simpelv2.kejaksaan.go.id

# Check health
curl https://portal-staging.simpelv2.kejaksaan.go.id/healthz
```

---

## Production Deployment

### 1. Pre-Deployment Checklist

- [ ] All tests passing
- [ ] Code reviewed and approved
- [ ] Staging deployment successful
- [ ] Performance tested
- [ ] Security scan completed
- [ ] Backup current production
- [ ] Maintenance window scheduled
- [ ] Rollback plan prepared

### 2. Build for Production

```bash
# Set production environment
export PORTAL_URL=https://portal.simpelv2.kejaksaan.go.id
export API_URL=https://api.simpelv2.kejaksaan.go.id

# Build with optimizations
trunk build --release

# Verify build
ls -lh dist/
```

### 3. Docker Build

```bash
# Build Docker image
docker build -t registry.kejaksaan.go.id/simpelv2/portal:v1.0.0 .

# Push to registry
docker push registry.kejaksaan.go.id/simpelv2/portal:v1.0.0

# Tag as latest
docker tag registry.kejaksaan.go.id/simpelv2/portal:v1.0.0 \
           registry.kejaksaan.go.id/simpelv2/portal:latest
docker push registry.kejaksaan.go.id/simpelv2/portal:latest
```

### 4. Kubernetes Deployment

```bash
# Apply deployment
kubectl apply -f k8s/portal-deployment.yaml

# Check rollout status
kubectl rollout status deployment/portal -n simpelv2

# Verify pods
kubectl get pods -n simpelv2 -l app=portal

# Check logs
kubectl logs -f deployment/portal -n simpelv2
```

### 5. Blue-Green Deployment

```bash
# Deploy green environment
kubectl apply -f k8s/portal-deployment-green.yaml

# Wait for green to be ready
kubectl wait --for=condition=available deployment/portal-green -n simpelv2

# Switch traffic to green
kubectl patch service portal -n simpelv2 \
    -p '{"spec":{"selector":{"version":"green"}}}'

# Monitor for issues
kubectl logs -f deployment/portal-green -n simpelv2

# If successful, remove blue
kubectl delete deployment portal-blue -n simpelv2
```

### 6. Post-Deployment Verification

```bash
# Check all endpoints
./scripts/verify-deployment.sh

# Monitor error rates
kubectl logs -f deployment/portal -n simpelv2 | grep ERROR

# Check performance metrics
curl https://portal.simpelv2.kejaksaan.go.id/metrics
```

---

## Infrastructure Setup

### 1. Nginx Configuration

```nginx
# /etc/nginx/sites-available/simpelv2

# Portal
server {
    listen 443 ssl http2;
    server_name portal.simpelv2.kejaksaan.go.id;

    ssl_certificate /etc/letsencrypt/live/portal.simpelv2.kejaksaan.go.id/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/portal.simpelv2.kejaksaan.go.id/privkey.pem;

    root /var/www/portal;
    index index.html;

    # WASM mime type
    types {
        application/wasm wasm;
    }

    # Gzip compression
    gzip on;
    gzip_types text/css application/javascript application/wasm;
    gzip_min_length 1000;

    # Security headers
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;

    # SPA routing
    location / {
        try_files $uri $uri/ /index.html;
    }

    # API proxy
    location /api/ {
        proxy_pass https://api.simpelv2.kejaksaan.go.id/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # Cache static assets
    location ~* \.(wasm|js|css)$ {
        expires 1y;
        add_header Cache-Control "public, immutable";
    }

    # Health check
    location /healthz {
        access_log off;
        return 200 "healthy\n";
        add_header Content-Type text/plain;
    }
}

# Redirect HTTP to HTTPS
server {
    listen 80;
    server_name portal.simpelv2.kejaksaan.go.id;
    return 301 https://$server_name$request_uri;
}
```

### 2. SSL Certificate Setup

```bash
# Install certbot
sudo apt install certbot python3-certbot-nginx

# Obtain certificate
sudo certbot --nginx -d portal.simpelv2.kejaksaan.go.id

# Auto-renewal
sudo certbot renew --dry-run

# Add to crontab
0 0 * * * certbot renew --quiet
```

### 3. Kubernetes Setup

```yaml
# k8s/portal-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: portal
  namespace: simpelv2
spec:
  replicas: 3
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
  selector:
    mas:
      app: portal
  template:
    metadata:
      labels:
        app: portal
    spec:
      containers:
      - name: portal
        image: registry.kejaksaan.go.id/simpelv2/portal:latest
        ports:
        - containerPort: 80
        resources:
          requests:
            memory: "256Mi"
            cpu: "200m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /healthz
            port: 80
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /healthz
            port: 80
          initialDelaySeconds: 5
          periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: portal
  namespace: simpelv2
spec:
  selector:
    app: portal
  ports:
  - port: 80
    targetPort: 80
  type: ClusterIP
---
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: portal
  namespace: simpelv2
  annotations:
    cert-manager.io/cluster-issuer: "letsencrypt-prod"
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
spec:
  tls:
  - hosts:
    - portal.simpelv2.kejaksaan.go.id
    secretName: portal-tls
  rules:
  - host: portal.simpelv2.kejaksaan.go.id
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: portal
            port:
              number: 80
```



---

## Monitoring

### 1. Application Metrics

```bash
# Prometheus metrics endpoint
curl https://portal.simpelv2.kejaksaan.go.id/metrics

# Key metrics to monitor:
# - http_requests_total
# - http_request_duration_seconds
# - wasm_load_time_seconds
# - active_users
# - error_rate
```

### 2. Logging

```bash
# View application logs
kubectl logs -f deployment/portal -n simpelv2

# Filter errors
kubectl logs deployment/portal -n simpelv2 | grep ERROR

# Tail logs from all pods
kubectl logs -f -l app=portal -n simpelv2 --all-containers=true
```

### 3. Health Checks

```bash
# Portal health
curl https://portal.simpelv2.kejaksaan.go.id/healthz

# Microfrontend health
for module in badiklat datun intel pidum pidsus pidmil pengawasan pemulihan_aset; do
    echo "Checking $module..."
    curl -I "https://$module.simpelv2.kejaksaan.go.id/healthz"
done
```

### 4. Performance Monitoring

```bash
# Lighthouse CI
lighthouse https://portal.simpelv2.kejaksaan.go.id \
    --output=json \
    --output-path=./lighthouse-report.json

# WebPageTest
webpagetest test https://portal.simpelv2.kejaksaan.go.id \
    --location=Jakarta:Chrome \
    --runs=3
```

### 5. Error Tracking

```javascript
// Sentry integration (in index.html)
<script src="https://browser.sentry-cdn.com/7.x.x/bundle.min.js"></script>
<script>
  Sentry.init({
    dsn: "https://your-dsn@sentry.io/project-id",
    environment: "production",
    release: "portal@1.0.0",
    tracesSampleRate: 0.1,
  });
</script>
```

---

## Rollback Procedures

### 1. Kubernetes Rollback

```bash
# View rollout history
kubectl rollout history deployment/portal -n simpelv2

# Rollback to previous version
kubectl rollout undo deployment/portal -n simpelv2

# Rollback to specific revision
kubectl rollout undo deployment/portal -n simpelv2 --to-revision=2

# Check rollback status
kubectl rollout status deployment/portal -n simpelv2
```

### 2. Docker Rollback

```bash
# List available images
docker images registry.kejaksaan.go.id/simpelv2/portal

# Deploy previous version
kubectl set image deployment/portal \
    portal=registry.kejaksaan.go.id/simpelv2/portal:v0.9.0 \
    -n simpelv2
```

### 3. Nginx Rollback

```bash
# Restore previous files
ssh deploy@production "
    sudo rm -rf /var/www/portal
    sudo cp -r /var/www/portal.backup /var/www/portal
    sudo systemctl reload nginx
"
```

### 4. Database Rollback

```bash
# If schema changes were made
psql -U postgres -d simpelv2 < backup/schema_v0.9.0.sql

# Verify
psql -U postgres -d simpelv2 -c "SELECT version FROM schema_migrations;"
```

---

## Troubleshooting

### Issue: WASM Not Loading

**Symptoms:**
- Blank page
- Console error: "Failed to instantiate WASM module"

**Solutions:**

1. Check WASM mime type:
```nginx
types {
    application/wasm wasm;
}
```

2. Verify file exists:
```bash
ls -lh /var/www/portal/*.wasm
```

3. Check nginx error log:
```bash
tail -f /var/log/nginx/error.log
```

4. Test WASM download:
```bash
curl -I https://portal.simpelv2.kejaksaan.go.id/portal-*.wasm
```

---

### Issue: 502 Bad Gateway

**Symptoms:**
- Nginx returns 502 error
- API calls failing

**Solutions:**

1. Check backend services:
```bash
kubectl get pods -n simpelv2
kubectl logs deployment/authenc -n simpelv2
```

2. Verify Envoy gateway:
```bash
kubectl logs deployment/envoy -n simpelv2
```

3. Check nginx upstream:
```bash
nginx -t
systemctl status nginx
```

4. Test backend directly:
```bash
curl http://localhost:8088/health
```

---

### Issue: SSL Certificate Errors

**Symptoms:**
- Browser shows "Not Secure"
- Certificate expired warning

**Solutions:**

1. Check certificate expiry:
```bash
echo | openssl s_client -servername portal.simpelv2.kejaksaan.go.id \
    -connect portal.simpelv2.kejaksaan.go.id:443 2>/dev/null | \
    openssl x509 -noout -dates
```

2. Renew certificate:
```bash
sudo certbot renew --force-renewal
sudo systemctl reload nginx
```

3. Verify certificate chain:
```bash
openssl s_client -connect portal.simpelv2.kejaksaan.go.id:443 -showcerts
```

---

### Issue: High Memory Usage

**Symptoms:**
- Pods being OOMKilled
- Slow performance

**Solutions:**

1. Check resource usage:
```bash
kubectl top pods -n simpelv2
```

2. Increase memory limits:
```yaml
resources:
  limits:
    memory: "1Gi"  # Increased from 512Mi
```

3. Optimize WASM:
```bash
wasm-opt -Oz input.wasm -o output.wasm
```

4. Enable gzip compression:
```nginx
gzip on;
gzip_types application/wasm;
```

---

### Issue: Slow Page Load

**Symptoms:**
- Long Time to Interactive (TTI)
- Large WASM bundle

**Solutions:**

1. Analyze bundle size:
```bash
ls -lh dist/*.wasm
```

2. Enable code splitting:
```rust
// Use lazy loading for routes
<Route path="/admin" view=|| {
    lazy(|| import("./pages/admin.rs"))
} />
```

3. Optimize images:
```bash
# Convert to WebP
for img in images/*.png; do
    cwebp -q 80 "$img" -o "${img%.png}.webp"
done
```

4. Enable CDN caching:
```nginx
location ~* \.(wasm|js|css)$ {
    expires 1y;
    add_header Cache-Control "public, immutable";
}
```

---

## CI/CD Pipeline

### GitLab CI Configuration

```yaml
# .gitlab-ci.yml
stages:
  - build
  - test
  - deploy

variables:
  DOCKER_REGISTRY: registry.kejaksaan.go.id
  IMAGE_NAME: simpelv2/portal

build:
  stage: build
  image: rust:1.75
  before_script:
    - cargo install trunk
    - rustup target add wasm32-unknown-unknown
  script:
    - cd antarmuka/portal
    - trunk build --release
  artifacts:
    paths:
      - antarmuka/portal/dist/
    expire_in: 1 day

test:
  stage: test
  image: rust:1.75
  script:
    - cargo test --all
    - cargo clippy --all-targets
    - cargo fmt --check

deploy_staging:
  stage: deploy
  only:
    - develop
  script:
    - docker build -t $DOCKER_REGISTRY/$IMAGE_NAME:staging .
    - docker push $DOCKER_REGISTRY/$IMAGE_NAME:staging
    - kubectl set image deployment/portal portal=$DOCKER_REGISTRY/$IMAGE_NAME:stagin2-staging

deploy_production:
  stage: deploy
  only:
    - main
  when: manual
  script:
    - docker build -t $DOCKER_REGISTRY/$IMAGE_NAME:$CI_COMMIT_TAG .
    - docker push $DOCKER_REGISTRY/$IMAGE_NAME:$CI_COMMIT_TAG
    - docker tag $DOCKER_REGISTRY/$IMAGE_NAME:$CI_COMMIT_TAG $DOCKER_REGISTRY/$IMAGE_NAME:latest
    - docker push $DOCKER_REGISTRY/$IMAGE_NAME:latest
    - kubectl set image deployment/portal portal=$DOCKER_REGISTRY/$IMAGE_NAME:$CI_COMMIT_TAG -n simpelv2
```

---

## Backup and Recovery

### 1. Backup Static Files

```bash
#!/bin/bash
# backup-frontend.sh

BACKUP_DIR="/backup/frontend/$(date +%Y%m%d_%H%M%S)"
mkdir -p "$BACKUP_DIR"

# Backup portal
tar -czf "$BACKUP_DIR/portal.tar.gz" /var/www/portal

# Backup microfrontends
for module in badiklat datun intel pidum pidsus pidmil pengawasan pemulihan_aset; do
    tar -czf "$BACKUP_DIR/$module.tar.gz" "/var/www/$module"
done

# Backup nginx config
tar -czf "$BACKUP_DIR/nginx.tar.gz" /etc/nginx

echo "Backup completed: $BACKUP_DIR"
```

### 2. Restore from Backup

```bash
#!/bin/bash
# restore-frontend.sh

BACKUP_DIR=$1

if [ -z "$BACKUP_DIR" ]; then
    echo "Usage: $0 <backup_directory>"
    exit 1
fi

# Restore portal
tar -xzf "$BACKUP_DIR/portal.tar.gz" -C /

# Restore microfrontends
for module in badiklat datun intel pidum pidsus pidmil pengawasan pemulihan_aset; do
    tar -xzf "$BACKUP_DIR/$module.tar.gz" -C /
done

# Restore nginx config
tar -xzf "$BACKUP_DIR/nginx.tar.gz" -C /

# Reload nginx
systemctl reload nginx

echo "Restore completed from: $BACKUP_DIR"
```

---

## Security Checklist

Before deploying to production:

- [ ] SSL/TLS certificates configured
- [ ] Security headers enabled (CSP, HSTS, X-Frame-Options)
- [ ] CORS properly configured
- [ ] Rate limiting enabled
- [ ] DDoS protection active
- [ ] Secrets stored in Kubernetes secrets (not in code)
- [ ] Container images scanned for vulnerabilities
- [ ] Network policies configured
- [ ] RBAC properly set up
- [ ] Audit logging enabled
- [ ] Backup and recovery tested
- [ ] Incident response plan documented

---

## Performance Checklist

- [ ] WASM bundle optimized (< 500KB)
- [ ] Gzip compression enabled
- [ ] Static assets cached (1 year)
- [ ] Images optimized (WebP format)
- [ ] Lazy loading implemented
- [ ] Code splitting enabled
- [ ] CDN configured
- [ ] HTTP/2 enabled
- [ ] Resource hints added (preload, prefetch)
- [ ] Lighthouse score > 90

---

## Additional Resources

- **Kubernetes Documentation**: https://kubernetes.io/docs/
- **Nginx Documentation**: https://nginx.org/en/docs/
- **Trunk Documentation**: https://trunkrs.dev/
- **Leptos Documentation**: https://leptos.dev/
- **Internal Wiki**: https://wiki.kejaksaan.go.id/simpelv2/deployment

---

## Support

Untuk bantuan deployment:

- **DevOps Team**: devops@kejaksaan.go.id
- **On-Call**: +62-xxx-xxxx-xxxx
- **Slack**: #simpelv2-deployment

---

**Built with ❤️ by Tim Pengembang SIMPelv2**
**Kejaksaan Agung Republik Indonesia**

