# SIMPelv2 Docker Compose Usage Guide

## 📁 **Consolidated Structure (3 Files)**

After consolidation, we now have a clean 3-file structure:

```
docker-compose.yml      # ✅ Base configuration (all services)
docker-compose.dev.yml  # ✅ Development overrides  
docker-compose.prod.yml # ✅ Production overrides
```

---

## 🚀 **Usage Commands**

### **Development Environment**
```bash
# Start all services in development mode
docker compose -f docker-compose.yml -f docker-compose.dev.yml up -d

# View logs for specific service
docker compose -f docker-compose.yml -f docker-compose.dev.yml logs -f portal

# Stop all services
docker compose -f docker-compose.yml -f docker-compose.dev.yml down
```

### **Production Environment**
```bash
# Start all services in production mode
docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d

# Scale specific service
docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d --scale badiklat=3

# Stop all services
docker compose -f docker-compose.yml -f docker-compose.prod.yml down
```

### **Base Services Only (Testing)**
```bash
# Start only base services without overrides
docker compose up -d

# Useful for testing base configuration
docker compose logs nginx postgres redis
```

---

## 📦 **Service Portfolio**

### **🎯 Microfrontends (11 Services)**
- **portal** → Main entry point (dev: 8089, base: internal)
- **badiklat** → Training management (dev: 8081)
- **datun** → Legal affairs (dev: 8082)
- **intel** → Intelligence (dev: 8083)
- **keuangan** → Finance (dev: 8084)
- **pemulihan-aset** → Asset recovery (dev: 8085)
- **pengawasan** → Supervision (dev: 8086)
- **perencanaan** → Planning (dev: 8087)
- **perlengkapan** → Equipment (dev: 8088)
- **pidmil** → Military crimes (dev: 8090)
- **pidsus** → Special crimes (dev: 8091)
- **pidum** → General crimes (dev: 8092)

### **⚙️ Backend Services (3 Services)**
- **gerbang** → API Gateway (dev: 8080, admin: 9901)
- **layanan-keamanan** → Security service (dev: 3001)
- **layanan-integrasi** → Integration service (dev: 3002)

### **🗄️ Data Services (2 Services)**
- **postgres** → Primary database (dev: 5432)
- **redis** → Cache service (dev: 6379)

### **🔧 Infrastructure (1 Service)**
- **nginx** → Reverse proxy (dev: 80, 443)

### **📊 Development Tools (3 Services)**
Only available in development mode:
- **pgadmin** → Database admin (dev: 5050)
- **redis-insight** → Redis admin (dev: 8001)
- **loki** → Log aggregation (dev: 3100)

### **📈 Production Monitoring (4 Services)**
Only available in production mode:
- **prometheus** → Metrics (prod: 9090)
- **grafana** → Dashboards (prod: 3000)
- **loki** → Log aggregation (prod: 3100)
- **promtail** → Log shipping
- **alertmanager** → Alert management (prod: 9093)

---

## 🔧 **Configuration Details**

### **Base Configuration (docker-compose.yml)**
- All service definitions with consistent naming
- Health checks for all services
- Standard network configuration (simpelv2 + internal)
- Basic restart policies
- Volume definitions

### **Development Overrides (docker-compose.dev.yml)**
- Port exposure for all services
- Debug logging (RUST_LOG=debug)
- Volume mounts for hot-reloading
- Development database credentials
- Additional development tools

### **Production Overrides (docker-compose.prod.yml)**
- Resource limits and reservations
- Production logging (RUST_LOG=info)
- Docker secrets integration
- Enhanced restart policies
- Complete monitoring stack
- SSL/TLS configurations

---

## 🔐 **Security Configuration**

### **Development**
- Simple passwords for easy development
- Exposed ports for debugging
- Debug logging enabled

### **Production**
- Docker secrets for sensitive data
- No unnecessary port exposure
- Resource limits to prevent resource exhaustion
- Production-grade monitoring

### **Required Secrets for Production**
Create these files in `./secrets/` directory:
```
secrets/
├── db_user.txt
├── db_password.txt
├── redis_password.txt
├── grafana_password.txt
├── security_secret.txt
└── integration_secret.txt
```

---

## 🌍 **Network Architecture**

### **Networks**
- **simpelv2** → Public network (bridge, subnet: 172.20.0.0/16)
- **internal** → Private network (bridge, internal only)

### **Service Communication**
- External access → nginx → gerbang → backend services
- Databases only accessible from internal network
- Monitoring services on simpelv2 network

---

## 📋 **Common Tasks**

### **Check Service Status**
```bash
# Development
docker compose -f docker-compose.yml -f docker-compose.dev.yml ps

# Production  
docker compose -f docker-compose.yml -f docker-compose.prod.yml ps
```

### **View Logs**
```bash
# All services
docker compose -f docker-compose.yml -f docker-compose.dev.yml logs

# Specific service
docker compose -f docker-compose.yml -f docker-compose.dev.yml logs -f portal
```

### **Execute Commands in Container**
```bash
# Access PostgreSQL
docker compose -f docker-compose.yml -f docker-compose.dev.yml exec postgres psql -U simpelv2_dev -d simpelv2_dev

# Access Redis
docker compose -f docker-compose.yml -f docker-compose.dev.yml exec redis redis-cli
```

### **Scale Services**
```bash
# Scale microfrontends for load testing
docker compose -f docker-compose.yml -f docker-compose.dev.yml up -d --scale badiklat=3 --scale datun=3
```

---

## ✅ **Migration Benefits**

### **Before (5 Files)**
- ❌ Confusing file structure
- ❌ Duplicate service definitions  
- ❌ Inconsistent configurations
- ❌ High maintenance overhead

### **After (3 Files)**
- ✅ Clear separation of concerns
- ✅ Single source of truth for services
- ✅ Consistent naming and paths
- ✅ Easier maintenance and updates
- ✅ Standard Docker Compose patterns
- ✅ Better documentation

---

## 🔄 **Rollback Plan**

If needed, original configurations are backed up in:
```
backup/docker-compose-old/
├── docker-compose.yml
├── docker-compose.dev.yml  
├── docker-compose.prod.yml
├── docker-compose.production.yml
└── docker-compose.secure.yml
```

To rollback: `cp backup/docker-compose-old/* .`

---

*Updated: August 20, 2025*  
*Status: ✅ Consolidated - Ready for use*
