#!/usr/bin/env bash
# Build the six e2e images ONCE (shared by every e2e job) with BuildKit's
# registry cache, and leave them in the local daemon as `simpel2-*:local`.
#
# Why a registry cache: the runner's dind is ephemeral, so `docker compose build`
# started from an empty layer cache every run — `cargo install cargo-chef` plus a
# cold dependency build (`cargo chef cook`) for each of six images, ~40-55 min.
# Those layers only change when a manifest or Dockerfile does, so they are
# exported to the in-cluster registry and imported by the next build.
#
# Read vs write:
#   * Every build IMPORTS `cache/<image>:main`. A miss is harmless (BuildKit
#     prints "not found" and builds from scratch).
#   * Only a build with E2E_CACHE_WRITE=true EXPORTS. CI sets that for a push to
#     main that changed a Cargo manifest or Dockerfile (or when asked by hand).
#     Exporting on every push is what you must NOT do here: `mode=max` also
#     exports the large `cargo build` layer, which differs on every commit, and
#     the registry has no garbage collector, so it would grow by gigabytes per
#     merge. A PR never writes, so a PR cannot poison the cache main reads.
#
# Cache names match the `build-docker` matrix names so the main-branch image
# check and the release build read the same entries.
#
# Needs: a buildx builder with the docker-container driver that trusts the
# registry (`.github/buildkitd.toml`), selected as the current builder.
#
# Env:
#   E2E_REGISTRY      registry host:port           (required)
#   E2E_CACHE_WRITE   "true" to export the cache   (default: false)
#   E2E_CACHE_REPO    registry path holding the cache (default: cache); set
#                     another one to try this script without touching CI's cache
#   E2E_SERVICES      space-separated compose services to build (default: all six)
set -euo pipefail

: "${E2E_REGISTRY:?E2E_REGISTRY is required, e.g. 172.15.10.252:32000}"
WRITE="${E2E_CACHE_WRITE:-false}"
CACHE_REPO="${E2E_CACHE_REPO:-cache}"

# compose service -> cache/image name (the build-docker matrix name)
declare -A CACHE_NAME=(
  [authenc]=layanan-authenc
  [secreton]=layanan-secreton
  [layanan-integrasi]=layanan-integrasi
  [layanan-perlengkapan]=layanan-perlengkapan
  [perlengkapan]=antarmuka-perlengkapan
  [portal]=antarmuka-portal
)

read -r -a services <<<"${E2E_SERVICES:-authenc secreton layanan-integrasi layanan-perlengkapan perlengkapan portal}"

args=()
for svc in "${services[@]}"; do
  name="${CACHE_NAME[$svc]:-}"
  if [[ -z "$name" ]]; then
    echo "::error::unknown e2e service '$svc' (no cache name mapped)" >&2
    exit 2
  fi
  ref="${E2E_REGISTRY}/${CACHE_REPO}/${name}:main"
  args+=(--set "${svc}.cache-from=type=registry,ref=${ref}")
  if [[ "$WRITE" == "true" ]]; then
    # ignore-error: a cache write is an optimisation and must never fail a build
    # whose content is correct (the registry being down is not a code defect).
    args+=(--set "${svc}.cache-to=type=registry,ref=${ref},mode=max,image-manifest=true,oci-mediatypes=true,ignore-error=true")
  fi
done

echo "building: ${services[*]} (cache write: ${WRITE})"
exec docker buildx bake \
  -f docker-compose.yml -f docker-compose.e2e.yml \
  --load \
  "${args[@]}" \
  "${services[@]}"
