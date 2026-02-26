#!/bin/sh

# Generate config.json from environment variables before starting nginx
cat <<EOT > /usr/share/nginx/html/config.json
{
  "PORTAL_URL": "${PORTAL_URL:-https://10.1.7.121/portal}",
  "AUTHENC_URL": "${AUTHENC_URL:-https://10.1.7.121/api/auth}"
}
EOT

# Execute the default CMD (nginx)
exec "$@"
