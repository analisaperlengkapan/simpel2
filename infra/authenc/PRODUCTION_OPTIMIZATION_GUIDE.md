# Authenc Production Optimization Guide

## Executive Summary

This guide provides comprehensive optimization strategies for deploying authenc IAM service to production. It covers performance tuning, security hardening, monitoring setup, and operational best practices.

## 1. Performance Optimization

### 1.1 Database Connection Pooling

**Current Configuration** (authenc.toml):
```toml
[database]
max_connections = 50
min_connections = 10
connection_timeout = 30
idle_timeout = 600
max_lifetime = 1800
```

**Optimization Recommendations**:
- **For 1000+ concurrent users**: Increase `max_connections` to 100-200
- **For high-traffic scenarios**: Reduce `idle_timeout` to 300 seconds
- **Connection timeout**: Keep at 30s for production

**Implementation**:
```bash
# Update via environment variable
export DB_MAX_CONNECTIONS=150
export DB_IDLE_TIMEOUT=300
```

### 1.2 Redis Caching Strategy

**Enable Multi-Layer Caching**:
```toml
[caching]
enable_session_cache = true
enable_config_cache = true
enable_permission_cache = true
cache_ttl_session = 3600
cache_ttl_config = 300
cache_ttl_permission = 600
```

**Cache Invalidation**:
- Session cache: Invalidate on logout
- Config cache: Invalidate on config update
- Permission cache: Invalidate on role change

### 1.3 Query Optimization

**Enable Query Caching**:
```sql
-- Create indexes for frequently queried columns
CREATE INDEX idx_users_email ON authenc.users(email);
CREATE INDEX idx_users_active ON authenc.users(is_active);
CREATE INDEX idx_sessions_user_id ON authenc.sessions(user_id);
CREATE INDEX idx_audit_logs_user_id ON authenc.audit_logs(user_id);
CREATE INDEX idx_audit_logs_timestamp ON authenc.audit_logs(created_at DESC);
```

**Use Prepared Statements**:
- All queries use prepared statements (already implemented)
- Reduces parsing overhead
- Prevents SQL injection

### 1.4 Response Compression

**Enable Gzip Compression**:
```toml
[server]
enable_compression = true
compression_level = 6  # 1-9, higher = better compression but slower
compression_min_size = 1024  # Only compress responses > 1KB
```

**Benefits**:
- Reduces bandwidth by 70-90%
- Minimal CPU overhead
- Transparent to clients

## 2. Security Hardening

### 2.1 TLS/SSL Configuration

**Enable TLS in Production**:
```toml
[server]
tls_enabled = true
tls_cert_path = "/etc/authenc/certs/cert.pem"
tls_key_path = "/etc/authenc/certs/key.pem"
```

**Generate Self-Signed Certificate** (for testing):
```bash
openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes
```

**Use Let's Encrypt** (for production):
```bash
certbot certonly --standalone -d authenc.example.com
```

### 2.2 JWT Security

**Current Configuration**:
```toml
[security]
jwt_secret = "change_me_in_production"
jwt_expiry = 3600  # 1 hour
```

**Production Recommendations**:
- **JWT Secret**: Use strong 256+ character secret from Secreton
- **Expiry**: 1 hour for access tokens
- **Refresh Token**: 7 days (configurable)
- **Algorithm**: RS256 (RSA) recommended for multi-service environments

**Rotate JWT Secret**:
```bash
# Generate new secret
openssl rand -base64 32

# Update in Secreton
curl -X POST http://secreton:8200/v1/secret/authenc/jwt_secret \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"value":"new_secret"}'

# Restart authenc service
docker restart authenc-service
```

### 2.3 Rate Limiting

**Configure Adaptive Rate Limiting**:
```toml
[rate_limit]
requests_per_minute = 100
burst_size = 20
enable_adaptive = true

[adaptive_rate_limit]
normal_threshold = 50
degraded_threshold = 80
critical_threshold = 95
```

**Brute Force Protection**:
```toml
brute_force_max_attempts = 5
brute_force_window_seconds = 300  # 5 minutes
```

### 2.4 Input Validation

**Enable Strict Validation**:
```toml
[security]
enable_input_validation = true
max_request_size = 1048576  # 1MB
max_password_length = 128
min_password_length = 12
```

**Validation Rules**:
- Email: RFC 5322 compliant
- Password: Complexity requirements enforced
- Username: Alphanumeric + underscore only
- Phone: E.164 format

## 3. Monitoring & Observability

### 3.1 Prometheus Metrics

