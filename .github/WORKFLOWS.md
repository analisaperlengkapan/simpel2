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

- **Cargo** dependencies (monthly, 02:00 WIB, at most 3 open PRs)
- **GitHub Actions** versions (monthly, 02:30 WIB)
- **Docker** base images (monthly, 03:00 WIB)
- **npm** (`lib/ui`, monthly, 03:30 WIB)

Updates are grouped (related crates together, every other minor/patch crate in one
`rust-minor-patch` PR) because each bot PR runs the full CI on the same runner pool
as everyone else's work. Security updates ignore the schedule. The crypto crates
stay out of the catch-all group so a minor bump there is always reviewed by a person.

## Image builds and the BuildKit cache

- **Pull requests** build the six images once, in `e2e-build-images`
  (`infra/scripts/ci-build-e2e-images.sh`), because the e2e jobs need them. The
  separate `Build Docker` matrix does **not** run on PRs; it runs on pushes to
  `main`/`staging` (when code, a manifest or a Dockerfile changed) and on
  `workflow_dispatch`, and `release.yml` builds them again before anything is promoted.
- **Cache:** BuildKit's registry cache in the in-cluster registry
  (`172.15.10.252:32000`, `cache/<image>:main`; builders trust it through
  `.github/buildkitd.toml`). Every build reads it; a miss only means a cold build.
  Only a push to `main` that changed a Cargo manifest, a Dockerfile or a workflow
  writes it (or `workflow_dispatch` with `run_e2e` and `refresh_cache` on `main`).
- **Why not write on every push:** `mode=max` also exports the `cargo build` layer,
  which differs on every commit, and the registry has no garbage collector. The
  blobs a rewritten cache ref leaves behind are reclaimed only by running
  `registry garbage-collect` in the registry pod (an operator step, not automated).
