# GitHub Actions Workflows - Best Practices & Guidelines

This document describes the CI/CD workflows for SIMPEL, their purposes, and best practices for maintaining them.

## 📋 Table of Contents

- [Workflow Overview](#workflow-overview)
- [Core Workflows](#core-workflows)
- [Best Practices](#best-practices)
- [Troubleshooting](#troubleshooting)
- [Adding New Workflows](#adding-new-workflows)

## Workflow Overview

Our CI/CD pipeline consists of several specialized workflows:

| Workflow | Trigger | Purpose | Duration |
|----------|---------|---------|----------|
| **CI - Rust Core** | Push, PR | Lint, format, test Rust code | ~5-10 min |
| **CI - WASM Microfrontends** | Push, PR | Build and validate WASM modules | ~15-20 min |
| **CI - Quality & Metrics** | Push, PR, Schedule | Code coverage, docs, metrics | ~10-15 min |
| **CI - Security Checks** | Push, PR, Schedule | Security audit, SAST, secrets scan | ~5-10 min |
| **Integration Tests** | Push, PR, Schedule | End-to-end integration testing | ~20-30 min |
| **Release & Deployment** | Tag | Build, package, and deploy releases | ~30-45 min |
| **Auto-Fix** | Schedule | Automated code formatting and fixes | ~10 min |
| **Health Check** | Schedule | Monitor CI/CD health | ~2-5 min |

## Core Workflows

### 1. CI - Rust Core

**File**: [`.github/workflows/rust-ci.yml`](.github/workflows/rust-ci.yml)

**Purpose**: Primary CI workflow for Rust backend services.

**Jobs**:
- **Check & Lint**: Format checking, `cargo check`, `cargo clippy`
- **Test Suite**: Run all workspace tests
- **Build**: Release builds (main branch only)

**Best Practices**:
- Uses standardized `setup-rust` action for consistency
- Implements proper caching via `rust-cache`
- Fails fast on formatting issues
- Enforces `-D warnings` for clippy

**Optimization**:
```yaml
env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1
  RUSTFLAGS: "-D warnings"

concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true  # Cancel old runs on new push
```

### 2. CI - WASM Microfrontends

**File**: [`.github/workflows/wasm-ci.yml`](.github/workflows/wasm-ci.yml)

**Purpose**: Build and validate all WASM microfrontend modules.

**Matrix Strategy**:
- Builds 12 microfrontends in parallel
- Individual cache per module for efficiency
- Bundle size validation (max 50MB)

**Best Practices**:
- Use `fail-fast: false` to see all failures
- Upload artifacts for debugging
- Validate bundle sizes to catch bloat

### 3. CI - Security Checks

**File**: [`.github/workflows/ci-security.yml`](.github/workflows/ci-security.yml)

**Purpose**: Comprehensive security scanning.

**Security Layers**:
1. **Dependency Audit**: `cargo audit` for known vulnerabilities
2. **Policy Enforcement**: `cargo deny` for licenses and bans
3. **SAST**: Semgrep for code patterns
4. **Secrets Scanning**: Gitleaks for exposed credentials

**Schedule**: Runs on every push/PR + weekly scheduled scan

### 4. Integration Tests

**File**: [`.github/workflows/integration-tests.yml`](.github/workflows/integration-tests.yml)

**Purpose**: End-to-end integration testing across all modules.

**Test Suites**:
- **Backend Integration**: Test each service with real database
- **Frontend-Backend E2E**: Full stack testing with API Gateway
- **gRPC Integration**: Inter-service communication tests
- **Cross-Module**: Workspace-wide integration

**Infrastructure**:
- Spins up PostgreSQL and Redis in Docker
- Health checks before running tests
- Automatic cleanup after tests

**Best Practices**:
```yaml
steps:
  - name: Setup Test Infrastructure
    run: |
      docker run -d --name postgres-integration \
        -e POSTGRES_DB=simpelv2_integration \
        -p 5432:5432 postgres:16-alpine

  - name: Wait for services
    run: |
      for i in {1..30}; do
        if docker exec postgres-integration pg_isready; then
          break
        fi
        sleep 2
      done

  - name: Cleanup
    if: always()  # Always cleanup, even on failure
    run: docker stop postgres-integration
```

### 5. Release & Deployment

**File**: [`.github/workflows/release.yml`](.github/workflows/release.yml)

**Purpose**: Automated release process.

**Release Pipeline**:
1. **Create Release**: Generate GitHub release with changelog
2. **Build Backend**: Multi-platform binaries (x86_64, arm64, musl)
3. **Build Frontend**: All WASM microfrontends
4. **Build Docker**: Multi-arch Docker images
5. **Deploy Staging**: Deploy to staging environment
6. **Deploy Production**: Deploy to production (manual approval)

**Versioning**:
- Triggered by tags: `v*.*.*` (e.g., `v1.0.0`)
- Supports prerelease: `alpha`, `beta`, `rc`
- Automatic changelog extraction from `CHANGELOG.md`

## Best Practices

### 1. Standardized Setup

Always use the `setup-rust` action for consistency:

```yaml
- name: Setup Rust
  uses: ./.github/actions/setup-rust
  with:
    components: clippy, rustfmt
    targets: wasm32-unknown-unknown  # Optional
    cache-key: my-workflow           # Unique per workflow
    github-token: ${{ secrets.GITHUB_TOKEN }}
```

**Benefits**:
- ✅ Consistent Rust version across all workflows
- ✅ Automatic protoc installation
- ✅ Optimized caching strategy
- ✅ Permission fixes built-in

### 2. Caching Strategy

Use unique cache keys per workflow:

```yaml
# Good - Specific cache key
cache-key: integration-backend-tests

# Bad - Generic cache key
cache-key: default
```

**Cache Hierarchy**:
1. Workflow-specific: `integration-backend-tests`
2. Job-specific: `wasm-portal`
3. Matrix-specific: `release-x86_64-musl`

### 3. Concurrency Control

Prevent resource waste with concurrency:

```yaml
concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true  # Cancel old runs
```

**When to use**:
- ✅ Development workflows (CI, tests)
- ❌ Release workflows (don't cancel releases)
- ❌ Deployment workflows (don't cancel deploys)

### 4. Fail-Fast Strategy

Use `fail-fast: false` for matrix builds:

```yaml
strategy:
  fail-fast: false  # See all failures, not just first
  matrix:
    module: [portal, badiklat, datun, ...]
```

**Benefits**:
- See all failing modules in one run
- Better for debugging widespread issues
- Doesn't waste matrix resources

### 5. Environment Variables

Set consistent environment variables:

```yaml
env:
  CARGO_TERM_COLOR: always      # Colored output
  RUST_BACKTRACE: 1             # Stack traces
  RUSTFLAGS: "-D warnings"      # Treat warnings as errors
  CARGO_INCREMENTAL: 0          # Disable incremental (CI optimization)
```

### 6. Permission Fixes

Always include permission fixes for self-hosted runners:

```yaml
- name: Fix permissions
  run: |
    # Robust cleanup via Docker
    docker run --rm -v ${{ github.workspace }}:/workspace \
      -w /workspace alpine chown -R $(id -u):$(id -g) . || true

    # Fix corrupted cargo registry
    rm -rf ~/.cargo/registry/src/index.crates.io-*/tokio-postgres-0.7.16 || true
```

### 7. Error Handling

Use proper error handling:

```yaml
# Continue on error (collect all failures)
continue-on-error: true

# Conditional steps
if: always()  # Run even if previous steps failed
if: success()  # Only run if previous steps succeeded
if: failure()  # Only run if previous steps failed
```

### 8. Artifacts

Upload artifacts for debugging:

```yaml
- name: Upload Artifact
  if: always()  # Upload even on failure
  uses: actions/upload-artifact@v5
  with:
    name: wasm-${{ matrix.module }}
    path: antarmuka/${{ matrix.module }}/dist/
    retention-days: 7  # Auto-cleanup after 7 days
```

### 9. Step Summaries

Use step summaries for visibility:

```yaml
- name: Summary
  run: |
    echo "## 🎉 Build Summary" >> $GITHUB_STEP_SUMMARY
    echo "" >> $GITHUB_STEP_SUMMARY
    echo "| Metric | Value |" >> $GITHUB_STEP_SUMMARY
    echo "|--------|-------|" >> $GITHUB_STEP_SUMMARY
    echo "| Status | ${{ job.status }} |" >> $GITHUB_STEP_SUMMARY
```

## Troubleshooting

### Common Issues

#### 1. Protoc Installation Failure

**Error**: `mv: cannot overwrite directory: Directory not empty`

**Solution**: Fixed in `setup-rust` action v1.1.0+. Uses `cp -r` instead of `mv`.

#### 2. Permission Denied

**Error**: `Permission denied` when accessing workspace files

**Solution**: Add permission fix step at the beginning:

```yaml
- name: Fix permissions
  run: |
    docker run --rm -v ${{ github.workspace }}:/workspace \
      -w /workspace alpine chown -R $(id -u):$(id -g) . || true
```

#### 3. Cache Corruption

**Error**: Corrupted cargo registry cache

**Solution**: Clear corrupted cache:

```yaml
- name: Fix permissions
  run: |
    rm -rf ~/.cargo/registry/src/index.crates.io-*/tokio-postgres-0.7.16 || true
```

#### 4. Out of Disk Space

**Error**: `No space left on device`

**Solution**:
1. Check disk usage: `df -h`
2. Clean Docker: `docker system prune -af`
3. Clean cargo cache: `cargo cache --autoclean`
4. Clean old artifacts in GitHub

#### 5. Workflow Timeout

**Error**: Workflow times out after 6 hours

**Solution**:
- Optimize builds with better caching
- Reduce test parallelism if memory-constrained
- Split into multiple workflows

## Adding New Workflows

When adding a new workflow, follow this checklist:

### Checklist

- [ ] Add concurrency control
- [ ] Use `setup-rust` action
- [ ] Add permission fixes (self-hosted)
- [ ] Set environment variables
- [ ] Add proper caching strategy
- [ ] Use fail-fast: false for matrix
- [ ] Add step summaries
- [ ] Upload artifacts (if needed)
- [ ] Add cleanup steps
- [ ] Document in this file

### Template

```yaml
name: My New Workflow

on:
  push:
    branches: [main, develop]
  pull_request:
  workflow_dispatch:

concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  my-job:
    name: My Job
    runs-on: self-hosted
    steps:
      - name: Fix permissions
        run: |
          docker run --rm -v ${{ github.workspace }}:/workspace \
            -w /workspace alpine chown -R $(id -u):$(id -g) . || true

      - name: Checkout repository
        uses: actions/checkout@v6
        with:
          clean: true

      - name: Setup Rust
        uses: ./.github/actions/setup-rust
        with:
          components: clippy, rustfmt
          github-token: ${{ secrets.GITHUB_TOKEN }}
          cache-key: my-workflow

      - name: Run my task
        run: |
          echo "Do work here..."

      - name: Summary
        if: always()
        run: |
          echo "## Summary" >> $GITHUB_STEP_SUMMARY
          echo "Status: ${{ job.status }}" >> $GITHUB_STEP_SUMMARY
```

## Monitoring

### Health Check Workflow

The health check workflow runs every 6 hours to monitor:

- ✅ Workflow syntax validation
- ✅ Recent run success rates
- ✅ Best practices compliance
- ✅ Dependency health
- ✅ Infrastructure health

View health status in the [Actions tab](../../actions/workflows/health-check.yml).

### Metrics

Monitor these metrics:

1. **Success Rate**: Aim for >95% success rate
2. **Duration**: Track workflow duration trends
3. **Cache Hit Rate**: Should be >80%
4. **Artifact Size**: Monitor growth
5. **Disk Usage**: Keep <80% full

## Resources

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Rust GitHub Actions](https://github.com/dtolnay/rust-toolchain)
- [Actions Cache](https://github.com/actions/cache)
- [Setup Protoc](https://github.com/arduino/setup-protoc)

## Support

For issues with workflows:

1. Check the [troubleshooting section](#troubleshooting)
2. Review workflow logs in Actions tab
3. Check the health check workflow
4. Open an issue with workflow run link

---

**Last Updated**: 2026-01-29
**Maintained by**: SIMPEL Team
