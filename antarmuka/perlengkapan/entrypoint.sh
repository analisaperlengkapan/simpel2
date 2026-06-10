#!/bin/sh

# Generate config.json di lokasi canonical microfrontend
# (/perlengkapan/simpel/v2/) — same path dengan WASM `<base href>`.
# Sebelumnya di /perlengkapan/, ikut path canonical baru.
cat <<EOT > /usr/share/nginx/html/perlengkapan/simpel/v2/config.json
{
  "api_url": "${API_URL:-http://10.1.7.121/api/v1/perlengkapan}",
  "authenc_url": "${AUTHENC_URL:-http://10.1.7.121/api/auth}"
}
EOT

# Execute the default CMD (nginx)
exec "$@"
