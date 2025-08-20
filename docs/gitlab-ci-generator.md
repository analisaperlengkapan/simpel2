# 🔄 GitLab CI/CD Generator - SIMPelv2

Dokumentasi untuk GitLab CI/CD pipeline generator yang mendukung **CI induk di root** dan **CI per layanan**.

## 📋 Overview

Generator ini menghasilkan:
- **Main GitLab CI** (`.gitlab-ci.yml`) - Pipeline utama untuk seluruh proyek
- **Service-specific CI** (`.gitlab-ci.{service}.yml`) - Pipeline khusus per layanan
- **Multi-environment deployment** - Dev, staging, production
- **Security scanning integration** - SAST, DAST, Container Scanning
- **Vault integration** - Secret management

## 🚀 Quick Start

### Generate Main CI Only
```bash
# Generate main GitLab CI
python3 scripts/generate.py gitlab-ci-main

# Dengan konfigurasi custom
python3 scripts/generate.py gitlab-ci-main --ci-config ci_config.json

# Dry run (test tanpa menulis file)
python3 scripts/generate.py gitlab-ci-main --dry-run
```

### Generate Service CI Only
```bash
# Generate CI untuk semua service
python3 scripts/generate.py gitlab-ci-service

# Dengan konfigurasi custom
python3 scripts/generate.py gitlab-ci-service --ci-config ci_config.json

# Dry run
python3 scripts/generate.py gitlab-ci-service --dry-run
```

### Generate All CI
```bash
# Generate main + service CI
python3 scripts/generate.py gitlab-ci

# Generate everything (Compose + K8s + CI)
python3 scripts/generate.py all
```

## 🛠️ Makefile Targets

```bash
# Generate main CI
make generate-gitlab-ci-main

# Generate service CI
make generate-gitlab-ci-service

# Generate all CI (main + service)
make generate-gitlab-ci-all

# Generate everything
make pipeline-all
```

## 📁 File Structure

### Main CI File
```
.gitlab-ci.yml                    # Main pipeline
├── stages: lint, validate, build, seal, test, deploy, release, cleanup
├── security scanning integration
├── multi-environment deployment
└── vault integration
```

### Service CI Files
```
.gitlab-ci.portal.yml             # Frontend service
.gitlab-ci.gerbang.yml            # Backend service
.gitlab-ci.db-simpelv2.yml        # Database service
.gitlab-ci.nginx.yml              # Proxy service
└── ... (one per service)
```

## ⚙️ Configuration

### Default Configuration
```json
{
  "ci_file": ".gitlab-ci.yml",
  "base_image": "python:3.10",
  "base_dir": "/var/www/simpelv2",
  "compose_file": "docker-compose.secure.yml",
  "security_includes": [
    {"template": "Security/SAST.gitlab-ci.yml"},
    {"template": "Security/Dependency-Scanning.gitlab-ci.yml"},
    {"template": "Security/Container-Scanning.gitlab-ci.yml"},
    {"template": "Security/License-Management.gitlab-ci.yml"},
    {"template": "Security/Secret-Detection.gitlab-ci.yml"},
    {"template": "Security/DAST.gitlab-ci.yml"},
    {"template": "Security/SCA.gitlab-ci.yml"}
  ],
  "stages": [
    "lint", "validate", "build", "seal", "test", "deploy", "release", "cleanup"
  ],
  "deployment_config": {
    "dev": {"branch": "main", "manual": true, "rollback": true},
    "staging": {"branch": "/^release\\/.*$/", "manual": true, "rollback": true},
    "prod": {"branch": "tags", "manual": true, "rollback": true}
  }
}
```

### Custom Configuration
```json
{
  "ci_file": ".gitlab-ci.yml",
  "base_image": "python:3.11",
  "custom_variables": {
    "CUSTOM_VAR_1": "value1",
    "CUSTOM_VAR_2": "value2"
  },
  "additional_stages": [
    "security-scan",
    "performance-test"
  ],
  "custom_jobs": {
    "security:custom": {
      "stage": "security-scan",
      "script": [
        "echo \"🔍 Custom security scan...\"",
        "make security-scan"
      ]
    }
  }
}
```

