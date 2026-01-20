# Authenc Docker Testing & Optimization Suite

## Overview
Comprehensive testing suite for authenc IAM service running in Docker. Tests all core features and validates production readiness.

## Prerequisites
- Docker and Docker Compose installed
- Services running: PostgreSQL, Redis, Authenc, Prometheus, Grafana
- Environment file (.env) configured
- Network: authenc-network

## 1. Service Health Checks

### 1.1 API Health Endpoint
```bash
# Test basic health check
curl -s http://localhost:8088/health | jq .

# Expected response:
# {
#   "status": "healthy",
#   "timestamp": "2024-11-28T...",
#   "services": {
#     "database": "connected",
#     "redis": "connected",
#     "secreton": "optional"
#   }
# }
```

### 1.2 Database Connectivity
```bash
# Test database connection
docker exec authenc-postgres psql -U postgres -d authenc -c "SELECT 1;"

# Expected: Returns 1
```

### 1.3 Redis Connectivity
```bash
# Test Redis connection
docker exec authenc-redis redis-cli -a redis_password ping

# Expected: PONG
```

### 1.4 Metrics Endpoint
```bash
# Check Prometheus metrics
curl -s http://localhost:9090/metrics | head -20

# Should show Prometheus format metrics
```

## 2. Authentication Tests

### 2.1 User Registration
```bash
# Register a new user
curl -X POST http://localhost:8088/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "testuser@example.com",
    "password": "SecurePassword123!",
    "full_name": "Test User"
  }'

# Expected: 201 Created with user ID
```

### 2.2 User Login
```bash
# Login with credentials
curl -X POST http://localhost:8088/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "testuser@example.com",
    "password": "SecurePassword123!"
  }'

# Expected: 200 OK with JWT token
# Response:
# {
#   "access_token": "eyJ...",
#   "token_type": "Bearer",
#   "expires_in": 3600,
#   "refresh_token": "..."
# }
```

### 2.3 JWT Token Validation
```bash
# Extract token from login response
TOKEN="<access_token_from_login>"

# Validate token
curl -X GET http://localhost:8088/api/v1/auth/validate \
  -H "Authorization: Bearer $TOKEN"

# Expected: 200 OK with token claims
```

### 2.4 Token Refresh
```bash
# Refresh token
curl -X POST http://localhost:8088/api/v1/auth/refresh \
  -H "Content-Type: application/json" \
  -d '{
    "refresh_token": "<refresh_token>"
  }'

# Expected: 200 OK with new access token
```

## 3. Multi-Factor Authentication (MFA) Tests

### 3.1 Enable MFA
```bash
# Enable MFA for user
curl -X POST http://localhost:8088/api/v1/mfa/enable \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json"

# Expected: 200 OK with TOTP secret and QR code
```

### 3.2 Verify MFA Setup
```bash
# Verify TOTP code
curl -X POST http://localhost:8088/api/v1/mfa/verify \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "code": "123456"
  }'

# Expected: 200 OK
```

### 3.3 Login with MFA
```bash
# Login with MFA enabled
curl -X POST http://localhost:8088/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "testuser@example.com",
    "password": "SecurePassword123!",
    "mfa_code": "123456"
  }'

# Expected: 200 OK with token
```

## 4. User Management Tests

### 4.1 Get User Profile
```bash
# Get current user profile
curl -X GET http://localhost:8088/api/v1/users/me \
  -H "Authorization: Bearer $TOKEN"

# Expected: 200 OK with user details
```

### 4.2 Update User Profile
```bash
# Update user information
curl -X PUT http://localhost:8088/api/v1/users/me \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "full_name": "Updated Name",
    "phone": "+1234567890"
  }'

# Expected: 200 OK with updated user
```

### 4.3 Change Password
```bash
# Change user password
curl -X POST http://localhost:8088/api/v1/users/change-password \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "old_password": "SecurePassword123!",
    "new_password": "NewSecurePassword456!"
  }'

# Expected: 200 OK
```

### 4.4 List Users (Admin)
```bash
# List all users (requires admin role)
curl -X GET http://localhost:8088/admin/users \
  -H "Authorization: Bearer $ADMIN_TOKEN"

# Expected: 200 OK with user list
```

