#!/bin/sh

# Generate config.json from environment variables before starting nginx
cat <<EOT > /usr/share/nginx/html/perlengkapan/config.json
{
  "api_url": "${API_URL:-http://10.1.7.121/api/perlengkapan}",
  "authenc_url": "${AUTHENC_URL:-http://10.1.7.121/api/auth}"
}
EOT

# Execute the default CMD (nginx)
exec "$@"
