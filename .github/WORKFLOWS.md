# GitHub Actions Workflows — SIMPEL

Overview of CI/CD workflows, their triggers, and purposes.

## Workflow Overview

| Workflow | File | Trigger | Purpose |
|----------|------|---------|---------|
| **CI - Rust Core** | `rust-ci.yml` | Push/PR to main,develop (backend paths) | Lint, format, test, build Rust workspace |
| **CI - WASM** | `wasm-ci.yml` | Push/PR to main,develop (frontend paths) | Lint & build WASM microfrontends |
| **CI - Quality** | `ci-quality.yml` | Push/PR + weekly schedule | Documentation build, code metrics |
| **CI - Security** | `ci-security.yml` | Push/PR + weekly schedule | Dependency audit, SAST, secrets scan |
| **Integration Tests** | `integration-tests.yml` | Push/PR + daily schedule | Backend integration & workspace tests |
| **Release** | `release.yml` | Tag push (`v*.*.*`) or manual | Build, package, publish release + Docker |
| **Auto-Fix** | `auto-fix.yml` | Weekly schedule (Monday) | Automated cargo fix, clippy, fmt |
| **Health Check** | `health-check.yml` | Daily schedule | Workflow validation & dependency audit |
| **Sync to GitLab** | `sync-to-gitlab.yml` | Push to main | Mirror changes to GitLab |
| **Sync from GitLab** | `sync-from-gitlab.yml` | Dispatch/schedule | Pull changes from GitLab |

## Path Filters

Workflows use path filters to avoid unnecessary runs:

- **rust-ci.yml**: `layanan/**`, `lib/**`, `Cargo.toml`, `Cargo.lock`
- **wasm-ci.yml**: `antarmuka/**`, `lib/**`, `Cargo.toml`, `Cargo.lock`

## Concurrency

All workflows use concurrency groups to cancel redundant runs:

```yaml
concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true
```

## Shared Setup

All Rust jobs use the composite action `.github/actions/setup-rust` which handles:
- Workspace permission fixes (self-hosted runner cleanup)
- System dependency installation (clang, libssl, protobuf, etc.)
- Rust toolchain installation via `dtolnay/rust-toolchain`
- Protocol Buffers compiler (protoc 25.1)
- Rust build cache via `Swatinem/rust-cache@v2`

## Dependabot

Configuration in `.github/dependabot.yml` manages:
- **Cargo** dependencies (weekly, Monday 02:00 WIB)
- **GitHub Actions** versions (weekly, Tuesday 02:00 WIB)
- **Docker** base images (weekly, Wednesday 02:00 WIB)

Dependency groups reduce PR noise by batching related updates.
