#!/bin/bash
# Deploy Portal Microfrontend dan Microservice

set -e

echo "╔════════════════════════════════════════════════════╗"
echo "║  Portal Deployment - Microfrontend + Microservice ║"
echo "╚════════════════════════════════════════════════════╝"
echo ""

# Ensure network exists
echo "📡 Creating network..."
docker network create simpelv2-network 2>/dev/null || echo "  ✓ Network already exists"
echo ""

# Build Backend (Microservice)
echo "🔧 Building Backend Microservice (layanan-portal)..."
cd /var/www/simpelv2/layanan/daskrimti/portal
docker compose build layanan-portal
echo "  ✓ Backend build complete"
echo ""

# Start Backend Services
echo "🚀 Starting Backend Services..."
docker compose up -d
echo "  ✓ Backend services started"
echo ""

# Build Frontend (Microfrontend)
echo "🎨 Building Frontend Microfrontend (portal-microfrontend)..."
cd /var/www/simpelv2/antarmuka/daskrimti/portal
docker compose build portal-microfrontend
echo "  ✓ Frontend build complete"
echo ""

# Start Frontend
echo "🚀 Starting Frontend..."
docker compose up -d
echo "  ✓ Frontend started"
echo ""

# Show status
echo "╔════════════════════════════════════════════════════╗"
echo "║  Deployment Status                                 ║"
echo "╚════════════════════════════════════════════════════╝"
echo ""
docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}" | grep -E "NAME|portal|layanan-portal"
echo ""
echo "✅ Portal services deployed!"
echo ""
echo "Access URLs:"
echo "  Frontend: http://localhost:8090"
echo "  Backend:  http://localhost:8081"
echo ""
echo "Health checks:"
echo "  curl http://localhost:8081/health"
