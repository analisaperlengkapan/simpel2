# Authenc Testing & Optimization Steps

## Status Update

### Current Progress
- ✅ Docker environment setup complete
- ✅ PostgreSQL running on port 5433
- ✅ Redis running on port 6379
- ⏳ Authenc Docker image build in progress (Rust compilation)
- ✅ Comprehensive test suite created
- ✅ Production optimization guide created
- ✅ Deployment documentation created

### Build Status
- Rust binary compilation in progress
- Multi-stage Docker build will create optimized image
- Estimated completion: 10-30 minutes depending on system

## Next Steps (After Build Completes)

### Step 1: Verify Docker Image
```bash
cd /srv/proyek/simpelv2/infra/authenc

# Check if image was built
docker images | grep authenc

# Expected output:
# authenc  latest  <image_id>  <size>  <created_time>
```

### Step 2: Start All Services
```bash
# Using the wrapper script
./dc.sh up -d

# Or using docker compose (CLI plugin)
docker compose up -d

# Verify all services are running
./dc.sh ps

# Expected output:
# NAME                COMMAND                  STATE           PORTS
# authenc-postgres    docker-entrypoint.s...   Up              0.0.0.0:5433->5432/tcp
# authenc-redis       docker-entrypoint.s...   Up              0.0.0.0:6379->6379/tcp
# authenc-service     ./authenc                Up              0.0.0.0:8088->8088/tcp
# authenc-prometheus  /bin/prometheus ...      Up              0.0.0.0:9091->9090/tcp
# authenc-grafana     /run.sh                  Up              0.0.0.0:3000->3000/tcp
```

### Step 3: Wait for Services to Be Healthy
```bash
# Check health of each service
./dc.sh logs authenc | grep -i "health\|ready\|listening"

# Test API health endpoint
curl -s http://localhost:8088/health | jq .

# Expected response:
# {
#   "status": "healthy",
#   "timestamp": "2024-11-28T...",
#   "services": {
#     "database": "connected",
#     "redis": "connected"
#   }
# }
```

### Step 4: Run Core Feature Tests

#### 4.1 Authentication Tests
```bash
# Register a user
curl -X POST http://localhost:8088/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "testuser@example.com",
    "password": "SecurePassword123!",
    "full_name": "Test User"
  }'

# Expected: 201 Created

# Login
TOKEN=$(curl -s -X POST http://localhost:8088/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "testuser@example.com",
    "password": "SecurePassword123!"
  }' | jq -r '.access_token')

echo "Token: $TOKEN"

# Validate token
curl -s -X GET http://localhost:8088/api/v1/auth/validate \
  -H "Authorization: Bearer $TOKEN" | jq .

# Expected: 200 OK with token claims
```

#### 4.2 User Management Tests
```bash
# Get user profile
curl -s -X GET http://localhost:8088/api/v1/users/me \
  -H "Authorization: Bearer $TOKEN" | jq .

# Update profile
curl -s -X PUT http://localhost:8088/api/v1/users/me \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "full_name": "Updated Name",
    "phone": "+1234567890"
  }' | jq .

# Expected: 200 OK
```

#### 4.3 MFA Tests
```bash
# Enable MFA
MFA_SETUP=$(curl -s -X POST http://localhost:8088/api/v1/mfa/enable \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json")

echo "$MFA_SETUP" | jq .

# Extract secret for TOTP
SECRET=$(echo "$MFA_SETUP" | jq -r '.secret')

# Generate TOTP code (requires oathtool or similar)
# For testing, use any 6-digit code

# Verify MFA setup
curl -s -X POST http://localhost:8088/api/v1/mfa/verify \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"code": "123456"}' | jq .

# Expected: 200 OK
```

### Step 5: Performance Testing

#### 5.1 Response Time Test
```bash
# Test API response time
time curl -s http://localhost:8088/health > /dev/null

# Expected: <100ms total time
```

#### 5.2 Concurrent Load Test
```bash
# Install Apache Bench if not available
sudo apt-get install -y apache2-utils

# Run load test
ab -n 1000 -c 10 http://localhost:8088/health

# Expected output should show:
# - Requests per second: >100
# - Mean time per request: <100ms
# - Failed requests: 0
```

