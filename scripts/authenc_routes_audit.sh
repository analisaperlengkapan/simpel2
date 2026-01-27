echo "--- Mapping OIDC Discovery ---"
curl -s http://localhost:8088/.well-known/openid-configuration | grep -o '"[^"]*":'

echo -e "\n--- Mapping JWT Jwks ---"
curl -s http://localhost:8088/v1/oidc/jwks || echo "Jwks not reachable"

echo -e "\n--- Mapping Health check ---"
curl -s http://localhost:8088/health
