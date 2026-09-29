#!/usr/bin/env bash
# Install a PINNED cargo-audit release into "$CARGO_HOME/bin", verifying its SHA-256.
#
# Shared by security.yml and maintenance.yml — both used to pipe `curl | tar`, i.e.
# executed whatever the release URL served, with no integrity check (cargo-deny in
# the same workflow was already checksum-pinned; this brings cargo-audit to the same
# bar). The digest below is the SHA-256 of the release asset as fetched when this
# pin was made (trust-on-first-use); bump VERSION and SHA256 together, and take the
# new digest from a download you have inspected.
#
# Failure semantics (deliberate):
#   * download failed (network)      -> fall back to `cargo install --locked` (slow, safe)
#   * download OK but digest differs -> HARD FAIL. That is not a network blip: the
#     asset changed under a tag we pinned. Falling back would hide exactly the
#     event this check exists to catch.
set -euo pipefail

VERSION="0.22.1"
SHA256="1890badd5f15831a9af4b074399fcd21e6f7c0fe42c84e9254cdffc9f813765c"
NAME="cargo-audit-x86_64-unknown-linux-gnu-v${VERSION}"
URL="https://github.com/rustsec/rustsec/releases/download/cargo-audit/v${VERSION}/${NAME}.tgz"

if command -v cargo-audit &>/dev/null; then
  exit 0
fi

: "${CARGO_HOME:?CARGO_HOME must be set (the setup-rust action sets it)}"
mkdir -p "$CARGO_HOME/bin"
archive="$(mktemp)"
trap 'rm -f "$archive"' EXIT

# --retry: a one-shot release download has thrown builds away in this repo before
# (#711 actionlint, the floating Blade plugin tag, the tailwindcss note in the FE
# Dockerfiles). --max-time bounds a hang.
if curl -LsSf --retry 4 --retry-delay 2 --retry-all-errors \
     --connect-timeout 15 --max-time 180 -o "$archive" "$URL"; then
  got="$(sha256sum "$archive" | awk '{print $1}')"
  if [ "$got" != "$SHA256" ]; then
    echo "::error::sha256 cargo-audit ${VERSION} tidak cocok: dapat ${got}, mau ${SHA256}" >&2
    exit 1
  fi
  # The tarball has a top-level `${NAME}/` directory; --strip-components=1 puts the
  # binary at "$CARGO_HOME/bin/cargo-audit" (on PATH).
  tar -xzf "$archive" --strip-components=1 -C "$CARGO_HOME/bin/" "${NAME}/cargo-audit"
  echo "Installed cargo-audit ${VERSION} (sha256 verified)"
  "$CARGO_HOME/bin/cargo-audit" --version
else
  echo "::warning::Pre-built binary unavailable, compiling from source"
  cargo install cargo-audit --locked
fi
