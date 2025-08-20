# SIMPelv2 Nginx Configuration Strategy
# Optimized: August 19, 2025

## Configuration Structure:

### 1. Individual Microfrontend Nginx Config
- Path: `antarmuka/shared/nginx/microfrontend.conf`
- Purpose: Template for individual microfrontend containers
- Usage: COPY in Dockerfile for each microfrontend

### 2. Infrastructure Reverse Proxy Config  
- Path: `infra/nginx/nginx.conf`
- Purpose: Production load balancer & service routing
- Usage: Docker Compose production deployment

### 3. Development Local Proxy Config
- Path: `infra/nginx/dev.conf`  
- Purpose: Development environment routing
- Usage: Local development with docker-compose.dev.yml

## Strategy:
- Remove: `antarmuka/nginx.conf` (duplicate)
- Create: Shared template in `antarmuka/shared/nginx/`
- Optimize: Infrastructure configs for production
- Add: Development-specific configuration
