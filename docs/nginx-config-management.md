# Nginx Configuration Management Integration

## Overview

The enhanced nginx configuration management system for SIMPelv2 provides automated generation, validation, and deployment of nginx configurations for both microfrontend containers and infrastructure reverse proxy.

## Components

### 1. Enhanced Generator (`nginx-config-generator.py`)
- **Auto-discovery** of microfrontend services from antarmuka directory
- **Template-based generation** using shared template with variable substitution
- **Infrastructure integration** for reverse proxy configuration
- **Validation support** with comprehensive syntax checking

### 2. Management Script (`nginx-manager.sh`)
- **Unified CLI** for all nginx configuration operations
- **Docker-aware validation** that handles containerized environments
- **Deployment automation** with Docker Compose integration
- **Status monitoring** and configuration reporting

## Generated Files

### Microfrontend Configurations
```
antarmuka/shared/nginx/generated/
├── nginx-badiklat.conf
├── nginx-datun.conf
├── nginx-intel.conf
├── nginx-pemulihan_aset.conf
├── nginx-pengawasan.conf
├── nginx-pidmil.conf
├── nginx-pidsus.conf
├── nginx-pidum.conf
└── nginx-portal.conf
```

### Infrastructure Configuration
```
infra/nginx/
├── nginx.conf           # Main reverse proxy
├── conf.d/
│   └── default.conf     # SSL/HTTPS configuration
└── certs/
    ├── fullchain.pem
    └── privkey.pem
```

## Template System

### Base Template (`microfrontend.conf`)
- **WASM optimizations** for Leptos applications
- **Security headers** with CSP for WASM
- **Compression settings** optimized for WASM files
- **SPA routing support** for Leptos Router
- **Health check endpoints**
- **API proxy configuration** to gerbang service

### Variable Substitution
- `{{SERVICE_NAME}}` → Service name (e.g., "portal", "badiklat")
- `{{SERVICE_NAME_UPPER}}` → Uppercase service name
- `{{SERVICE_NAME_TITLE}}` → Title case service name
- `{{TIMESTAMP}}` → Generation timestamp
- `{{SERVICE_PORT}}` → Service port (default: 3000)

## Commands

### Generation
```bash
# Generate all microfrontend configurations
./scripts/tools/nginx-manager.sh generate-all --validate

# Generate specific service configuration
./scripts/tools/nginx-manager.sh generate --service portal --validate

# Update infrastructure nginx configuration
./scripts/tools/nginx-manager.sh update-infra
```

### Validation
```bash
# Validate all generated configurations
./scripts/tools/nginx-manager.sh validate

# Show configuration status
./scripts/tools/nginx-manager.sh status
```

### Deployment
```bash
# Deploy configurations (dry run)
./scripts/tools/nginx-manager.sh deploy --dry-run

# Deploy configurations to running containers
./scripts/tools/nginx-manager.sh deploy
```

### Maintenance
```bash
# Clean generated configurations
./scripts/tools/nginx-manager.sh clean

# Show help
./scripts/tools/nginx-manager.sh help
```

## Integration Points

### Docker Compose
The nginx configurations are designed to work seamlessly with Docker Compose deployments:
- **Volume mounts** for configuration files
- **Service networking** using Docker container names
- **Health checks** for container orchestration

### CI/CD Pipeline
Can be integrated into GitLab CI/CD:
```yaml
generate_nginx:
  stage: build
  script:
    - ./scripts/tools/nginx-manager.sh generate-all --validate
  artifacts:
    paths:
      - antarmuka/shared/nginx/generated/
```

### Development Workflow
1. **Service Discovery** → Auto-detect microfrontends
2. **Configuration Generation** → Create nginx configs from template
3. **Validation** → Check syntax and structure
4. **Deployment** → Apply to running containers
5. **Monitoring** → Health checks and logging

## Optimizations Implemented

### Template Consolidation ✅
- Single shared template for all microfrontends
- Eliminated duplicate nginx configurations
- Centralized maintenance and updates

### Auto-Generation Enhancement ✅
- Service discovery from filesystem
- Dynamic template variable substitution
- Infrastructure nginx integration
- Comprehensive validation system

### Container-Aware Validation
- Docker hostname resolution awareness
- Syntax validation without network dependencies
- Production-ready configuration checking

## Future Enhancements

1. **SSL Certificate Management** - Automated certificate renewal
2. **Load Balancing Configuration** - Advanced upstream definitions  
3. **Rate Limiting Customization** - Per-service rate limit configuration
4. **Monitoring Integration** - Prometheus metrics and alerting
5. **Blue-Green Deployment** - Zero-downtime configuration updates

## Usage Examples

### Complete Setup
```bash
# 1. Generate all configurations
./scripts/tools/nginx-manager.sh generate-all --validate

# 2. Update infrastructure
./scripts/tools/nginx-manager.sh update-infra  

# 3. Check status
./scripts/tools/nginx-manager.sh status

# 4. Deploy to containers
./scripts/tools/nginx-manager.sh deploy
```

### Development Workflow
```bash
# Add new microfrontend service
mkdir antarmuka/new-service
echo 'name = "new-service"' > antarmuka/new-service/Cargo.toml

# Regenerate configurations
./scripts/tools/nginx-manager.sh generate-all

# Validate changes
./scripts/tools/nginx-manager.sh validate

# Apply to development environment
./scripts/tools/nginx-manager.sh deploy --dry-run
```

This enhanced nginx configuration management system provides enterprise-grade automation for SIMPelv2's microfrontend architecture while maintaining simplicity and reliability.