**Key Metrics to Monitor**:
```
authenc_requests_total{method,endpoint,status}
authenc_request_duration_seconds{method,endpoint}
authenc_database_query_duration_seconds
authenc_cache_hits_total
authenc_cache_misses_total
authenc_active_sessions
authenc_failed_logins_total
authenc_mfa_verifications_total
```

**Prometheus Configuration**:
```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'authenc'
    static_configs:
      - targets: ['localhost:9090']
    metrics_path: '/metrics'
```

### 3.2 Grafana Dashboards

**Create Dashboard for**:
- Request latency (p50, p95, p99)
- Error rates by endpoint
- Database query performance
- Cache hit rates
- Active sessions
- Failed login attempts
- MFA usage

**Alert Rules**:
```yaml
- alert: HighErrorRate
  expr: rate(authenc_requests_total{status=~"5.."}[5m]) > 0.05
  for: 5m
  annotations:
    summary: "High error rate detected"

- alert: SlowDatabaseQueries
  expr: authenc_database_query_duration_seconds > 0.1
  for: 5m
  annotations:
    summary: "Database queries are slow"

- alert: HighFailedLogins
  expr: rate(authenc_failed_logins_total[5m]) > 10
  for: 5m
  annotations:
    summary: "High number of failed login attempts"
```

### 3.3 Structured Logging

**Log Format**:
```json
{
  "timestamp": "2024-11-28T13:00:00Z",
  "level": "INFO",
  "service": "authenc",
  "request_id": "uuid",
  "user_id": "user_123",
  "action": "login",
  "status": "success",
  "duration_ms": 45,
  "ip_address": "192.168.1.1",
  "user_agent": "Mozilla/5.0..."
}
```

**Log Levels**:
- `ERROR`: Critical issues requiring immediate attention
- `WARN`: Potential issues to investigate
- `INFO`: Important business events
- `DEBUG`: Detailed diagnostic information
- `TRACE`: Very detailed debugging

**Production Log Level**: `WARN` (reduces I/O overhead)

### 3.4 Audit Logging

**Audit Events**:
- User login/logout
- Password changes
- MFA setup/disable
- Role assignments
- Configuration changes
- Failed authentication attempts
- Permission checks

**Retention Policy**:
- Keep audit logs for 90 days minimum
- Archive to cold storage after 30 days
- Implement tamper-proof logging

## 4. Docker Optimization

### 4.1 Image Size Optimization

**Current Multi-Stage Build**:
- Builder stage: ~2GB (includes Rust toolchain)
- Runtime stage: ~200MB (minimal dependencies)

**Further Optimization**:
```dockerfile
# Use distroless image for even smaller size
FROM gcr.io/distroless/debian11-nonroot

# Copy only necessary files
COPY --from=builder /app/target/release/authenc /app/authenc
COPY authenc.toml /app/authenc.toml

# Distroless images don't have shell, use direct exec
ENTRYPOINT ["/app/authenc"]
```

### 4.2 Container Resource Limits

**Set Resource Limits** (docker-compose.prod.yml):
```yaml
services:
  authenc:
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 1G
        reservations:
          cpus: '1'
          memory: 512M
```

**Recommended Limits**:
- CPU: 1-2 cores per instance
- Memory: 512MB-1GB per instance
- Adjust based on load testing

### 4.3 Health Checks

**Implement Comprehensive Health Checks**:
```yaml
healthcheck:
  test: ["CMD", "curl", "-f", "http://localhost:8088/health"]
  interval: 30s
  timeout: 10s
  retries: 3
  start_period: 40s
```

**Health Check Endpoint Response**:
```json
{
  "status": "healthy",
  "timestamp": "2024-11-28T13:00:00Z",
  "services": {
    "database": "connected",
    "redis": "connected",
    "secreton": "optional"
  },
  "version": "1.0.0"
}
```

## 5. Deployment Strategy

### 5.1 Blue-Green Deployment

**Process**:
1. Deploy new version to "green" environment
2. Run smoke tests
3. Switch traffic from "blue" to "green"
4. Keep "blue" as rollback option

**Implementation**:
```bash
# Deploy green environment
docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d authenc-green

# Run smoke tests
./test-suite.sh http://localhost:8089

# Switch traffic (via load balancer)
# Update load balancer to point to green

# Keep blue for rollback
# If issues, switch back to blue
```

### 5.2 Canary Deployment

**Process**:
1. Deploy new version to 10% of traffic
2. Monitor metrics for issues
3. Gradually increase to 100%
4. Rollback if issues detected

**Implementation**:
```bash
# Deploy canary (10% traffic)
docker-compose up -d authenc-canary

# Monitor for 30 minutes
watch -n 5 'curl -s http://localhost:8088/metrics | grep authenc_error'

# If healthy, increase to 50%
# Then 100%
```

### 5.3 Rollback Strategy

