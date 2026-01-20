# Authenc Production Readiness Guide

## Overview

This document provides a comprehensive checklist and optimization guide for deploying Authenc in production environments.

## Pre-Deployment Checklist

### Security

- [ ] **Secrets Management**
  - [ ] Generate strong JWT_SECRET (min 32 characters, random)
  - [ ] Generate Ed25519 signing keys via `cargo run --bin generate-signing-keys`
  - [ ] Store secrets in secure vault (not in .env file)
  - [ ] Rotate secrets regularly (quarterly minimum)

- [ ] **Database Security**
  - [ ] Change default PostgreSQL password
  - [ ] Use strong password (min 16 characters, mixed case, numbers, symbols)
  - [ ] Enable PostgreSQL SSL connections
  - [ ] Restrict database access to authenc service only
  - [ ] Enable PostgreSQL audit logging

- [ ] **Redis Security**
  - [ ] Set strong Redis password
  - [ ] Disable dangerous commands (FLUSHDB, FLUSHALL, KEYS)
  - [ ] Enable Redis ACL (Redis 6+)
  - [ ] Use Redis SSL/TLS for remote connections

- [ ] **TLS/HTTPS**
  - [ ] Obtain valid SSL certificate (Let's Encrypt or CA)
  - [ ] Set TLS_ENABLED=true
  - [ ] Configure TLS_CERT_PATH and TLS_KEY_PATH
  - [ ] Use TLS 1.2 minimum
  - [ ] Configure HSTS headers

- [ ] **CORS Configuration**
  - [ ] Set CORS_ALLOWED_ORIGINS to specific domains (not "*")
  - [ ] Remove development origins from production
  - [ ] Enable CORS preflight caching

- [ ] **Authentication & Authorization**
  - [ ] Enable MFA for all admin accounts
  - [ ] Configure RBAC policies
  - [ ] Set up audit logging
  - [ ] Enable rate limiting

### Infrastructure

- [ ] **Container Security**
  - [ ] Use non-root user (UID 1000)
  - [ ] Enable read-only root filesystem
  - [ ] Set resource limits (CPU, memory)
  - [ ] Enable security options (no-new-privileges)
  - [ ] Use minimal base images

- [ ] **Networking**
  - [ ] Use private Docker network
  - [ ] Restrict port exposure
  - [ ] Configure firewall rules
  - [ ] Use network policies if on Kubernetes

- [ ] **Storage**
  - [ ] Use named volumes for persistence
  - [ ] Configure backup strategy
  - [ ] Test restore procedures
  - [ ] Set up log rotation

- [ ] **Monitoring & Logging**
  - [ ] Enable structured logging
  - [ ] Configure log aggregation
  - [ ] Set up Prometheus metrics
  - [ ] Configure Grafana dashboards
  - [ ] Set up alerting

### Application Configuration

- [ ] **Environment**
  - [ ] Set AUTHENC_ENV=production
  - [ ] Set RUST_LOG=warn (not debug)
  - [ ] Disable API documentation (ENABLE_API_DOCS=false)
  - [ ] Enable compression (ENABLE_COMPRESSION=true)
  - [ ] Enable caching (ENABLE_CACHING=true)

- [ ] **Features**
  - [ ] Enable email verification
  - [ ] Enable MFA
  - [ ] Configure password reset
  - [ ] Enable rate limiting
  - [ ] Enable input validation

- [ ] **Database**
  - [ ] Increase connection pool (max_connections=50+)
  - [ ] Configure connection timeout
  - [ ] Enable connection pooling
  - [ ] Set up replication (optional)

- [ ] **CAPTCHA**
  - [ ] Set appropriate difficulty level
  - [ ] Enable behavioral analysis
  - [ ] Configure rate limiting
  - [ ] Enable accessibility features

## Deployment Steps

### 1. Prepare Environment

```bash
# Copy and configure environment file
cp .env.example .env
nano .env

# Verify all required variables are set
grep -E "^[A-Z_]+=" .env | wc -l
```

### 2. Generate Signing Keys

```bash
# Generate cryptographic keys
cargo run --bin generate-signing-keys

# Add to .env
echo "ED25519_PRIVATE_KEY_BASE64=..." >> .env
```

### 3. Build Docker Image

```bash
# Build production image
docker build -t authenc:latest .

# Tag for registry
docker tag authenc:latest registry.example.com/authenc:latest

# Push to registry
docker push registry.example.com/authenc:latest
```

### 4. Deploy Services

```bash
# Production deployment
docker-compose -f docker-compose.yml \
               -f docker-compose.prod.yml \
               up -d

# Verify services
docker-compose ps

# Check logs
docker-compose logs -f authenc
```

### 5. Verify Deployment

```bash
# Health check
curl -s http://localhost:8088/health | jq .

# API test
curl -X GET http://localhost:8088/api/v1/health

# Metrics
curl -s http://localhost:9090/metrics | head -20

# Database connectivity
docker-compose exec postgres psql -U postgres -d authenc -c "SELECT version();"
```

## Performance Optimization

### Database Optimization

```sql
-- Create indexes for common queries
CREATE INDEX idx_users_email_status ON authenc.users(email, status);
CREATE INDEX idx_sessions_user_expires ON authenc.sessions(user_id, expires_at);
CREATE INDEX idx_audit_logs_created_action ON audit.logs(created_at, action);

-- Analyze tables
ANALYZE authenc.users;
ANALYZE authenc.sessions;
ANALYZE audit.logs;

-- Configure autovacuum
ALTER TABLE authenc.users SET (autovacuum_vacuum_scale_factor = 0.01);
ALTER TABLE authenc.sessions SET (autovacuum_vacuum_scale_factor = 0.01);
```

### Redis Optimization

```bash
# Configure Redis for production
redis-cli CONFIG SET maxmemory 512mb
redis-cli CONFIG SET maxmemory-policy allkeys-lru
redis-cli CONFIG SET tcp-backlog 511
redis-cli CONFIG SET timeout 0
redis-cli CONFIG SET tcp-keepalive 300

# Enable persistence
redis-cli CONFIG SET appendonly yes
redis-cli CONFIG SET appendfsync everysec
```

### Application Optimization

```bash
# Environment variables for optimization
ENABLE_COMPRESSION=true
ENABLE_CACHING=true
RATE_LIMIT_REQUESTS_PER_MINUTE=200
CAPTCHA_DEFAULT_DIFFICULTY=3
```

### Container Resource Limits

```yaml
# In docker-compose.prod.yml
authenc:
  deploy:
    resources:
      limits:
        cpus: '2'
        memory: 2G
      reservations:
        cpus: '1'
        memory: 1G
```

## Monitoring & Alerting

### Key Metrics to Monitor

- **Application**
  - Request latency (p50, p95, p99)
  - Error rate
  - Authentication success rate
  - CAPTCHA solve rate

- **Database**
  - Connection pool usage
  - Query latency
  - Transaction rate
  - Replication lag

- **Infrastructure**
  - CPU usage
  - Memory usage
  - Disk I/O
  - Network I/O

### Alert Thresholds

```yaml
# Prometheus alert rules
- alert: HighErrorRate
  expr: rate(http_requests_total{status=~"5.."}[5m]) > 0.05
  for: 5m

- alert: HighLatency
  expr: histogram_quantile(0.95, http_request_duration_seconds) > 1
  for: 5m

- alert: DatabaseConnectionPoolExhausted
  expr: pg_stat_activity_count > 45
  for: 5m

- alert: RedisMemoryHigh
  expr: redis_memory_used_bytes / redis_memory_max_bytes > 0.9
  for: 5m
```

## Backup & Disaster Recovery

### Database Backup

```bash
# Daily backup
docker-compose exec postgres pg_dump -U postgres authenc | \
  gzip > backup-$(date +%Y%m%d).sql.gz

# Automated backup (cron)
0 2 * * * docker-compose -f /path/to/docker-compose.yml exec -T postgres \
  pg_dump -U postgres authenc | gzip > /backups/authenc-$(date +\%Y\%m\%d).sql.gz
```

### Restore Procedure

```bash
# Restore from backup
gunzip < backup-20240101.sql.gz | \
  docker-compose exec -T postgres psql -U postgres authenc

# Verify restore
docker-compose exec postgres psql -U postgres -d authenc -c "SELECT COUNT(*) FROM authenc.users;"
```

### Redis Backup

```bash
# Redis backup
docker-compose exec redis redis-cli BGSAVE

# Copy RDB file
docker cp authenc-redis:/data/dump.rdb ./redis-backup-$(date +%Y%m%d).rdb
```

## Scaling Strategies

### Horizontal Scaling

```yaml
# Multiple Authenc instances with load balancer
services:
  authenc-1:
    # ... same config
  authenc-2:
    # ... same config
  authenc-3:
    # ... same config

  nginx:
    image: nginx:latest
    ports:
      - "80:80"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
```

### Vertical Scaling

```bash
# Increase resources in docker-compose.prod.yml
deploy:
  resources:
    limits:
      cpus: '4'
      memory: 4G
```

### Database Scaling

```bash
# PostgreSQL replication
# Primary-Replica setup for read scaling
# Configure streaming replication
```

## Security Hardening

### Network Security

```bash
# Firewall rules
ufw allow 8088/tcp    # HTTP API
ufw allow 9088/tcp    # gRPC
ufw allow 9090/tcp    # Metrics (internal only)
ufw allow 3000/tcp    # Grafana (internal only)
```

### Container Security

```bash
# Run security scan
docker run --rm -v /var/run/docker.sock:/var/run/docker.sock \
  aquasec/trivy image authenc:latest

# Sign images
docker trust sign authenc:latest
```

### Secrets Management

```bash
# Use Docker secrets (Swarm) or Kubernetes secrets
docker secret create jwt_secret -
docker secret create db_password -

# Or use external vault
export VAULT_ADDR=https://vault.example.com
vault kv put secret/authenc jwt_secret=...
```

## Maintenance

### Regular Tasks

- **Daily**
  - Monitor error rates and latency
  - Check disk space
  - Review security logs

- **Weekly**
  - Review audit logs
  - Check backup integrity
  - Update dependencies

- **Monthly**
  - Rotate secrets
  - Review security policies
  - Capacity planning

- **Quarterly**
  - Full security audit
  - Disaster recovery drill
  - Performance review

### Troubleshooting

```bash
# Check service health
docker-compose ps

# View logs
docker-compose logs -f authenc

# Database connectivity
docker-compose exec postgres psql -U postgres -d authenc -c "SELECT 1;"

# Redis connectivity
docker-compose exec redis redis-cli PING

# Network connectivity
docker-compose exec authenc curl -s http://postgres:5432 || echo "DB unreachable"
```

## Compliance & Auditing

- [ ] Enable audit logging
- [ ] Configure log retention (90+ days)
- [ ] Set up log aggregation
- [ ] Enable encryption at rest
- [ ] Enable encryption in transit
- [ ] Document security policies
- [ ] Conduct regular security reviews
- [ ] Maintain compliance documentation

## Support & Escalation

For production issues:

1. Check logs: `docker-compose logs -f authenc`
2. Verify health: `curl http://localhost:8088/health`
3. Check metrics: `http://localhost:9091` (Prometheus)
4. Review Grafana: `http://localhost:3000`
5. Contact support with logs and metrics

## Additional Resources

- [Authenc Documentation](../docs/)
- [Docker Best Practices](https://docs.docker.com/develop/dev-best-practices/)
- [PostgreSQL Security](https://www.postgresql.org/docs/current/sql-syntax.html)
- [Redis Security](https://redis.io/topics/security)
- [OWASP Security Guidelines](https://owasp.org/)
