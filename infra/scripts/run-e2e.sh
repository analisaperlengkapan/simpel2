#!/usr/bin/env bash
# Local e2e harness: bring up the compose stack, seed fixtures, then run a
# Playwright project set. Mirrors the CI e2e-perlengkapan / e2e-portal jobs.
#
#   ./infra/scripts/run-e2e.sh up                # build + up + wait + seed
#   ./infra/scripts/run-e2e.sh up-cross          # same, with the cross-app ingress
#   ./infra/scripts/run-e2e.sh build             # (re)build the app images only
#   ./infra/scripts/run-e2e.sh wait              # wait for services to be ready
#   ./infra/scripts/run-e2e.sh seed              # (re)apply the SQL fixtures
#   ./infra/scripts/run-e2e.sh perl <projects>   # run perlengkapan projects
#   ./infra/scripts/run-e2e.sh portal <projects> # run portal projects
#   ./infra/scripts/run-e2e.sh portal-cross      # cross-app SSO project
#   ./infra/scripts/run-e2e.sh down              # stop and remove volumes
#
# `up` is the whole sequence: rebuild the application images, start them, wait
# for readiness, load the fixtures. Skipping any step is how you get a run that
# fails for reasons unrelated to the code — a stale image, a service still
# booting, or an empty database that makes the workflow specs' non-empty
# assertions fail.
set -uo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO" || exit 1
COMPOSE="docker compose -f docker-compose.yml -f docker-compose.e2e.yml"

# Compose interpolates ${POSTGRES_PASSWORD:?...} and ${SIMPELV1_APP_KEY:?...}
# for every service in the merged files — including ones we never start — so a
# fresh checkout with no `.env` fails at interpolation time before anything
# runs. These are disposable local test credentials (the same values CI passes);
# an explicitly supplied value always wins.
: "${POSTGRES_PASSWORD:=e2e_test_pw}"
: "${SIMPELV1_APP_KEY:=base64:dGVzdGtleTMyYnl0ZXNzc3Nzc3Nzc3Nzc3Nzc3Nz}"
export POSTGRES_PASSWORD SIMPELV1_APP_KEY

# Services with a `build:` that the up-* targets start.
APP_SERVICES="authenc secreton gateway layanan-integrasi layanan-perlengkapan perlengkapan portal"
UP_SERVICES="postgres redis authenc secreton layanan-integrasi layanan-perlengkapan perlengkapan portal"
CROSS_SERVICES="postgres redis authenc layanan-integrasi layanan-perlengkapan perlengkapan portal cross-app-ingress"

PERL_E2E=antarmuka/perlengkapan/tests/e2e
PORTAL_E2E=antarmuka/portal/tests/e2e

net() { docker network ls --format '{{.Name}}' | grep -m1 simpel-network; }

cmd_build() {
  # Without this, `up -d` reuses whatever `simpel2-*:local` images already
  # exist and the suite tests the previous build. CI always builds first
  # (e2e-build-images) for exactly this reason.
  echo ">>> building app images"
  # shellcheck disable=SC2086
  $COMPOSE build $APP_SERVICES
}

cmd_wait() {
  local n
  n="$(net)" || { echo "simpel-network not found" >&2; return 1; }
  echo ">>> stack network: $n"
  docker run --rm --network "$n" curlimages/curl:8.11.1 sh -c '
    for i in $(seq 1 90); do
      if curl -sf http://authenc:8088/health >/dev/null \
         && curl -sf http://perlengkapan:8080/perlengkapan/ >/dev/null \
         && curl -sf http://layanan-perlengkapan:3020/health >/dev/null; then
        echo "stack ready after $i tries"; exit 0; fi
      sleep 5
    done
    echo "stack did not become ready"; exit 1'
}

