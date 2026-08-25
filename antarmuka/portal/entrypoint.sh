#!/bin/sh

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
