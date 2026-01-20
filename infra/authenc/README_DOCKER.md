# Authenc IAM Service - Docker Deployment Guide

## 🚀 Quick Start (2 Minutes)

```bash
# 1. Navigate to authenc directory
cd /srv/proyek/simpelv2/infra/authenc

# 2. Setup environment
cp .env.example .env
nano .env  # Edit passwords and secrets

# 3. Start services
docker-compose up -d

# 4. Verify
curl http://localhost:8088/health
```

## 📋 What's New

This consolidated setup replaces multiple CAPTCHA-specific configuration files with a unified, production-ready system:

### ✅ Removed (Consolidated)
- `.env.captcha.development.example`
- `.env.captcha.production.example`
- `docker-compose.captcha.yml`

### ✅ Created (Unified)
- `authenc.toml` - Single configuration file with integrated CAPTCHA
- `.env.example` - Environment variable template
- `docker-compose.yml` - Complete service stack
- `docker-compose.prod.yml` - Production optimizations
- `docker-start.sh` - Automated startup script
- Comprehensive documentation

## 📚 Documentation

| Document | Purpose |
|----------|---------|
| [QUICKSTART.md](./QUICKSTART.md) | 5-minute quick reference |
| [DOCKER_SETUP.md](./DOCKER_SETUP.md) | Complete setup guide |
| [PRODUCTION_READINESS.md](./PRODUCTION_READINESS.md) | Production deployment checklist |
| [MIGRATION_SUMMARY.md](./MIGRATION_SUMMARY.md) | What changed and why |
| [authenc.toml](./authenc.toml) | Configuration reference |

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Docker Network                        │
├─────────────────────────────────────────────────────────┤
│                                                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │  PostgreSQL  │  │    Redis     │  │   Authenc    │  │
│  │   (5432)     │  │   (6379)     │  │   (8088)     │  │
│  └──────────────┘  └──────────────┘  └──────────────┘  │
│                                                           │
│  ┌──────────────┐  ┌──────────────┐                     │
│  │ Prometheus   │  │   Grafana    │                     │
│  │  (9091)      │  │   (3000)     │                     │
│  └──────────────┘  └──────────────┘                     │
│                                                           │
└─────────────────────────────────────────────────────────┘
```

## 🔧 Services

### PostgreSQL (Port 5432)
- **Image**: postgres:16-alpine
- **Database**: authenc
- **Volume**: authenc-postgres-data
- **Health Check**: Enabled

### Redis (Port 6379)
- **Image**: redis:7-alpine
- **Purpose**: Cache, sessions, rate limiting
- **Volume**: authenc-redis-data
- **Health Check**: Enabled

### Authenc (Port 8088)
- **API**: http://localhost:8088
- **gRPC**: localhost:9088
- **Metrics**: http://localhost:9090/metrics
- **Health Check**: Enabled

### Prometheus (Port 9091)
- **Image**: prom/prometheus:latest
- **Purpose**: Metrics collection
- **Volume**: authenc-prometheus-data

### Grafana (Port 3000)
- **Image**: grafana/grafana:latest
- **Purpose**: Visualization
- **Volume**: authenc-grafana-data
- **Default**: admin / (from .env)

## 🚀 Deployment Modes

### Development (Default)
```bash
docker-compose up -d
```
- Debug logging
- Lower CAPTCHA difficulty
- Development defaults
- Faster startup

### Production
```bash
docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d
```
- Production hardening
- Security optimizations
- Enhanced monitoring
- Resource limits

### Custom
```bash
# Edit .env with your settings
cp .env.example .env
nano .env

# Then start
docker-compose up -d
```

## 📊 Configuration

### Unified Configuration File: `authenc.toml`

```toml
[server]
host = "0.0.0.0"
port = 8088
tls_enabled = false

[database]
host = "postgres"
port = 5432
database = "authenc"

[captcha]
enabled = true
default_difficulty = 3
# ... more CAPTCHA settings

[security]
jwt_secret = "${JWT_SECRET}"
# ... more security settings
```

### Environment Variables: `.env`

```bash
# Database
DB_USER=postgres
DB_PASSWORD=strong_password
DATABASE_URL=postgresql://...

# Security
JWT_SECRET=your_secret_here
ED25519_PRIVATE_KEY_BASE64=...

# CAPTCHA
CAPTCHA_ENABLED=true
CAPTCHA_DEFAULT_DIFFICULTY=3