cmd_seed() {
  # `e2e-seed` carries `profiles: [seed]`, so `up -d` never runs it; the
  # fixtures must be applied explicitly. Both files are idempotent. Re-run this
  # before repeating a suite: the workflow specs assert exact seeded
  # transitions (kebutuhan 2001→2002, pakaian 1001→1004, pemakaian I2,
  # penghapusan H1→H3), so a second run without a re-seed fails them in a way
  # that looks like a regression but is only consumed state.
  local f
  for f in tests/fixtures/e2e/seed-multisatker.sql tests/fixtures/e2e/seed-perlengkapan-workflow.sql; do
    echo ">>> applying $f"
    $COMPOSE run -T --rm e2e-seed \
      -v ON_ERROR_STOP=1 -h postgres -U postgres -d dbsimpelv2 -f - < "$f" \
      || { echo "seed $f FAILED" >&2; return 1; }
  done
}

# build → up → wait → seed, stopping at the first failure. `up -d` alone leaves
# the stack unbuilt, unready and unseeded.
cmd_up() {
  cmd_build && \
  $COMPOSE up -d $UP_SERVICES && \
  cmd_wait && \
  cmd_seed
}

cmd_up_cross() {
  cmd_build && \
  $COMPOSE --profile cross-app up -d $CROSS_SERVICES && \
  cmd_wait && \
  cmd_seed
}

cmd_perl() {
  local n
  n="$(net)" || { echo "simpel-network not found" >&2; return 1; }
  docker build -q -t simpel-e2e-smoke \
    -f "$PERL_E2E/Dockerfile.e2e" "$PERL_E2E" || return 1
  docker run --rm --network "$n" \
    -e BASE_URL=http://perlengkapan:8080 \
    -e PERLENGKAPAN_URL=http://perlengkapan:8080 \
    -e AUTHENC_URL=http://authenc:8088 \
    -e PERLENGKAPAN_API_URL=http://layanan-perlengkapan:3020 \
    simpel-e2e-smoke \
    npx playwright test "$@"
}

# The integrasi proto lives in the repo tree, OUTSIDE the portal e2e image
# context. Copy it in (gitignored) and locate it via INTEGRASI_PROTO_PATH,
# exactly as the CI e2e-integrasi-authenc job does.
cmd_portal_proto() {
  cp layanan/integrasi/proto/integrasi.proto "$PORTAL_E2E/integrasi.proto"
}

cmd_portal() {
  local n
  n="$(net)" || { echo "simpel-network not found" >&2; return 1; }
  cmd_portal_proto
  docker build -q -t simpel-e2e-portal \
    -f "$PORTAL_E2E/Dockerfile.e2e" "$PORTAL_E2E" || return 1
  docker run --rm --network "$n" \
    -e BASE_URL=http://portal:8080 \
    -e PORTAL_URL=http://portal:8080 \
    -e AUTHENC_URL=http://authenc:8088 \
    -e INTEGRASI_GRPC_URL=layanan-integrasi:50052 \
    -e INTEGRASI_PROTO_PATH=/e2e/integrasi.proto \
    simpel-e2e-portal \
    npx playwright test "$@"
}

# Cross-app SSO needs the single-origin ingress (mirrors the prod Istio VS).
cmd_portal_cross() {
  local n
  n="$(net)" || { echo "simpel-network not found" >&2; return 1; }
  docker build -q -t simpel-e2e-portal \
    -f "$PORTAL_E2E/Dockerfile.e2e" "$PORTAL_E2E" || return 1
  docker run --rm --network "$n" \
    -e BASE_URL=http://cross-app-ingress:8080 \
    simpel-e2e-portal \
    npx playwright test --project=portal-cross-app
}

cmd_down() { $COMPOSE down -v; }

case "${1:-}" in
  build)        cmd_build ;;
  up)           cmd_up ;;
  up-cross)     cmd_up_cross ;;
  wait)         cmd_wait ;;
  seed)         cmd_seed ;;
  perl)         shift; cmd_perl "$@" ;;
  portal)       shift; cmd_portal "$@" ;;
  portal-cross) cmd_portal_cross ;;
  down)         cmd_down ;;
  *)
    echo "usage: $0 {build|up|up-cross|wait|seed|perl|portal|portal-cross|down}" >&2
    exit 2
    ;;
esac