#### 5.3 Database Query Performance
```bash
# Check database query performance
docker exec authenc-postgres psql -U postgres -d authenc \
  -c "SELECT query, calls, mean_time FROM pg_stat_statements ORDER BY mean_time DESC LIMIT 10;"

# Expected: All queries <10ms average
```

### Step 6: Monitoring & Metrics

#### 6.1 Check Prometheus Metrics
```bash
# Query Prometheus for authenc metrics
curl -s 'http://localhost:9091/api/v1/query?query=authenc_requests_total' | jq .

# Check request latency
curl -s 'http://localhost:9091/api/v1/query?query=authenc_request_duration_seconds' | jq .

# Check error rates
curl -s 'http://localhost:9091/api/v1/query?query=authenc_requests_total{status=~"5.."}' | jq .
```

#### 6.2 Access Grafana Dashboard
```
URL: http://localhost:3000
Username: admin
Password: admin

Dashboards to check:
- Request Latency (p50, p95, p99)
- Error Rates
- Database Query Performance
- Cache Hit Rates
- Active Sessions
- Failed Login Attempts
```

### Step 7: Security Testing

#### 7.1 SQL Injection Test
```bash
# Try SQL injection
curl -X POST http://localhost:8088/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "admin'\'' OR '\''1'\''='\''1",
    "password": "anything"
  }'

# Expected: 400 Bad Request or 401 Unauthorized (NOT a SQL error)
```

#### 7.2 Rate Limiting Test
```bash
# Make multiple requests to trigger rate limiting
for i in {1..150}; do
  curl -s http://localhost:8088/health > /dev/null
done

# Check for 429 Too Many Requests
curl -i http://localhost:8088/health | grep HTTP

# Expected: Eventually get 429 status
```

#### 7.3 CORS Test
```bash
# Check CORS headers
curl -i -X OPTIONS http://localhost:8088/api/v1/auth/login \
  -H "Origin: http://example.com" \
  -H "Access-Control-Request-Method: POST"

# Expected: Appropriate CORS headers in response
```

### Step 8: Integration Testing

#### 8.1 Full Authentication Flow
```bash
#!/bin/bash

# 1. Register
echo "1. Registering user..."
REGISTER=$(curl -s -X POST http://localhost:8088/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "integration-test@example.com",
    "password": "IntegrationTest123!",
    "full_name": "Integration Test User"
  }')

USER_ID=$(echo "$REGISTER" | jq -r '.id')
echo "Registered user: $USER_ID"

# 2. Login
echo "2. Logging in..."
LOGIN=$(curl -s -X POST http://localhost:8088/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "integration-test@example.com",
    "password": "IntegrationTest123!"
  }')

TOKEN=$(echo "$LOGIN" | jq -r '.access_token')
echo "Got token: ${TOKEN:0:20}..."

# 3. Get profile
echo "3. Getting user profile..."
PROFILE=$(curl -s -X GET http://localhost:8088/api/v1/users/me \
  -H "Authorization: Bearer $TOKEN")

echo "Profile: $(echo "$PROFILE" | jq '.email')"

# 4. Update profile
echo "4. Updating profile..."
UPDATE=$(curl -s -X PUT http://localhost:8088/api/v1/users/me \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "full_name": "Updated Integration Test User"
  }')

echo "Updated: $(echo "$UPDATE" | jq '.full_name')"

# 5. Logout
echo "5. Logging out..."
LOGOUT=$(curl -s -X POST http://localhost:8088/api/v1/auth/logout \
  -H "Authorization: Bearer $TOKEN")

echo "Logout status: $(echo "$LOGOUT" | jq '.status')"

echo "✅ Integration test completed successfully!"
```

### Step 9: Production Optimization

#### 9.1 Database Optimization
```bash
# Create indexes for better performance
docker exec authenc-postgres psql -U postgres -d authenc << EOF
CREATE INDEX IF NOT EXISTS idx_users_email ON authenc.users(email);
CREATE INDEX IF NOT EXISTS idx_users_active ON authenc.users(is_active);
CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON authenc.sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_user_id ON authenc.audit_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_timestamp ON authenc.audit_logs(created_at DESC);
EOF

echo "✅ Indexes created"
```

#### 9.2 Connection Pool Tuning
```bash
# Update connection pool settings via environment
export DB_MAX_CONNECTIONS=150
export DB_IDLE_TIMEOUT=300

# Restart service
./dc.sh restart authenc

echo "✅ Connection pool tuned"
```