**Quick Rollback**:
```bash
# Stop current version
docker-compose stop authenc

# Start previous version
docker-compose up -d authenc

# Verify health
curl http://localhost:8088/health
```

## 6. Scaling Strategies

### 6.1 Horizontal Scaling

**Load Balancer Configuration** (nginx example):
```nginx
upstream authenc_backend {
    server authenc-1:8088;
    server authenc-2:8088;
    server authenc-3:8088;
    keepalive 32;
}

server {
    listen 80;
    server_name authenc.example.com;

    location / {
        proxy_pass http://authenc_backend;
        proxy_http_version 1.1;
        proxy_set_header Connection "";
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    }
}
```

**Start Multiple Instances**:
```bash
for i in {1..3}; do
  docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d authenc-$i
done
```

### 6.2 Database Scaling

**Read Replicas**:
- Setup PostgreSQL streaming replication
- Route read-only queries to replicas
- Keep writes on primary

**Connection Pooling**:
- Use PgBouncer for connection pooling
- Reduces database connection overhead

### 6.3 Cache Scaling

**Redis Cluster**:
- Setup Redis cluster for high availability
- Automatic failover
- Horizontal scaling

## 7. Operational Best Practices

### 7.1 Backup & Recovery

**Daily Backups**:
```bash
# Backup database
docker exec authenc-postgres pg_dump -U postgres authenc > backup-$(date +%Y%m%d).sql

# Backup configuration
docker exec authenc-postgres pg_dump -U postgres authenc authenc.configuration > config-$(date +%Y%m%d).sql

# Store in S3
aws s3 cp backup-*.sql s3://authenc-backups/
```

**Recovery**:
```bash
# Restore database
docker exec -i authenc-postgres psql -U postgres authenc < backup-20241128.sql

# Verify
docker exec authenc-postgres psql -U postgres -d authenc -c "SELECT COUNT(*) FROM authenc.users;"
```

### 7.2 Maintenance Windows

**Schedule**:
- Weekly: Database maintenance (VACUUM, ANALYZE)
- Monthly: Security updates
- Quarterly: Major version upgrades

**Maintenance Tasks**:
```bash
# Database maintenance
docker exec authenc-postgres psql -U postgres -d authenc -c "VACUUM ANALYZE;"

# Clear old audit logs
docker exec authenc-postgres psql -U postgres -d authenc \
  -c "DELETE FROM authenc.audit_logs WHERE created_at < NOW() - INTERVAL '90 days';"

# Update dependencies
cargo update
```

### 7.3 Incident Response

**Monitoring Alerts**:
- High error rate (>5%)
- Slow database queries (>100ms)
- High failed login attempts (>10/min)
- Memory usage >80%
- Disk usage >80%

**Response Procedures**:
1. Alert triggered
2. Investigate logs and metrics
3. Determine root cause
4. Apply fix or rollback
5. Verify recovery
6. Post-incident review

## 8. Performance Targets

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| API Response Time (p95) | <100ms | TBD | ⏳ |
| Database Query Time (p95) | <10ms | TBD | ⏳ |
| Cache Hit Rate | >95% | TBD | ⏳ |
| Error Rate | <0.1% | TBD | ⏳ |
| Availability | >99.9% | TBD | ⏳ |
| Throughput | 1000+ req/sec | TBD | ⏳ |

## 9. Security Checklist

- [ ] TLS/SSL enabled
- [ ] JWT secrets strong and rotated
- [ ] Rate limiting configured
- [ ] Input validation enabled
- [ ] Audit logging enabled
- [ ] Secrets in Secreton (not in code)
- [ ] CORS properly configured
- [ ] RBAC enforced
- [ ] MFA available
- [ ] Brute force protection active
- [ ] SQL injection prevention (prepared statements)
- [ ] XSS prevention (input sanitization)
- [ ] CSRF tokens implemented
- [ ] Security headers configured
- [ ] Regular security audits scheduled

## 10. Deployment Checklist

- [ ] All tests passing
- [ ] Security scan completed
- [ ] Performance benchmarks met
- [ ] Monitoring configured
- [ ] Alerting configured
- [ ] Backup strategy verified
- [ ] Rollback plan documented
- [ ] Load balancer configured
- [ ] DNS configured
- [ ] SSL certificates valid
- [ ] Documentation updated
- [ ] Team trained
- [ ] Runbooks prepared
- [ ] Incident response plan ready

## Conclusion

Following these optimization guidelines will ensure authenc is production-ready, secure, performant, and maintainable. Regular monitoring, testing, and updates are essential for long-term success.

---

**Last Updated**: November 28, 2024
**Version**: 1.0
**Status**: Ready for Implementation
