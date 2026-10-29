#!/usr/bin/env bash
# Pre-seed every tool trunk downloads into trunk's own tool cache.
#
# Why: trunk fetches these binaries from GitHub releases with a single-shot
# reqwest call — no retry, ~30 s budget — and it does so in the asset pipeline,
# i.e. AFTER the 10-35 minute wasm compile. One dropped connection therefore
# discards the whole build. Incidents: CI run 30778635010 (tailwindcss), and
# 2026-10-27 (wasm-opt on #1002/#1003, tailwindcss on #1003). curl --retry with
# backoff turns each of those into a few seconds of retry.
#
# trunk only checks that the cached file exists and is executable — no checksum
# (trunk/src/tools.rs:338-348) — so seeding the file removes the network
# dependency for that tool entirely.
#
# Cache layout (trunk/src/tools.rs): "{cache}/{name}-{version}/{path}", where
# {cache} is the `dev.trunkrs.trunk` project cache dir and {path} is
# "bin/wasm-opt" for binaryen, "{name}" for the others.
#
# Versions are read from the same places trunk reads them, so a pre-seeded path
# cannot drift from the one trunk looks up:
#   tailwindcss, wasm-opt -> Trunk.toml [tools]
#   wasm-bindgen          -> Cargo.lock
# A [tools] key this script cannot fetch is a hard error: an unhandled pin would
# otherwise look pre-seeded while trunk still downloads it at build time.
#
# Usage: preseed-trunk-tools.sh <frontend-dir> [repo-root]
set -euo pipefail

fe_dir=${1:?usage: preseed-trunk-tools.sh <frontend-dir> [repo-root]}
trunk_toml="$fe_dir/Trunk.toml"
[ -f "$trunk_toml" ] || { echo "::error::$trunk_toml not found"; exit 1; }

# The FE crates are members of the root workspace, so the Cargo.lock trunk reads
# is the one at the repository root.
if [ $# -ge 2 ]; then
  repo_root=$2
else
  repo_root=""
  d=$(cd "$fe_dir" && pwd)
  while [ "$d" != "/" ]; do
    if [ -f "$d/Cargo.lock" ]; then repo_root=$d; break; fi
    d=$(dirname "$d")
  done
fi
[ -n "$repo_root" ] || { echo "::error::no Cargo.lock found above $fe_dir"; exit 1; }

cache="${XDG_CACHE_HOME:-$HOME/.cache}/trunk"

# Read a `key = "value"` pin out of Trunk.toml's [tools] section.
tool_pin() {
  awk -v want="$1" '
    /^\[tools\]/ { in_tools = 1; next }
    /^\[/       { in_tools = 0 }
    in_tools && $1 == want {
      line = $0
      sub(/^[^=]*= */, "", line)
      gsub(/"/, "", line)
      print line
      exit
    }
  ' "$trunk_toml"
}

lock_version() {
  awk -v want="$1" '
    $1 == "name" && $3 == "\"" want "\"" { getline; gsub(/"/, "", $3); print $3; exit }
  ' "$repo_root/Cargo.lock"
}

# download <url> <dest-file> — curl owns the retries; trunk's budget cannot.
download() {
  local url=$1 dest=$2 tmp
  tmp=$(mktemp -d)
  curl -fsSL --retry 5 --retry-delay 5 --retry-all-errors --connect-timeout 30 \
    -o "$tmp/archive" "$url"
  mkdir -p "$(dirname "$dest")"
  install -m 0755 "$tmp/archive" "$dest"
  rm -rf "$tmp"
}

# download_member <url> <member> <dest-file> — same, for a member of a tarball.
download_member() {
  local url=$1 member=$2 dest=$3 tmp
  tmp=$(mktemp -d)
  curl -fsSL --retry 5 --retry-delay 5 --retry-all-errors --connect-timeout 30 \
    -o "$tmp/archive.tgz" "$url"
  tar -xzf "$tmp/archive.tgz" -C "$tmp" "$member"
  mkdir -p "$(dirname "$dest")"
  install -m 0755 "$tmp/$member" "$dest"
  rm -rf "$tmp"
}

seed_tailwindcss() {
  local v=$1 dest="$cache/tailwindcss-$1/tailwindcss"
  [ -x "$dest" ] && { echo "tailwindcss $v already cached"; return; }
  download \
    "https://github.com/tailwindlabs/tailwindcss/releases/download/v$v/tailwindcss-linux-x64" \
    "$dest"
  echo "seeded tailwindcss $v"
}

seed_wasm_opt() {
  local v=$1 dest="$cache/wasm-opt-$1/bin/wasm-opt"
  [ -x "$dest" ] && { echo "wasm-opt $v already cached"; return; }
  download_member \
    "https://github.com/WebAssembly/binaryen/releases/download/$v/binaryen-$v-x86_64-linux.tar.gz" \
    "binaryen-$v/bin/wasm-opt" "$dest"
  "$dest" --version
}

seed_wasm_bindgen() {
  local v=$1 dest="$cache/wasm-bindgen-$1/wasm-bindgen"
  [ -x "$dest" ] && { echo "wasm-bindgen $v already cached"; return; }
  download_member \
    "https://github.com/rustwasm/wasm-bindgen/releases/download/$v/wasm-bindgen-$v-x86_64-unknown-linux-musl.tar.gz" \
    "wasm-bindgen-$v-x86_64-unknown-linux-musl/wasm-bindgen" "$dest"
  "$dest" --version
}

# Every [tools] pin is handled, or this fails. Adding a pin without teaching
# this script to fetch it would otherwise silently reintroduce the download.
mapfile -t pins < <(awk '
  /^\[tools\]/ { in_tools = 1; next }
  /^\[/       { in_tools = 0 }
  in_tools && /^[A-Za-z_]+ *=/ { sub(/ *=.*/, ""); print }
' "$trunk_toml")
[ ${#pins[@]} -gt 0 ] || { echo "::error file=$trunk_toml::no [tools] pins to pre-seed"; exit 1; }

for key in "${pins[@]}"; do
  case "$key" in
    tailwindcss)
      v=$(tool_pin tailwindcss)
      seed_tailwindcss "$v"
      ;;
    wasm_opt)
      v=$(tool_pin wasm_opt)
      seed_wasm_opt "$v"
      ;;
    wasm_bindgen)
      v=$(tool_pin wasm_bindgen)
      seed_wasm_bindgen "$v"
      ;;
    *)
      echo "::error file=$trunk_toml::[tools] $key has no pre-seed rule; trunk will download it at build time"
      exit 1
      ;;
  esac
done

# wasm-bindgen is usually NOT pinned in [tools] — trunk resolves it from
# Cargo.lock, and the CLI baked into the image may be a different version
# (which trunk then ignores in favour of a download). Seed the lock's version.
if ! printf '%s\n' "${pins[@]}" | grep -qx wasm_bindgen; then
  v=$(lock_version wasm-bindgen)
  [ -n "$v" ] || { echo "::error::wasm-bindgen not found in $repo_root/Cargo.lock"; exit 1; }
  seed_wasm_bindgen "$v"
fi
