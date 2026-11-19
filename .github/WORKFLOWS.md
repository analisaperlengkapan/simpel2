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

## CI scoping: only what the change can affect

A pull request (and a push to `main`) runs the Rust units and e2e suites its diff can
affect, and nothing else. The decision is made once, in the `detect` job, by
`infra/scripts/ci-scope.py`:

- **The crate graph is derived**, from the workspace's own `Cargo.toml` files (normal,
  dev, build and target-specific dependencies). A change to a crate affects it and
  everything that depends on it, transitively: `lib/core` reaches nearly everything;
  `antarmuka/portal` reaches only the portal; `layanan/integrasi` reaches itself and
  `layanan/perlengkapan`, because that crate imports it.
- **What a `build.rs` reads from another crate's directory** is part of the graph too,
  although Cargo does not know it: `layanan/perlengkapan/build.rs` compiles
  `layanan/authenc/proto/*.proto` into its gRPC clients without depending on an authenc
  crate. The script is scanned for string literals that climb out of the crate and name
  a path that exists; a change under such a path runs the crate that reads it.
- **`.github/ci-scope.json`** says which crates form a *unit* (the granularity of the
  clippy/test/build jobs), holds those matrices as data, and classifies paths that are
  not inside a crate: `everything` (lockfile, toolchain, `ci.yml`), `e2e_only`
  (compose files, specs, fixtures, nginx), `no_effect` (docs, Helm values, other
  workflows, guard scripts).
- **e2e suites** run when they consume an image whose unit is affected. What a suite
  consumes is read from its own `for img in simpel2-...; do docker pull` loop in
  `ci.yml`. `e2e-build-images` builds and pushes only the union of those images.
- **A doubt is always resolved by running more.** A path that matches no rule runs
  everything, and so does a diff that could not be computed or that is long enough to
  have been truncated. Only the `no_effect` globs may skip work, and they are matched
  before ownership.
- **Safety nets:** the nightly `schedule` run and `workflow_dispatch` are always full.
  Run a full CI (`gh workflow run ci.yml --ref main -f run_e2e=true`) on the commit you
  are about to tag as a release candidate.
- **Guards:** `check-ci-scope.py` fails if a per-unit job or an e2e suite is not wired
  to the decision (it would run on every PR again) or if `ci-scope.json` stops matching
  the workspace; `test_ci_scope.py` proves the derivation. A new crate must be added to
  a unit; a new `test-<unit>` job needs the `contains(fromJson(needs.detect.outputs.units), ...)`
  condition; both are enforced. The guard also fails if a change to `ci-scope.json`, the
  scope scripts or `ci.yml` would not run the `enforce-versioning` job that hosts these
  guards (the gate is read from that job's own `if:`).
- **Reading a run:** the `detect` job's summary lists every unit and suite with why it
  ran or was skipped.

## Rust-only security and autofix jobs

`cargo-audit`, `cargo-deny`, Verus (`security.yml`) and the Rust autofixer (`autofix.yml`)
cost about 7, 7, 7 and 27 runner-minutes per pull-request push (measured over five runs;
the autofixer peaked at 44) and can only react to Rust sources, Cargo manifests, their own
config and the workflow that runs them. A small `changes` job in each workflow decides
from the paths the pull request touches (`dorny/paths-filter`), and the jobs are skipped
when it finds none.

- **Always full:** push to `main`, the daily schedule and `workflow_dispatch`, and any pull
  request whose file list may have been truncated (2,900 files). A newly published advisory
  against an unchanged lockfile therefore surfaces in the scheduled run, no longer on an
  unrelated pull request; that run is the one to watch.
- **Gated per job, not per workflow:** a workflow-level `paths:` filter would stop the
  workflow from reporting, and a required check that never reports blocks the pull request.
- **`changes` is blocking in the Security Summary:** if it fails, the three jobs are skipped
  and the summary would otherwise be green with nothing checked.
- **Guard:** `infra/scripts/check-rust-job-gating.py` derives what the jobs read (workspace
  manifests, files that contain `verus!`, crate sources) and fails if a filter does not cover
  it or a gate stops failing closed; it has a mutation self-test.

## Image builds and the BuildKit cache

- **Pull requests** build the six images once, in `e2e-build-images`
  (`infra/scripts/ci-build-e2e-images.sh`), because the e2e jobs need them. The
  separate `Build Docker` matrix does **not** run on PRs; it runs on every push to
  `main`/`staging` and on `workflow_dispatch` (every push, not only code changes: the
  portal and perlengkapan images `COPY . .` and run `infra/scripts/*` inside the build),
  and `release.yml` builds them again before anything is promoted.
- **What reaches an image build on a PR:** `e2e-build-images` runs when the
  `rust-frontend`, `rust-backend` or `e2e` filter fires. Files an image reads from outside
  its own directory (root `.dockerignore`, the `infra/scripts/*` the Dockerfiles run) are in
  the `e2e` filter, and `infra/scripts/check-image-build-scope.py` derives that list from
  the Dockerfiles and fails when one is missing.
- **Cache:** BuildKit's registry cache in the in-cluster registry
  (`172.15.10.252:32000`, `cache/<image>:main`; builders trust it through
  `.github/buildkitd.toml`). `ci.yml`'s builds read it; a miss only means a cold build.
  **`release.yml` never uses it**: the registry is plain HTTP without authentication, so
  anyone on the runner network could replace a cached layer and have it end up in an image
  that gets promoted. Release images are built from source only; the same guard fails if
  `release.yml` ever imports the cache.
  Only a push to `main` that changed a Cargo manifest, a Dockerfile or a workflow
  writes it (or `workflow_dispatch` with `run_e2e` and `refresh_cache` on `main`).
- **Why not write on every push:** `mode=max` also exports the `cargo build` layer,
  which differs on every commit, and the registry has no garbage collector. The
  blobs a rewritten cache ref leaves behind are reclaimed only by running
  `registry garbage-collect` in the registry pod (an operator step, not automated).
