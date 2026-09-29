#!/usr/bin/env bash
# Local e2e harness: bring up the compose stack, seed fixtures, then run a
# Playwright project set. Mirrors the CI e2e-perlengkapan / e2e-portal jobs.
#
#   ./infra/scripts/run-e2e.sh up                # build+up stack, seed fixtures
#   ./infra/scripts/run-e2e.sh stack-wait        # wait for services to be ready
#   ./infra/scripts/run-e2e.sh perl <projects>   # run perlengkapan projects (space separated)
#   ./infra/scripts/run-e2e.sh portal <projects> # run portal projects
#   ./infra/scripts/run-e2e.sh down
set -uo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO" || exit 1
COMPOSE="docker compose -f docker-compose.yml -f docker-compose.e2e.yml"
NET="$(docker network ls --format '{{.Name}}' | grep -m1 simpel-network || true)"

cmd_up() {
  $COMPOSE up -d postgres redis authenc secreton layanan-integrasi layanan-perlengkapan perlengkapan portal
}

# Cross-app SSO stack: both FEs + the single-origin ingress (--profile cross-app).
cmd_up_cross() {
  $COMPOSE --profile cross-app up -d \
    postgres redis authenc layanan-integrasi layanan-perlengkapan perlengkapan portal cross-app-ingress
}

cmd_wait() {
  NET="$(docker network ls --format '{{.Name}}' | grep -m1 simpel-network)"
  echo "stack network: ${NET:?simpel-network not found}"
  docker run --rm --network "$NET" curlimages/curl:8.11.1 sh -c '
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
  for f in tests/fixtures/e2e/seed-multisatker.sql tests/fixtures/e2e/seed-perlengkapan-workflow.sql; do
    echo ">>> applying $f"
    $COMPOSE run -T --rm e2e-seed \
      -v ON_ERROR_STOP=1 -h postgres -U postgres -d dbsimpelv2 -f - < "$f" \
      || { echo "seed $f FAILED"; return 1; }
  done
}

cmd_perl() {
  NET="$(docker network ls --format '{{.Name}}' | grep -m1 simpel-network)"
  docker build -q -t simpel-e2e-smoke \
    -f antarmuka/perlengkapan/tests/e2e/Dockerfile.e2e \
    antarmuka/perlengkapan/tests/e2e || return 1
  docker run --rm --network "$NET" \
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
  cp layanan/integrasi/proto/integrasi.proto \
     antarmuka/portal/tests/e2e/integrasi.proto
}

cmd_portal() {
  NET="$(docker network ls --format '{{.Name}}' | grep -m1 simpel-network)"
  cmd_portal_proto
  docker build -q -t simpel-e2e-portal \
    -f antarmuka/portal/tests/e2e/Dockerfile.e2e \
    antarmuka/portal/tests/e2e || return 1
  docker run --rm --network "$NET" \
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
  NET="$(docker network ls --format '{{.Name}}' | grep -m1 simpel-network)"
  docker build -q -t simpel-e2e-portal \
    -f antarmuka/portal/tests/e2e/Dockerfile.e2e \
    antarmuka/portal/tests/e2e || return 1
  docker run --rm --network "$NET" \
    -e BASE_URL=http://cross-app-ingress:8080 \
    simpel-e2e-portal \
    npx playwright test --project=portal-cross-app
}

cmd_down() { $COMPOSE down -v; }

case "${1:-}" in
  up)         cmd_up ;;
  up-cross)   cmd_up_cross ;;
  wait)       cmd_wait ;;
  seed)       cmd_seed ;;
  perl)       shift; cmd_perl "$@" ;;
  portal)     shift; cmd_portal "$@" ;;
  portal-cross) cmd_portal_cross ;;
  down)       cmd_down ;;
  *) echo "usage: $0 {up|up-cross|wait|seed|perl|portal|portal-cross|down}"; exit 2 ;;
esac