## 5. Role-Based Access Control (RBAC) Tests

### 5.1 Assign Role
```bash
# Assign role to user
curl -X POST http://localhost:8088/admin/users/{user_id}/roles \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "role": "editor"
  }'

# Expected: 200 OK
```

### 5.2 Check Permissions
```bash
# Check user permissions
curl -X GET http://localhost:8088/api/v1/users/me/permissions \
  -H "Authorization: Bearer $TOKEN"

# Expected: 200 OK with permission list
```

### 5.3 Unauthorized Access
```bash
# Try to access admin endpoint without admin role
curl -X GET http://localhost:8088/admin/users \
  -H "Authorization: Bearer $USER_TOKEN"

# Expected: 403 Forbidden
```

## 6. Session Management Tests

### 6.1 Create Session
```bash
# Login creates a session
curl -X POST http://localhost:8088/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "testuser@example.com",
    "password": "SecurePassword123!"
  }'

# Expected: Session created and stored in Redis
```

### 6.2 List Sessions
```bash
# List user sessions
curl -X GET http://localhost:8088/api/v1/sessions \
  -H "Authorization: Bearer $TOKEN"

# Expected: 200 OK with active sessions
```

### 6.3 Logout
```bash
# Logout (invalidate session)
curl -X POST http://localhost:8088/api/v1/auth/logout \
  -H "Authorization: Bearer $TOKEN"

# Expected: 200 OK, token invalidated
```

## 7. Rate Limiting Tests

### 7.1 Brute Force Protection
```bash
# Make multiple failed login attempts
for i in {1..10}; do
  curl -X POST http://localhost:8088/api/v1/auth/login \
    -H "Content-Type: application/json" \
    -d '{
      "email": "testuser@example.com",
      "password": "WrongPassword"
    }'
done

# Expected: After threshold, get 429 Too Many Requests
```

### 7.2 Rate Limit Headers
```bash
# Check rate limit headers
curl -i http://localhost:8088/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "testuser@example.com",
    "password": "SecurePassword123!"
  }' | grep -i "rate-limit"

# Expected: Rate-Limit-* headers in response
```

## 8. Configuration Management Tests

### 8.1 Get Configuration
```bash
# Get service configuration (admin only)
curl -X GET http://localhost:8088/admin/config \
  -H "Authorization: Bearer $ADMIN_TOKEN"

# Expected: 200 OK with current configuration
```

### 8.2 Update Configuration
```bash
# Update service configuration
curl -X PUT http://localhost:8088/admin/config \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "password_min_length": 14,
    "session_timeout": 7200
  }'

# Expected: 200 OK with updated config
```

## 9. Monitoring & Observability Tests

### 9.1 Prometheus Metrics
```bash
# Query Prometheus for authenc metrics
curl -s 'http://localhost:9091/api/v1/query?query=authenc_requests_total' | jq .

# Expected: Metrics data
```

### 9.2 Grafana Dashboard
```bash
# Access Grafana
# URL: http://localhost:3000
# Default credentials: admin/admin
# Check dashboards for:
# - Request latency
# - Error rates
# - Database query times
# - Cache hit rates
```

### 9.3 Structured Logs
```bash
# Check structured logs
docker logs authenc-service | grep -i "request\|error\|auth" | head -20

# Expected: JSON-formatted logs with request details
```

## 10. Error Handling Tests

### 10.1 Invalid Input
```bash
# Send invalid email format
curl -X POST http://localhost:8088/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "invalid-email",
    "password": "SecurePassword123!"
  }'

# Expected: 400 Bad Request with validation error
```

### 10.2 Missing Required Fields
```bash
# Missing password field
curl -X POST http://localhost:8088/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "testuser@example.com"
  }'

# Expected: 400 Bad Request
```

### 10.3 Duplicate User
```bash
# Try to register with existing email
curl -X POST http://localhost:8088/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "testuser@example.com",
    "password": "SecurePassword123!"
  }'

# Expected: 409 Conflict
```

## 11. Performance Tests

### 11.1 Response Time
```bash
# Measure response time for login
time curl -X POST http://localhost:8088/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "testuser@example.com",
    "password": "SecurePassword123!"
  }'

# Expected: <100ms
```

