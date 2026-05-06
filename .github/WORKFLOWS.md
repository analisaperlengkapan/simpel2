# GitHub Actions Workflows — SIMPEL

Overview of CI/CD workflows, their triggers, and purposes.

## Workflow Overview

| Workflow | File | Trigger | Purpose |
|----------|------|---------|---------|
| **CI - Unified Pipeline** | `ci-pipeline.yml` | Push/PR to main,develop,stag | Unified pipeline for Linting, Testing, and Building Rust/WASM (parallel) |
| **CI - Quality** | `ci-quality.yml` | Push/PR + weekly schedule | Documentation build, code metrics |
| **CI - Security** | `ci-security.yml` | Push/PR + weekly schedule | Dependency audit, SAST, secrets scan (cached tools) |
| **Integration Tests** | `integration-tests.yml` | Push/PR + daily schedule | Expanded matrix backend integration & workspace tests |
| **Release** | `release.yml` | Tag push (`v*.*.*`) or manual | Build, package, publish release + Docker (with caching & SBOM) |
| **Benchmark** | `benchmark.yml` | Push + weekly schedule | Track WASM bundle sizes and system performance |
| **Autofix PR** | `autofix-pr.yml` | Event trigger from PRs | Automated PR-based fixes |
| **Health Check** | `health-check.yml` | Daily schedule | Workflow validation & dependency audit |
| **Sync to GitLab** | `sync-to-gitlab.yml` | Push to main | Mirror changes to GitLab |
| **Sync from GitLab** | `sync-from-gitlab.yml` | Dispatch/schedule | Pull changes from GitLab |

## Path Filters

The `ci-pipeline.yml` unified workflow uses `dorny/paths-filter` to detect changes and conditionally run only the necessary jobs:

- **backend**: `layanan/**`, `lib/**`, `Cargo.toml`, `Cargo.lock`, `.github/actions/**`, `.github/workflows/ci-pipeline.yml`
- **frontend**: `antarmuka/**`, `lib/**`, `Cargo.toml`, `Cargo.lock`, `.github/actions/**`, `.github/workflows/ci-pipeline.yml`

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
