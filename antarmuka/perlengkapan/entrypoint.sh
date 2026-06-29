#!/bin/sh
set -e

# Substitute the nginx upstream host:port placeholders from env, defaulting to
# the K8s Service ports (so the Helm/K8s deployment, which sets no env, keeps its
# current behavior). docker-compose sets these to the compose ports so the
# in-browser FE->BE data path actually resolves. nginx.conf is chowned to the
# nginx user in the Dockerfile so this in-place edit works as uid 101.
PERLENGKAPAN_API_UPSTREAM="${PERLENGKAPAN_API_UPSTREAM:-layanan-perlengkapan:8093}"
AUTHENC_API_UPSTREAM="${AUTHENC_API_UPSTREAM:-authenc:8091}"
sed -i \
  -e "s|__PERLENGKAPAN_API_UPSTREAM__|${PERLENGKAPAN_API_UPSTREAM}|g" \
  -e "s|__AUTHENC_API_UPSTREAM__|${AUTHENC_API_UPSTREAM}|g" \
  /etc/nginx/nginx.conf

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