#### 9.3 Cache Configuration
```bash
# Update cache settings via environment
export CACHE_TTL_SESSION=3600
export CACHE_TTL_CONFIG=300
export CACHE_TTL_PERMISSION=600

# Restart service
./dc.sh restart authenc

echo "✅ Cache configured"
```

### Step 10: Production Deployment Checklist

Before deploying to production, verify:

```bash
#!/bin/bash

echo "=== Production Readiness Checklist ==="

# 1. Health checks
echo "1. Checking health endpoints..."
curl -s http://localhost:8088/health | jq .status

# 2. Database connectivity
echo "2. Checking database..."
docker exec authenc-postgres pg_isready -U postgres

# 3. Redis connectivity
echo "3. Checking Redis..."
docker exec authenc-redis redis-cli -a redis_password ping

# 4. Metrics available
echo "4. Checking metrics..."
curl -s http://localhost:9090/metrics | head -5

# 5. No errors in logs
echo "5. Checking logs for errors..."
./dc.sh logs authenc | grep -i "error" | wc -l

# 6. Performance targets
echo "6. Running performance test..."
ab -n 100 -c 5 http://localhost:8088/health 2>&1 | grep "Requests per second"

# 7. Security headers
echo "7. Checking security headers..."
curl -i http://localhost:8088/health | grep -i "strict\|security\|x-"

echo "✅ Checklist complete!"
```

## Performance Targets

| Metric | Target | How to Test |
|--------|--------|------------|
| API Response Time (p95) | <100ms | `ab -n 1000 -c 10 http://localhost:8088/health` |
| Database Query Time (p95) | <10ms | Check `pg_stat_statements` |
| Cache Hit Rate | >95% | Monitor Prometheus metrics |
| Error Rate | <0.1% | Check `authenc_requests_total{status=~"5.."}` |
| Availability | >99.9% | Monitor uptime |
| Throughput | 1000+ req/sec | Load test with Apache Bench |

## Optimization Recommendations

### If Performance is Below Target:

1. **Slow API Response**
   - Check database query performance
   - Verify connection pool settings
   - Enable response caching
   - Check for N+1 queries

2. **Slow Database Queries**
   - Create missing indexes
   - Optimize query patterns
   - Use prepared statements (already done)
   - Consider query caching

3. **Low Cache Hit Rate**
   - Increase cache TTL
   - Verify Redis connectivity
   - Check cache invalidation logic
   - Monitor cache memory usage

4. **High Error Rate**
   - Check logs for errors
   - Verify database connectivity
   - Check input validation
   - Review security settings

## Troubleshooting

### Service Won't Start
```bash
# Check logs
./dc.sh logs authenc | tail -50

# Check port conflicts
sudo lsof -i :8088

# Check Docker daemon
docker ps
```

### Database Connection Issues
```bash
# Test database
docker exec authenc-postgres pg_isready -U postgres

# Check network
docker network inspect authenc-network

# Check credentials
echo $DATABASE_URL
```

### Performance Issues
```bash
# Check metrics
curl -s http://localhost:9090/metrics | grep authenc

# Check slow queries
docker exec authenc-postgres psql -U postgres -d authenc \
  -c "SELECT query, calls, mean_time FROM pg_stat_statements LIMIT 10;"

# Check resource usage
docker stats authenc-service
```

## Success Criteria

✅ All services start without errors
✅ API responds to requests (<100ms)
✅ Database connected and queries fast (<10ms)
✅ Redis caching working
✅ Authentication flows work
✅ MFA enabled and working
✅ RBAC enforced
✅ Rate limiting active
✅ Metrics available
✅ Logs structured and useful
✅ No security vulnerabilities
✅ Performance targets met

## Documentation References

- `DOCKER_TEST_SUITE.md` - Comprehensive test suite
- `PRODUCTION_OPTIMIZATION_GUIDE.md` - Optimization strategies
- `DOCKER_DEPLOYMENT_SUMMARY.md` - Deployment overview
- `HYBRID_CONFIG_GUIDE.md` - Configuration management
- `README.md` - General overview

---

**Status**: Ready for Testing
**Last Updated**: November 28, 2024
**Next Action**: Wait for Docker build to complete, then execute tests