### 11.2 Concurrent Requests
```bash
# Test with 10 concurrent requests
ab -n 100 -c 10 -H "Authorization: Bearer $TOKEN" \
  http://localhost:8088/api/v1/users/me

# Expected: All requests succeed, <100ms average
```

### 11.3 Database Query Performance
```bash
# Check slow query logs
docker exec authenc-postgres psql -U postgres -d authenc \
  -c "SELECT query, calls, mean_time FROM pg_stat_statements ORDER BY mean_time DESC LIMIT 10;"

# Expected: All queries <10ms
```

## 12. Security Tests

### 12.1 SQL Injection
```bash
# Try SQL injection in login
curl -X POST http://localhost:8088/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "admin' OR '1'='1",
    "password": "anything"
  }'

# Expected: 400 Bad Request or 401 Unauthorized (not SQL error)
```

### 12.2 XSS Prevention
```bash
# Try XSS in user profile
curl -X PUT http://localhost:8088/api/v1/users/me \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "full_name": "<script>alert(1)</script>"
  }'

# Expected: Script tags escaped or rejected
```

### 12.3 CORS Headers
```bash
# Check CORS headers
curl -i -X OPTIONS http://localhost:8088/api/v1/auth/login \
  -H "Origin: http://example.com"

# Expected: Appropriate CORS headers
```

## 13. Integration Tests

### 13.1 Full Authentication Flow
```bash
# 1. Register
REGISTER=$(curl -s -X POST http://localhost:8088/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "flowtest@example.com",
    "password": "SecurePassword123!"
  }')

# 2. Login
LOGIN=$(curl -s -X POST http://localhost:8088/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "flowtest@example.com",
    "password": "SecurePassword123!"
  }')

TOKEN=$(echo $LOGIN | jq -r '.access_token')

# 3. Get profile
curl -s -X GET http://localhost:8088/api/v1/users/me \
  -H "Authorization: Bearer $TOKEN" | jq .

# 4. Logout
curl -s -X POST http://localhost:8088/api/v1/auth/logout \
  -H "Authorization: Bearer $TOKEN"

# Expected: All steps succeed
```

## 14. Docker Compose Verification

### 14.1 Service Status
```bash
docker-compose ps

# Expected: All services "Up"
```

### 14.2 Network Connectivity
```bash
docker network inspect authenc-network

# Expected: All containers connected
```

### 14.3 Volume Mounts
```bash
docker inspect authenc-service | grep -A 10 "Mounts"

# Expected: Proper volume mounts
```

## 15. Production Readiness Checklist

- [ ] All services start without errors
- [ ] Health checks pass
- [ ] API responds to requests (<100ms)
- [ ] Database connected and queries fast (<10ms)
- [ ] Redis caching working
- [ ] Authentication flows work
- [ ] MFA enabled and working
- [ ] RBAC enforced
- [ ] Rate limiting active
- [ ] Metrics available
- [ ] Logs structured and useful
- [ ] No SQL injection vulnerabilities
- [ ] CORS properly configured
- [ ] TLS ready (optional)
- [ ] Secrets not in logs
- [ ] Error handling graceful
- [ ] Performance targets met
- [ ] Load testing passed

## Test Execution Script

```bash
#!/bin/bash
# Run all tests

echo "=== Health Checks ==="
curl -s http://localhost:8088/health | jq .

echo -e "\n=== Authentication Tests ==="
# Register
curl -s -X POST http://localhost:8088/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"Test123!"}' | jq .

# Login
TOKEN=$(curl -s -X POST http://localhost:8088/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"Test123!"}' | jq -r '.access_token')

echo -e "\n=== User Profile ==="
curl -s -X GET http://localhost:8088/api/v1/users/me \
  -H "Authorization: Bearer $TOKEN" | jq .

echo -e "\n=== Metrics ==="
curl -s http://localhost:9090/metrics | head -20

echo -e "\n✅ All tests completed"
```

## Success Criteria

✅ All services running and healthy
✅ API responds to all requests
✅ Authentication working
✅ MFA functional
✅ RBAC enforced
✅ Rate limiting active
✅ Metrics available
✅ Logs structured
✅ Performance targets met
✅ No security vulnerabilities
✅ Production ready

---

**Status**: Ready for testing
**Last Updated**: November 28, 2024