## 🔧 Service Types

Generator mendeteksi tipe service secara otomatis:

### Frontend Services
- **Image**: `node:18-alpine`
- **Before Script**: `apk add --no-cache git make && npm install -g npm@latest`
- **Examples**: `portal`, `perlengkapan`, `keuangan`

### Backend Services
- **Image**: `golang:1.21-alpine`
- **Before Script**: `apk add --no-cache git make && go version`
- **Examples**: `gerbang`, `layanan-keamanan`, `layanan-audit`

### Generic Services
- **Image**: `python:3.10`
- **Before Script**: `apt-get update && apt-get install -y curl git make && pip install python-dotenv pyyaml`
- **Examples**: `nginx`, `db-simpelv2`

## 📊 Main CI Jobs

### Lint Stage
```yaml
lint:yaml:
  stage: lint
  script:
    - "echo \"🔍 Melakukan lint YAML...\""
    - make lint-yaml
```

### Validate Stage
```yaml
validate:all:
  stage: validate
  script:
    - "echo \"✅ Validasi semua konfigurasi...\""
    - make validate-env
    - make validate-all
```

### Build Stage
```yaml
generate:k8s:
  stage: build
  script:
    - "echo \"🔧 Generate YAML K8s dari docker-compose...\""
    - make generate-k8s
```

### Seal Stage
```yaml
seal:secrets:
  stage: seal
  script:
    - "echo \"🔐 Install kubeseal...\""
    - curl -sL https://github.com/bitnami-labs/sealed-secrets/releases/latest/download/kubeseal-linux-amd64 -o kubeseal
    - chmod +x kubeseal && mv kubeseal /usr/local/bin/kubeseal
    - "echo \"🔐 Menyegel semua secret layanan...\""
    - make seal-secret
    - "echo \"🔍 Validasi hasil SealedSecrets...\""
    - make validate-sealed
```

### Test Stage
```yaml
test:simulate:
  stage: test
  script:
    - "echo \"🧪 Simulasi pod run semua image...\""
    - make simulate-pod-run

coverage:go:
  stage: test
  script:
    - cd backend || exit 0
    - go test -coverprofile=coverage.out ./...
    - go tool cover -func=coverage.out
  coverage: "/total:\\s+\\(statements\\)\\s+(\\d+\\.\\d+%)/"
  allow_failure: true
```

### Deploy Stage
```yaml
deploy:dev:
  stage: deploy
  script:
    - "echo \"🚀 Deploy ke dev...\""
    - make deploy-dev
  environment:
    name: dev
    url: https://dev.simpelv2.local
  when: manual
  only: [main]
```

## 🔧 Service CI Jobs

### Build Job
```yaml
build:portal:
  stage: build
  script:
    - "echo \"🔨 Building portal...\""
    - docker build -t portal:$CI_COMMIT_SHA .
    - docker tag portal:$CI_COMMIT_SHA portal:latest
  artifacts:
    paths: [dist/portal/]
    expire_in: 1 week
```

### Test Job
```yaml
test:portal:
  stage: test
  script:
    - "echo \"🧪 Testing portal...\""
    - make test-portal
  allow_failure: true
```

### Deploy Job
```yaml
deploy:portal:
  stage: deploy
  script:
    - "echo \"🚀 Deploying portal...\""
    - make deploy-portal
  environment:
    name: portal-dev
    url: https://portal.dev.simpelv2.local
  when: manual
```

## 🔐 Security Integration

### Security Templates
```yaml
include:
  - template: Security/SAST.gitlab-ci.yml
  - template: Security/Dependency-Scanning.gitlab-ci.yml
  - template: Security/Container-Scanning.gitlab-ci.yml
  - template: Security/License-Management.gitlab-ci.yml
  - template: Security/Secret-Detection.gitlab-ci.yml
  - template: Security/DAST.gitlab-ci.yml
  - template: Security/SCA.gitlab-ci.yml
```