# Services
SECRETON_API_URL=http://secreton:8200
```

## 🔐 Security Features

- ✅ Non-root user execution (UID 1000)
- ✅ Read-only root filesystem support
- ✅ Health checks for all services
- ✅ Network isolation (private Docker network)
- ✅ Secret management via environment variables
- ✅ TLS/HTTPS support
- ✅ Rate limiting and CAPTCHA
- ✅ Audit logging

## 📈 Monitoring

### Prometheus
- URL: http://localhost:9091
- Metrics: http://localhost:9090/metrics
- Scrape interval: 15 seconds

### Grafana
- URL: http://localhost:3000
- Username: admin
- Password: (from GRAFANA_PASSWORD in .env)

### Key Metrics
- Request latency (p50, p95, p99)
- Error rate
- Authentication success rate
- Database connection pool
- Redis memory usage

## 🛠️ Common Commands

### View Status
```bash
docker-compose ps
docker-compose logs -f authenc
```

### Database
```bash
docker-compose exec postgres psql -U postgres -d authenc
```

### Redis
```bash
docker-compose exec redis redis-cli -a ${REDIS_PASSWORD}
```

### Restart
```bash
docker-compose restart authenc
docker-compose restart postgres
docker-compose restart redis
```

### Stop
```bash
docker-compose stop
docker-compose down
docker-compose down -v  # Remove volumes
```

## 🐛 Troubleshooting

### Services Won't Start
```bash
# Check logs
docker-compose logs authenc

# Verify environment
cat .env | grep -E "^[A-Z_]+"

# Check ports
netstat -tlnp | grep 8088
```

### Database Connection Failed
```bash
# Check PostgreSQL
docker-compose logs postgres

# Test connection
docker-compose exec postgres psql -U postgres -c "SELECT 1;"
```

### High Memory Usage
```bash
# Check stats
docker stats

# Reduce Redis memory in docker-compose.prod.yml
```

## 📦 Volumes

| Volume | Purpose | Persistence |
|--------|---------|-------------|
| authenc-postgres-data | Database files | Yes |
| authenc-redis-data | Cache data | Yes |
| authenc-logs | Application logs | Yes |
| authenc-prometheus-data | Metrics | Yes |
| authenc-grafana-data | Dashboards | Yes |

## 🔄 Backup & Restore

### Backup Database
```bash
docker-compose exec postgres pg_dump -U postgres authenc | \
  gzip > backup-$(date +%Y%m%d).sql.gz
```

### Restore Database
```bash
gunzip < backup-20240101.sql.gz | \
  docker-compose exec -T postgres psql -U postgres authenc
```

## 🌍 Integration

### Secreton (Secrets Management)
```bash
SECRETON_API_URL=http://secreton:8200
SECRETON_TOKEN=your_token
```

### External Services
```bash
# SMTP for email
SMTP_HOST=mail.example.com
SMTP_PORT=587
SMTP_USER=user@example.com
SMTP_PASSWORD=password
```

## 📋 Pre-Deployment Checklist

- [ ] Copy .env.example to .env
- [ ] Update all passwords in .env
- [ ] Generate JWT_SECRET
- [ ] Generate signing keys (production)
- [ ] Configure CORS_ALLOWED_ORIGINS
- [ ] Set AUTHENC_ENV=production
- [ ] Enable TLS if needed
- [ ] Configure email settings
- [ ] Set up backups
- [ ] Review security settings

## 🚀 Production Deployment

1. **Prepare Environment**
   ```bash
   cp .env.example .env
   nano .env  # Set all production values
   ```

2. **Generate Keys**
   ```bash
   cargo run --bin generate-signing-keys
   # Add to .env
   ```

3. **Build Image**
   ```bash
   docker build -t authenc:latest .
   docker tag authenc:latest registry.example.com/authenc:latest
   docker push registry.example.com/authenc:latest
   ```

4. **Deploy**
   ```bash
   docker-compose -f docker-compose.yml \
                  -f docker-compose.prod.yml \
                  up -d
   ```

5. **Verify**
   ```bash
   curl http://localhost:8088/health
   docker-compose ps
   ```

## 📞 Support

For issues:
1. Check logs: `docker-compose logs -f`
2. Review config: `cat authenc.toml`
3. Test health: `curl http://localhost:8088/health`
4. See [DOCKER_SETUP.md](./DOCKER_SETUP.md) for detailed troubleshooting

## 📖 Additional Resources

- [Authenc Documentation](../docs/)
- [Docker Documentation](https://docs.docker.com/)
- [PostgreSQL Documentation](https://www.postgresql.org/docs/)
- [Redis Documentation](https://redis.io/documentation)
- [Prometheus Documentation](https://prometheus.io/docs/)
- [Grafana Documentation](https://grafana.com/docs/)

## 🎯 Next Steps

1. **Quick Start**: Follow [QUICKSTART.md](./QUICKSTART.md)
2. **Full Setup**: Read [DOCKER_SETUP.md](./DOCKER_SETUP.md)
3. **Production**: Review [PRODUCTION_READINESS.md](./PRODUCTION_READINESS.md)
4. **Configuration**: Customize [authenc.toml](./authenc.toml)

---

**Status**: ✅ Production Ready

**Last Updated**: November 28, 2024

**Version**: 1.0
