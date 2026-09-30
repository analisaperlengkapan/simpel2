#!/usr/bin/env bash
# Preload the SIMPEL2 development environment for a new OpenHands session.
#
# OpenHands runs this on every session start (`.openhands/setup.sh` — see
# docs.openhands.dev, "Repository Customization"). A fresh sandbox has Node and
# Docker but no Rust toolchain, no protoc, no `.env` and no Playwright browsers,
# so the first thing an agent does is spend ten minutes discovering that and
# installing it by hand. This does it once, up front.
#
# Contract:
#   - Idempotent. Every step checks first and says "already present" when it can.
#   - Non-fatal. A step that fails (offline mirror, rate limit, missing sudo)
#     warns and moves on; setup must never be the reason a session cannot start.
#   - Non-interactive, and quiet enough to read in a transcript.
#
# Opt-in extras (off by default — both are minutes, not seconds):
#   SIMPEL_SETUP_BUILD=1   build the compose images up front
#   SIMPEL_SETUP_STACK=1   build, then bring the e2e stack up and seed it
#
# Note on scope: this provisions the *toolchain and local test environment*.
# It deliberately does not install anything the repository already pins for CI
# (the versions below mirror `.github/actions/setup-rust` and `Dockerfile.e2e`),
# so a local run and a CI run agree.

set -uo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO" || exit 0

# Standard install locations, added up front and unconditionally. A
# non-interactive shell never reads `.bashrc`, so without this the presence
# checks below cannot see a toolchain installed by a previous run and every
# session reinstalls it.
export PATH="$HOME/.cargo/bin:$HOME/.local/bin:$PATH"

RUST_VERSION="1.97.1"   # mirrors .github/actions/setup-rust + Cargo.toml rust-version
PROTOC_VERSION="25.1"   # mirrors .github/actions/setup-rust
NODE_MAJOR="20"         # mirrors .github/actions/setup-node

step() { printf '\n\033[1m==> %s\033[0m\n' "$1"; }
note() { printf '    %s\n' "$1"; }
warn() { printf '    \033[33m! %s\033[0m\n' "$1"; }

# Persist PATH additions for later shells. The setup script runs in its own
# process, so an `export` here would not outlive it — and the terminal sessions
# that follow are where the toolchain actually gets used.
persist_path() {
  local dir="$1"
  case ":$PATH:" in *":$dir:"*) ;; *) export PATH="$dir:$PATH" ;; esac
  local line="export PATH=\"$dir:\$PATH\""
  if ! grep -qxF "$line" "$HOME/.bashrc" 2>/dev/null; then
    printf '\n# added by .openhands/setup.sh\n%s\n' "$line" >>"$HOME/.bashrc"
  fi
}

step "Rust toolchain ($RUST_VERSION)"
if command -v cargo >/dev/null 2>&1 &&
  cargo --version 2>/dev/null | grep -q "$RUST_VERSION"; then
  note "cargo $RUST_VERSION already present"
else
  if ! command -v rustup >/dev/null 2>&1; then
    note "installing rustup (minimal profile)"
    curl --proto '=https' --tlsv1.2 -fsSL https://sh.rustup.rs 2>/dev/null |
      sh -s -- -y --profile minimal --default-toolchain "$RUST_VERSION" \
        --component rustfmt,clippy >/dev/null 2>&1 ||
      warn "rustup install failed (offline?) — install Rust manually to build crates"
  fi
  if command -v rustup >/dev/null 2>&1; then
    rustup toolchain install "$RUST_VERSION" --profile minimal \
      --component rustfmt,clippy >/dev/null 2>&1
    # The wasm32 target is needed for the microfrontends and the WASM-safe
    # lib/ crates; without it `cargo check -p perlengkapan-microfrontend
    # --target wasm32-unknown-unknown` fails on a missing std.
    rustup target add wasm32-unknown-unknown >/dev/null 2>&1 ||
      warn "could not add wasm32-unknown-unknown target"
  fi
  if command -v cargo >/dev/null 2>&1; then
    persist_path "$HOME/.cargo/bin"
    note "cargo $(cargo --version 2>/dev/null | awk '{print $2}') ready"
  else
    warn "cargo still unavailable"
  fi
fi

step "protoc ($PROTOC_VERSION, for the gRPC build scripts)"
if command -v protoc >/dev/null 2>&1 &&
  [ "$(protoc --version 2>/dev/null | awk '{print $2}')" = "$PROTOC_VERSION" ]; then
  note "protoc $PROTOC_VERSION already present"
else
  _zip="protoc-${PROTOC_VERSION}-linux-x86_64.zip"
  _tmp="$(mktemp -d)"
  if curl -LsSf -o "$_tmp/$_zip" \
    "https://github.com/protocolbuffers/protobuf/releases/download/v${PROTOC_VERSION}/${_zip}" 2>/dev/null &&
    python3 -c "import zipfile,sys; zipfile.ZipFile(sys.argv[1]).extractall(sys.argv[2])" \
      "$_tmp/$_zip" "$_tmp/x" 2>/dev/null; then
    mkdir -p "$HOME/.local/bin" "$HOME/.local/include"
    cp -f "$_tmp/x/bin/protoc" "$HOME/.local/bin/" 2>/dev/null
    cp -r "$_tmp/x/include/." "$HOME/.local/include/" 2>/dev/null
    chmod +x "$HOME/.local/bin/protoc" 2>/dev/null
    persist_path "$HOME/.local/bin"
    note "protoc installed to ~/.local/bin"
  else
    warn "could not download protoc $PROTOC_VERSION — the tonic build scripts need it"
  fi
  rm -rf "$_tmp"
