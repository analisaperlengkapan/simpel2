#!/bin/sh

# Generate config.json from environment variables before starting nginx
cat <<EOT > /usr/share/nginx/html/config.json
{
  "portal_url": "${PORTAL_URL:-http://10.1.7.121/portal}",
  "authenc_url": "${AUTHENC_URL:-http://10.1.7.121/api/auth}"
}
EOT

# Execute the default CMD (nginx)
exec "$@"
