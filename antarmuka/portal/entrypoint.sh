#!/bin/sh

# Substitute the nginx upstream host:port placeholders from env.
#
# Same shape as perlengkapan's entrypoint, and for the same reason: the image
# ships nginx.conf with placeholders, and docker-compose overrides them via env
# (K8s Services are 8093/8091; compose binds 3020/8088).
#
# Under Helm the chart mounts its own rendered nginx.conf over this path. A
# ConfigMap mount is read-only regardless of file ownership and the rendered
# config carries no placeholders, so substitution is skipped there — and, as in
# perlengkapan, `sed -i` must not run unconditionally under `set -e` (it writes
# its temp file into the target's directory and would CrashLoop the container).
PORTAL_PERLENGKAPAN_API_UPSTREAM="${PERLENGKAPAN_API_UPSTREAM:-layanan-perlengkapan:8093}"
PORTAL_AUTHENC_API_UPSTREAM="${AUTHENC_API_UPSTREAM:-authenc:8091}"
if grep -q '__PERLENGKAPAN_API_UPSTREAM__\|__AUTHENC_API_UPSTREAM__' /etc/nginx/nginx.conf 2>/dev/null; then
  if sed -i \
      -e "s|__PERLENGKAPAN_API_UPSTREAM__|${PORTAL_PERLENGKAPAN_API_UPSTREAM}|g" \
      -e "s|__AUTHENC_API_UPSTREAM__|${PORTAL_AUTHENC_API_UPSTREAM}|g" \
      /etc/nginx/nginx.conf 2>/dev/null; then
    echo "entrypoint: nginx.conf upstream placeholders substituted"
  else
    # Placeholders present but unwritable: nginx would proxy to a literal
    # `__..__` host. Fail loudly rather than serve a config that cannot work.
    echo "entrypoint: FATAL - nginx.conf has upstream placeholders but is not writable" >&2
    exit 1
  fi
else
  echo "entrypoint: no upstream placeholders in nginx.conf (Helm ConfigMap already rendered) - skipping substitution"
fi

# Generate config.json from environment variables before starting nginx.
#
# Under Helm this NEVER succeeds: the chart sets `readOnlyRootFilesystem: true`,
# so the web root is not writable. Verified directly against staging on
# 2026-08-25 — the running portal pod has no config.json at all and the failure
# only ever appeared as a bare `can't create ...: Read-only file system` line
# that nothing looked at. The FE keeps working on build-time defaults, so this
# is not fatal; it is just invisible. Made explicit below so the degraded state
# is readable. Real fix = serve config.json from a chart ConfigMap (like
# nginx.conf) so nothing is written at runtime.
CONFIG_JSON=/usr/share/nginx/html/config.json
if cat <<EOT > "$CONFIG_JSON" 2>/dev/null
{
  "portal_url": "${PORTAL_URL:-http://10.1.7.121/portal}",
  "authenc_url": "${AUTHENC_URL:-http://10.1.7.121/api/auth}"
}
EOT
then
  echo "entrypoint: wrote $CONFIG_JSON"
else
  echo "entrypoint: WARNING - could not write $CONFIG_JSON (read-only web root); FE falls back to build-time defaults" >&2
fi

# Execute the default CMD (nginx)
exec "$@"
