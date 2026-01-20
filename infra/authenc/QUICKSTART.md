# Authenc Quick Start Guide

## 5-Minute Setup

### 1. Clone/Navigate to Authenc Directory
```bash
cd /srv/proyek/simpelv2/infra/authenc
```

### 2. Create Environment File
```bash
cp .env.example .env
# Edit with your settings (at minimum, change passwords)
nano .env
```

### 3. Start Services
```bash
# Development (fastest)
docker-compose up -d

# Production (with optimizations)
docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d

# Or use the helper script
./docker-start.sh dev    # or prod
```

### 4. Verify Services
```bash
# Check status
docker-compose ps

# Test API
curl http://localhost:8088/health

# View logs
docker-compose logs -f authenc
```

### 5. Access Services

| Service | URL | Credentials |
|---------|-----|-------------|
| API | http://localhost:8088 | - |
| gRPC | localhost:9088 | - |
| Metrics | http://localhost:9090/metrics | - |
| Prometheus | http://localhost:9091 | - |
| Grafana | http://localhost:3000 | admin / (from .env) |

## Common Commands

### View Logs
```bash
docker-compose logs -f authenc          # Follow authenc logs
docker-compose logs --tail=100 authenc  # Last 100 lines
docker-compose logs authenc > logs.txt  # Save to file
```

### Database Access
```bash
docker-compose exec postgres psql -U postgres -d authenc
```

### Redis Access
```bash
docker-compose exec redis redis-cli -a ${REDIS_PASSWORD}
```

### Restart Services
```bash
docker-compose restart authenc
docker-compose restart postgres
docker-compose restart redis
```

### Stop Services
```bash
docker-compose stop              # Stop all
docker-compose stop authenc      # Stop specific service
docker-compose down              # Stop and remove containers
docker-compose down -v           # Stop and remove everything
```

## Troubleshooting

### Services Won't Start
```bash
# Check logs
docker-compose logs authenc

# Verify .env file
cat .env | grep -E "^[A-Z_]+" | head -10

# Check port availability
netstat -tlnp | grep 8088
```

### Database Connection Failed
```bash
# Check PostgreSQL
docker-compose ps postgres
docker-compose logs postgres

# Test connection
docker-compose exec postgres psql -U postgres -c "SELECT 1;"
```

### High Memory Usage
```bash
# Check container stats
docker stats

# Reduce Redis memory in docker-compose.prod.yml
# Increase database connection pool if needed
```

## Next Steps

1. **Configure for Production**
   - Read: [PRODUCTION_READINESS.md](./PRODUCTION_READINESS.md)
   - Update security settings
   - Generate signing keys

2. **Setup Monitoring**
   - Access Grafana: http://localhost:3000
   - Configure dashboards
   - Set up alerts

3. **Integrate with Other Services**
   - Secreton (secrets management)
   - Authenc (authentication)
   - Other microservices

4. **Deploy to Production**
   - Use docker-compose.prod.yml
   - Set up CI/CD pipeline
   - Configure backups

## Documentation

- [Full Setup Guide](./DOCKER_SETUP.md)
- [Production Readiness](./PRODUCTION_READINESS.md)
- [Configuration Reference](./authenc.toml)
- [API Documentation](../docs/API.md)

## Support

For issues:
1. Check logs: `docker-compose logs -f`
2. Review configuration: `cat .env`
3. Verify services: `docker-compose ps`
4. Test health: `curl http://localhost:8088/health`
