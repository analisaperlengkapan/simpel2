#!/bin/bash
set -e

echo "Waiting for services to be healthy..."

check_health() {
  url=$1
  service=$2
  max_retries=150
  count=0

  echo "Checking $service at $url..."
  until curl -s -f "$url" > /dev/null; do
    sleep 2
    count=$((count+1))
    if [ $count -ge $max_retries ]; then
      echo "Timeout waiting for $service"
      return 1
    fi
    echo "Waiting for $service... ($count/$max_retries)"
  done
  echo "$service is HEALTHY!"
}

# Authenc
check_health "http://localhost:8088/health" "Authenc HTTP"
# Secreton
check_health "http://localhost:8200/health" "Secreton HTTP"
# Portal Backend
check_health "http://localhost:8081/api/v1/health" "Portal Backend"
# Portal Frontend (Nginx)
check_health "http://localhost:8090" "Portal Frontend"

echo "All services are running and healthy."