fi

step "Node"
if command -v node >/dev/null 2>&1; then
  _node_major="$(node --version 2>/dev/null | sed 's/^v//; s/\..*//')"
  if [ "${_node_major:-0}" -ge "$NODE_MAJOR" ]; then
    note "node $(node --version) (CI uses $NODE_MAJOR)"
  else
    warn "node $(node --version) is older than CI's $NODE_MAJOR — Playwright may misbehave"
  fi
else
  warn "node not found — the Playwright suites cannot run"
fi

step "Node dependencies for the Playwright suites"
for _suite in antarmuka/perlengkapan/tests/e2e antarmuka/portal/tests/e2e; do
  if [ ! -f "$_suite/package.json" ]; then
    warn "$_suite/package.json missing — skipping"
    continue
  fi
  if [ -d "$_suite/node_modules" ]; then
    note "$_suite: node_modules present"
  else
    note "$_suite: npm ci"
    (cd "$_suite" && npm ci --no-audit --no-fund >/dev/null 2>&1) ||
      warn "$_suite: npm ci failed"
  fi
done

step "Playwright browsers"
# The CI e2e images bake these in; a host run needs them. `--with-deps` wants
# root, which the sandbox may not grant — the browser itself is the part that
# matters, so fall back to a plain install.
if [ -d "$HOME/.cache/ms-playwright" ]; then
  note "browser cache present"
elif [ -d antarmuka/perlengkapan/tests/e2e/node_modules ]; then
  (cd antarmuka/perlengkapan/tests/e2e &&
    npx playwright install --with-deps chromium >/dev/null 2>&1 ||
    npx playwright install chromium >/dev/null 2>&1) ||
    warn "could not install Chromium — use the simpel-e2e-* images instead"
  [ -d "$HOME/.cache/ms-playwright" ] && note "Chromium installed"
else
  warn "skipped (suite dependencies not installed)"
fi

step "Local .env for docker compose"
# docker-compose.yml requires POSTGRES_PASSWORD and SIMPELV1_APP_KEY via
# `${VAR:?}` interpolation for every service in the merged files — including
# ones a given command never starts — so `docker compose` fails outright without
# them. The e2e harness fills these in at runtime, but anything else the agent
# runs by hand (docker compose config/ps/logs) needs them to exist here.
#
# Only created when absent: overwriting a developer's .env would make Compose
# initialise Postgres with a different password than the one the existing volume
# was created with, and every later command would fail authentication.
if [ -f .env ]; then
  note ".env present — leaving it alone"
elif [ -f .env.example ]; then
  note ".env.example found but no .env; creating a disposable local .env"
fi
if [ ! -f .env ]; then
  # umask scoped to the write: a credentials file should not be world-readable,
  # and leaving the umask changed would affect everything created afterwards.
  (
    umask 077
    cat >.env <<'EOF'
# Disposable LOCAL test credentials, created by .openhands/setup.sh.
# Not secrets: they only ever reach a throwaway compose stack. Production and
# staging get theirs from Secreton (see infra/AGENTS.md).
POSTGRES_USER=postgres
POSTGRES_PASSWORD=e2e_test_pw
SIMPELV1_APP_KEY=base64:dGVzdGtleTMyYnl0ZXNzc3Nzc3Nzc3Nzc3Nzc3Nz
EOF
  )
  note "wrote .env (POSTGRES_USER/POSTGRES_PASSWORD/SIMPELV1_APP_KEY)"
  note "ignored by git, so it will not be committed"
fi

if [ "${SIMPEL_SETUP_BUILD:-0}" = "1" ]; then
  step "Building compose images (SIMPEL_SETUP_BUILD=1)"
  ./infra/scripts/run-e2e.sh build || warn "image build failed"
fi

if [ "${SIMPEL_SETUP_STACK:-0}" = "1" ]; then
  step "Bringing the e2e stack up (SIMPEL_SETUP_STACK=1)"
  ./infra/scripts/run-e2e.sh up || warn "stack did not come up cleanly"
fi

printf '\n\033[1m==> Ready\033[0m\n'
note "repo:     $REPO"
note "rust:     $(command -v cargo >/dev/null 2>&1 && cargo --version || echo 'unavailable')"
note "protoc:   $(command -v protoc >/dev/null 2>&1 && protoc --version || echo 'unavailable')"
note "docker:   $(command -v docker >/dev/null 2>&1 && docker --version || echo 'unavailable')"
note ""
note "e2e stack:  ./infra/scripts/run-e2e.sh up      # build + start + seed"
note "            ./infra/scripts/run-e2e.sh smoke   # quick auth smoke"
note "            ./infra/scripts/run-e2e.sh perl    # full perlengkapan suite"
note "            ./infra/scripts/run-e2e.sh down"
note "spec guard: python3 infra/scripts/check-playwright-projects-cover-specs.py --self-test"