### Custom Security Jobs
```yaml
security:custom:
  stage: security-scan
  script:
    - "echo \"🔍 Custom security scan...\""
    - make security-scan
```

## 🔄 Vault Integration

### Vault Setup
```yaml
vault:setup:
  stage: setup
  script:
    - "echo \"🔐 Setting up Vault...\""
    - make vault-setup
  when: manual
```

### Vault Decrypt
```yaml
vault:decrypt:
  stage: seal
  script:
    - "echo \"🔓 Decrypting secrets...\""
    - make vault-decrypt
  when: manual
```

## 📈 Monitoring & Metrics

### Health Checks
```yaml
monitor:health:
  stage: test
  script:
    - "echo \"🏥 Health check...\""
    - make health-check
  allow_failure: true
```

### Metrics Collection
```yaml
monitor:metrics:
  stage: test
  script:
    - "echo \"📊 Collecting metrics...\""
    - make collect-metrics
  artifacts:
    paths: [metrics/]
    expire_in: 1 week
```

## 🚀 Advanced Usage

### Custom Configuration File
```bash
# Buat file konfigurasi custom
cat > my-ci-config.json << EOF
{
  "base_image": "python:3.11",
  "custom_variables": {
    "MY_VAR": "my_value"
  },
  "additional_stages": ["custom-stage"],
  "custom_jobs": {
    "custom:job": {
      "stage": "custom-stage",
      "script": ["echo 'Custom job'"]
    }
  }
}
EOF

# Gunakan konfigurasi custom
python3 scripts/generate.py gitlab-ci-main --ci-config my-ci-config.json
```

### Force Overwrite
```bash
# Force overwrite tanpa konfirmasi
python3 scripts/generate.py gitlab-ci-main --force
```

### Dry Run
```bash
# Test tanpa menulis file
python3 scripts/generate.py gitlab-ci-main --dry-run
```

## 🔧 Integration dengan Existing Pipeline

### Update Existing CI
```bash
# Backup existing CI
cp .gitlab-ci.yml .gitlab-ci.yml.backup

# Generate new CI
python3 scripts/generate.py gitlab-ci-main --force

# Compare changes
diff .gitlab-ci.yml.backup .gitlab-ci.yml
```

### Service-specific Deployment
```bash
# Deploy specific service
make deploy-portal

# Test specific service
make test-portal

# Build specific service
make build-portal
```

## 📚 Examples

### Complete Pipeline
```bash
# Generate everything
make pipeline-all

# Deploy to dev
make deploy-dev

# Deploy to staging
make deploy-staging

# Deploy to production
make deploy-prod
```

### Service-specific Workflow
```bash
# Generate CI for portal service
python3 scripts/generate.py gitlab-ci-service

# Build and test portal
make build-portal
make test-portal

# Deploy portal
make deploy-portal
```

## 🎯 Best Practices

1. **Use Configuration Files**: Simpan konfigurasi di `ci_config.json`
2. **Test with Dry Run**: Selalu test dengan `--dry-run` sebelum deploy
3. **Backup Existing Files**: Generator otomatis backup file yang ada
4. **Service-specific CI**: Gunakan service CI untuk development cepat
5. **Main CI for Production**: Gunakan main CI untuk deployment production

## 🔍 Troubleshooting

### Common Issues

1. **Permission Denied**
   ```bash
   # Fix permissions
   chmod +x scripts/generate.py
   ```

2. **Missing Dependencies**
   ```bash
   # Install dependencies
   pip install pyyaml python-dotenv
   ```

3. **Invalid Configuration**
   ```bash
   # Validate configuration
   python3 scripts/generate.py gitlab-ci-main --dry-run
   ```

### Debug Mode
```bash
# Enable debug logging
export PYTHONPATH=/var/www/simpelv2/scripts
python3 -u scripts/generate.py gitlab-ci-main --dry-run
```

## 📞 Support

Untuk bantuan lebih lanjut:
- Cek log dengan `--dry-run`
- Validasi konfigurasi dengan `python -m json.tool ci_config.json`
- Test dengan service sederhana terlebih dahulu 