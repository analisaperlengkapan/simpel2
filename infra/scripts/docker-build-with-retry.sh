#!/usr/bin/env bash
# Retry `docker build` (and the base-image pull it needs) against a flaky
# registry.
#
# WHY: every e2e job builds a Playwright runner image
# `FROM mcr.microsoft.com/playwright:v1.60.0-jammy`. BuildKit resolves that
# tag's manifest over the network BEFORE compiling anything, and
# mcr.microsoft.com has repeatedly dropped those connections mid-TLS:
#
#   failed to resolve source metadata for mcr.microsoft.com/playwright:v1.60.0-jammy:
#     Head "https://mcr.microsoft.com/v2/playwright/manifests/v1.60.0-jammy":
#     net/http: TLS handshake timeout     (job 111578628196)
#     ... read: connection reset by peer  (job 111580763516)
#
# That has nothing to do with the code under test, yet it turns `CI Summary`
# red — the only required check besides `Security Summary` — and blocks every
# PR in the repo at once. It is the same rule the repo already applies to
# advisory-DB and GitHub-release fetches: never hang a build on a network read
# without pin + retry.
#
# Usage: docker-build-with-retry.sh <tag> <dockerfile> <context> [attempts]

set -uo pipefail

TAG="${1:?usage: docker-build-with-retry.sh <tag> <dockerfile> <context> [attempts]}"
DOCKERFILE="${2:?missing dockerfile}"
CONTEXT="${3:?missing build context}"
ATTEMPTS="${4:-4}"
DELAY="${RETRY_BASE_DELAY:-10}"

# Best-effort warm-up of the images named by `FROM`. BuildKit re-resolves the
# manifest itself, so this cannot replace the retry below; it only keeps the
# (slow, most failure-prone) blob download out of the retried path.
prepull_bases() {
  local image
  while IFS= read -r image; do
    [ -n "$image" ] || continue
    case "$image" in
      scratch | "$TAG") continue ;;
    esac
    if ! docker pull "$image"; then
      echo "::warning::pre-pull $image failed; the build will try it again itself"
    fi
  done < <(
    # `FROM [--platform=x] image[:tag] [AS stage]` → image[:tag]. Skip flag
    # args; the tag is kept because `docker pull image:tag` is what we want.
    sed -nE 's/^[[:space:]]*FROM[[:space:]]+(.*)$/\1/p' "$DOCKERFILE" |
      awk '{ for (i = 1; i <= NF; i++) { if ($i ~ /^--/) continue; print $i; break } }' |
      grep -v '^scratch$' | sort -u
  )
}

prepull_bases

attempt=1
while true; do
  if docker build -t "$TAG" -f "$DOCKERFILE" "$CONTEXT"; then
    exit 0
  fi

  if [ "$attempt" -ge "$ATTEMPTS" ]; then
    echo "::error::docker build $TAG failed after $ATTEMPTS attempts"
    exit 1
  fi

  echo "::warning::docker build $TAG attempt $attempt/$ATTEMPTS failed (registry fetch?); retrying in ${DELAY}s"
  sleep "$DELAY"
  attempt=$((attempt + 1))
  DELAY=$((DELAY * 2))
done
