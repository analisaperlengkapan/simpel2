#!/usr/bin/env bash
# Every simpelv2 image the chart renders at <tag> must carry a cosign signature made by this repository's
# <release|promote> workflow, or nothing is deployed.
#
#   verify-images.sh <values-file> <tag> <release|promote>
#
# Needs helm and cosign on PATH, GITHUB_REPOSITORY, GITHUB_ACTOR and GITHUB_TOKEN (cosign does not always
# read ~/.docker/config.json on the self-hosted runner, so the registry credential is passed explicitly,
# as release.yml does when it signs).
set -euo pipefail

values="${1:?values file}"
tag="${2:?tag}"
workflow="${3:?release|promote}"
case "$workflow" in release | promote) ;; *) echo "workflow must be release or promote" >&2; exit 2 ;; esac
repo="${GITHUB_REPOSITORY:?GITHUB_REPOSITORY}"
min="${MIN_IMAGES:-6}"
chart="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../helm/simpel" && pwd)"

images="$(helm template simpel "$chart" -f "$values" \
  | grep -oE "ghcr\.io/[A-Za-z0-9._/-]+/simpelv2-[A-Za-z0-9._-]+:${tag}" | sort -u || true)"
count="$(grep -c . <<< "$images" || true)"
if [ "$count" -lt "$min" ]; then
  echo "::error::the chart renders $count simpelv2 image(s) at ${tag}, expected at least ${min}; refusing to deploy on a partial list"
  exit 1
fi
for img in $images; do
  echo "cosign verify ${img}"
  cosign verify \
    --registry-username "${GITHUB_ACTOR:?}" --registry-password "${GITHUB_TOKEN:?}" \
    --certificate-identity-regexp "^https://github.com/${repo}/\\.github/workflows/${workflow}\\.yml@refs/tags/v" \
    --certificate-oidc-issuer https://token.actions.githubusercontent.com \
    "$img" > /dev/null
done
echo "all ${count} images verified (${workflow}.yml, ${tag})"
