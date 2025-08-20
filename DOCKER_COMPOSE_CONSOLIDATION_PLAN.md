# Docker Compose Consolidation Plan
## SIMPelv2 Container Architecture Simplification

### Current State Analysis
- 🔴 **5 Docker Compose Files** - Too many, causing confusion
- 🔴 **Redundant Services** - Same services defined multiple times
- 🔴 **Inconsistent Configurations** - Different service names and paths
- 🔴 **Maintenance Overhead** - Changes need to be made in multiple files

### Proposed Solution: 3-File Strategy

#### 1. `docker-compose.yml` (Base Configuration)
**Purpose**: Core services for all environments
**Contains**:
- All 11 microfrontends (standardized)
- PostgreSQL database
- Redis cache
- Nginx reverse proxy
- Basic health checks
- Standard network configuration

#### 2. `docker-compose.override.yml` (Development Default)
**Purpose**: Development-specific overrides (automatically loaded)
**Contains**:
- Exposed ports for development
- Volume mounts for hot reloading
- Development environment variables
- Logging configurations
- Debug settings

#### 3. `docker-compose.prod.yml` (Production Override)
**Purpose**: Production optimizations
**Contains**:
- Resource limits and reservations
- Security configurations
- Monitoring services (Prometheus, Grafana, Loki)
- Secrets management
- SSL/TLS configurations
- Production environment variables

### Migration Strategy

#### Phase 1: Consolidate Base Services
1. Standardize all microfrontend definitions in `docker-compose.yml`
2. Fix dockerfile paths and service names
3. Remove duplicated services across files

#### Phase 2: Separate Development Overrides
1. Create `docker-compose.override.yml` with development settings
2. Move port exposures and volume mounts from dev file
3. Add hot-reload capabilities

#### Phase 3: Optimize Production Configuration
1. Merge `docker-compose.prod.yml` and `docker-compose.production.yml`
2. Add monitoring stack to production override
3. Implement proper secrets management

#### Phase 4: Remove Redundant Files
1. Archive `docker-compose.dev.yml`
2. Archive `docker-compose.production.yml`
3. Archive `docker-compose.secure.yml`
4. Update documentation and scripts

### Benefits of Consolidation

#### ✅ Simplified Operations
- **Development**: `docker compose up` (auto-loads override.yml)
- **Production**: `docker compose -f docker-compose.yml -f docker-compose.prod.yml up`
- **Testing**: `docker compose -f docker-compose.yml up` (base only)

#### ✅ Reduced Maintenance
- Single source of truth for service definitions
- Consistent naming and paths
- Easier updates and changes

#### ✅ Better Documentation
- Clear separation of concerns
- Standard Docker Compose patterns
- Easier onboarding for new developers

#### ✅ Improved Security
- Centralized security configurations
- Consistent secret management
- Better environment separation

### Implementation Timeline
- **Week 1**: Phase 1 - Consolidate base services
- **Week 2**: Phase 2 - Development overrides  
- **Week 3**: Phase 3 - Production optimization
- **Week 4**: Phase 4 - Cleanup and documentation

### Risk Mitigation
- Backup existing configurations
- Test each phase thoroughly
- Gradual rollout with fallback plans
- Update CI/CD pipelines accordingly

### Success Metrics
- Reduced configuration files from 5 to 3
- Consistent service definitions across environments
- Faster development setup time
- Reduced maintenance overhead
- Improved developer experience

---
*Generated: August 20, 2025*
*Status: Proposal - Awaiting approval for implementation*
