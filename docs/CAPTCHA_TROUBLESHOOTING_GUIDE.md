# CAPTCHA System Troublesh

## Quick Reference

### Emergency Commands

```bash
# Check service status
docker-compose ps | grep captcha

# View recent logs
docker logs --tail=100 simpelv2-authenc | grep -i captcha

# Check health endpoints
curl -f http://localhost:8088/api/v1/captcha/health

# Emergency rate limiting
redis-cli set "emergency_rate_limit" "true" EX 3600

# Disable CAPTCHA temporarily
redis-cli set "captcha_disabled" "true" EX 1800
```

### Critical Metrics Dashboard

Access Grafana dashboard: http://captcha-grafana:3000/d/captcha-overview

## Common Issues and Solutions

### 1. CAPTCHA Not Loading

#### Symptoms
- CAPTCHA component doesn't appear on login page
- JavaScript errors in browser console
- Blank space where CAPTCHA should be

#### Diagnostic Steps

1. **Check Browser Console**:
   ```javascript
   // Open browser dev tools (F12)
   // Look for errors in Console tab
   ```

2. **Verify Component Integration**:
   ```bash
   # Check if CAPTCHA component is properly imported
   grep -r "shared_microfrontend::components::captcha" antarmuka/portal/src/
   ```

3. **Check Network Requests**:
   ```bash
   # Monitor network tab in browser dev tools
   # Look for failed API calls to /api/v1/captcha/
   ```

#### Solutions

1. **Component Import Issue**:
   ```rust
   // Ensure proper import in login.rs
   use shared_microfrontend::components::captcha::Captcha;
   ```

2. **API Endpoint Issue**:
   ```bash
   # Check if CAPTCHA endpoints are registered
   curl -v http://localhost:8088/api/v1/captcha/challenge
   ```

3. **CORS Issue**:
   ```toml
   # Check CORS configuration in authenc
   [cors]
   allowed_origins = ["http://localhost:3000", "https://portal.kejaksaan.go.id"]
   ```

### 2. High Failure Rate

#### Symptoms
- CAPTCHA failure rate > 70%
- Users complaining about difficulty
- Increased support tickets

#### Diagnostic Steps

1. **Check Current Difficulty**:
   ```sql
   SELECT
       AVG(difficulty_level) as avg_difficulty,
       COUNT(*) as total_challenges,
       COUNT(*) FILTER (WHERE solved = true) as solved_challenges
   FROM captcha_challenges
   WHERE created_at > NOW() - INTERVAL '1 hour';
   ```

2. **Analyze Failure Patterns**:
   ```sql
   SELECT
       challenge_type,
       difficulty_level,
       COUNT(*) as attempts,
       COUNT(*) FILTER (WHERE success = true) as successes,
       ROUND(COUNT(*) FILTER (WHERE success = true)::NUMERIC / COUNT(*)::NUMERIC * 100, 2) as success_rate
   FROM captcha_challenges c
   JOIN captcha_validation_attempts va ON c.id = va.challenge_id
   WHERE c.created_at > NOW() - INTERVAL '24 hours'
   GROUP BY challenge_type, difficulty_level
   ORDER BY success_rate ASC;
   ```

3. **Check Bot Detection**:
   ```sql
   SELECT
       classification,
       COUNT(*) as count,
       ROUND(COUNT(*)::NUMERIC / (SELECT COUNT(*) FROM captcha_behavioral_metrics WHERE created_at > NOW() - INTERVAL '1 hour')::NUMERIC * 100, 2) as percentage
   FROM captcha_behavioral_metrics
   WHERE created_at > NOW() - INTERVAL '1 hour'
   GROUP BY classification;
   ```

#### Solutions

1. **Reduce Difficulty Temporarily**:
   ```sql
   -- Reduce global difficulty
   UPDATE captcha_difficulty_adjustments
   SET difficulty_level = GREATEST(difficulty_level - 1, 1)
   WHERE active = true;

   -- Or add temporary adjustment
   INSERT INTO captcha_difficulty_adjustments (
       ip_pattern, difficulty_level, reason, active, expires_at
   ) VALUES (
       '0.0.0.0/0', 2, 'Temporary difficulty reduction', true, NOW() + INTERVAL '2 hours'
   );
   ```

2. **Adjust Challenge Types**:
   ```toml
   # Disable difficult challenge types temporarily
   [captcha.challenge_types]
   visual = true
   audio = true
   logical = false  # Disable logical challenges
   behavioral = true
   hybrid = false   # Disable hybrid challenges
   ```

3. **Review Bot Detection Accuracy**:
   ```bash
   # Check if legitimate users are being flagged as bots
   curl http://localhost:8088/api/v1/captcha/metrics | grep bot_detection_accuracy
   ```

### 3. Slow Response Times

#### Symptoms
- CAPTCHA takes > 2 seconds to load
- Validation responses are slow
- Users experiencing timeouts

#### Diagnostic Steps

1. **Check Service Performance**:
   ```bash
   # Monitor resource usage
   docker stats simpelv2-authenc

   # Check response times
   curl -w "@curl-format.txt" -o /dev/null -s http://localhost:8088/api/v1/captcha/challenge
   ```

2. **Database Performance**:
   ```sql
   -- Check active connections
   SELECT count(*) FROM pg_stat_activity WHERE state = 'active';

   -- Check slow queries
   SELECT query, mean_time, calls
   FROM pg_stat_statements
   WHERE query LIKE '%captcha%'
   ORDER BY mean_time DESC
   LIMIT 10;

   -- Check table sizes
   SELECT
       schemaname,
       tablename,
       pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) as size
   FROM pg_tables
   WHERE tablename LIKE 'captcha_%'
   ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;
   ```

3. **Cache Performance**:
   ```bash
   # Check Redis performance
   redis-cli info stats | grep -E "(keyspace_hits|keyspace_misses|used_memory)"

   # Check cache hit rate
   redis-cli info stats | awk -F: '/keyspace_hits/{hits=$2} /keyspace_misses/{misses=$2} END{print "Hit rate: " hits/(hits+misses)*100 "%"}'
   ```

#### Solutions

1. **Database Optimization**:
   ```sql
   -- Vacuum and analyze tables
   VACUUM ANALYZE captcha_challenges;
   VACUUM ANALYZE captcha_behavioral_metrics;
   VACUUM ANALYZE captcha_validation_attempts;

   -- Reindex if needed
   REINDEX TABLE captcha_challenges;

   -- Clean up old data
   SELECT cleanup_expired_captcha_challenges();
   ```

2. **Increase Cache TTL**:
   ```toml
   [captcha.performance]
   cache_ttl = 7200  # Increase from 3600 to 7200 seconds
   ```

3. **Scale Services**:
   ```bash
   # Scale authenc service
   docker-compose up -d --scale authenc=3

   # Add more database connections
   # Edit postgresql.conf: max_connections = 200
   ```

### 4. Bot Detection Issues

#### Symptoms
- High false positive rate (legitimate users flagged as bots)
- High false negative rate (bots passing through)
- Inconsistent bot detection accuracy

#### Diagnostic Steps

1. **Check Detection Accuracy**:
   ```sql
   SELECT
       DATE(created_at) as date,
       classification,
       COUNT(*) as count,
       AVG(risk_score) as avg_risk_score
   FROM captcha_behavioral_metrics
   WHERE created_at > NOW() - INTERVAL '7 days'
   GROUP BY DATE(created_at), classification
   ORDER BY date DESC, classification;
   ```

2. **Analyze Behavioral Patterns**:
   ```sql
   -- Check mouse movement patterns
   SELECT
       classification,
       AVG((mouse_movements->>'total_distance')::NUMERIC) as avg_distance,
       AVG((mouse_movements->>'velocity_variance')::NUMERIC) as avg_velocity_variance
   FROM captcha_behavioral_metrics
   WHERE mouse_movements IS NOT NULL
   GROUP BY classification;

   -- Check keystroke patterns
   SELECT
       classification,
       AVG((keystroke_dynamics->>'typing_speed')::NUMERIC) as avg_typing_speed,
       AVG((keystroke_dynamics->>'rhythm_variance')::NUMERIC) as avg_rhythm_variance
   FROM captcha_behavioral_metrics
   WHERE keystroke_dynamics IS NOT NULL
   GROUP BY classification;
   ```

3. **Review Risk Scoring**:
   ```bash
   # Check risk score distribution
   curl http://localhost:8088/api/v1/captcha/analytics/risk-distribution
   ```

#### Solutions

1. **Adjust Risk Thresholds**:
   ```toml
   [captcha.behavioral_analysis]
   # Make thresholds more lenient
   low_risk_threshold = 0.4    # Increase from 0.3
   medium_risk_threshold = 0.7  # Increase from 0.6
   high_risk_threshold = 0.9    # Increase from 0.8
   ```

2. **Retrain Models**:
   ```bash
   # Trigger model retraining with recent data
   curl -X POST http://localhost:8088/api/v1/captcha/admin/retrain-models
   ```

3. **Disable Problematic Features**:
   ```toml
   [captcha.behavioral_analysis]
   mouse_tracking = true
   keystroke_analysis = true
   timing_analysis = true
   browser_fingerprinting = false  # Disable if causing issues
   session_analysis = true
   ```

### 5. Integration Issues

#### Symptoms
- Secreton connection errors
- Authenc authentication failures
- Database connection issues

#### Diagnostic Steps

1. **Check Service Connectivity**:
   ```bash
   # Test Secreton connection
   curl -f http://secreton:8200/v1/sys/health

   # Test Authenc connection
   curl -f http://authenc:8088/health

   # Test database connection
   pg_isready -h captcha-analytics-db -p 5432 -U captcha_user
   ```

2. **Check Authentication**:
   ```bash
   # Verify Secreton token
   curl -H "X-Vault-Token: $SECRETON_TOKEN" http://secreton:8200/v1/auth/token/lookup-self

   # Test Authenc client credentials
   curl -X POST http://authenc:8088/realms/simpel/protocol/openid-connect/token \
        -d "grant_type=client_credentials&client_id=portal&client_secret=$AUTHENC_CLIENT_SECRET"
   ```

3. **Review Configuration**:
   ```bash
   # Check environment variables
   env | grep -E "(SECRETON|AUTHENC|CAPTCHA_DB)"

   # Validate configuration files
   toml-validator config/captcha.production.toml
   ```

#### Solutions

1. **Secreton Issues**:
   ```bash
   # Renew Secreton token
   secreton auth -method=userpass username=captcha-service

   # Check mount path
   secreton mounts | grep captcha

   # Test encryption/decryption
   secreton write captcha/encrypt plaintext=$(echo "test" | base64)
   ```

2. **Authenc Issues**:
   ```bash
   # Refresh client credentials
   # Contact Authenc admin to regenerate client secret

   # Check realm configuration
   curl http://authenc:8088/realms/simpel/.well-known/openid_configuration
   ```

3. **Database Issues**:
   ```bash
   # Check database logs
   docker logs captcha-analytics-db

   # Test connection with different parameters
   psql "host=captcha-analytics-db port=5432 dbname=captcha_analytics user=captcha_user"

   # Check connection pool
   SELECT count(*) FROM pg_stat_activity WHERE datname = 'captcha_analytics';
   ```

### 6. Memory and Resource Issues

#### Symptoms
- Out of memory errors
- High CPU usage
- Container restarts

#### Diagnostic Steps

1. **Monitor Resource Usage**:
   ```bash
   # Check container resources
   docker stats --no-stream

   # Check system resources
   free -h
   df -h
   top -p $(pgrep -f authenc)
   ```

2. **Analyze Memory Usage**:
   ```bash
   # Check Java heap (if applicable)
   jstat -gc $(pgrep java)

   # Check Redis memory
   redis-cli info memory

   # Check PostgreSQL memory
   SELECT * FROM pg_stat_database WHERE datname = 'captcha_analytics';
   ```

#### Solutions

1. **Increase Container Limits**:
   ```yaml
   # In docker-compose.captcha.yml
   services:
     authenc:
       deploy:
         resources:
           limits:
             memory: 2G      # Increase from 1G
             cpus: '2.0'     # Increase from 1.0
   ```

2. **Optimize Memory Usage**:
   ```toml
   [captcha.performance]
   connection_pool_size = 5     # Reduce from 10
   max_concurrent_challenges = 500  # Reduce from 1000
   ```

3. **Clean Up Resources**:
   ```sql
   -- Clean up old data more frequently
   SELECT cleanup_expired_captcha_challenges();

   -- Reduce data retention
   DELETE FROM captcha_validation_attempts WHERE created_at < NOW() - INTERVAL '7 days';
   ```

## Monitoring and Alerting

### Key Metrics to Monitor

1. **Performance Metrics**:
   - Response time < 2 seconds
   - Success rate > 80%
   - Memory usage < 85%
   - CPU usage < 80%

2. **Security Metrics**:
   - Bot detection accuracy > 85%
   - False positive rate < 10%
   - Attack attempts per minute
   - Blocked IPs count

3. **Business Metrics**:
   - User abandonment rate < 20%
   - Accessibility usage rate
   - User satisfaction score > 70%

### Alert Thresholds

```yaml
# Critical Alerts (immediate response required)
- Service down: 30 seconds
- High attack rate: > 50 attempts/minute
- Bot detection accuracy < 70%

# Warning Alerts (response within 1 hour)
- High failure rate: > 70% for 5 minutes
- Slow response: > 2 seconds for 3 minutes
- High resource usage: > 85% for 5 minutes
```

## Escalation Procedures

### Level 1: Operations Team
- **Response Time**: 15 minutes
- **Scope**: Performance issues, minor configuration problems
- **Actions**: Restart services, adjust configuration, basic troubleshooting

### Level 2: Security Team
- **Response Time**: 30 minutes
- **Scope**: Security incidents, bot detection issues, attack patterns
- **Actions**: Block IPs, adjust security settings, investigate threats

### Level 3: Development Team
- **Response Time**: 1 hour
- **Scope**: Code issues, integration problems, complex bugs
- **Actions**: Code fixes, deployment updates, architectural changes

### Level 4: Management
- **Response Time**: 2 hours
- **Scope**: Business impact, major outages, security breaches
- **Actions**: Business decisions, external communication, resource allocation

## Useful Commands and Scripts

### Health Check Script
```bash
#!/bin/bash
# health-check.sh

echo "=== CAPTCHA System Health Check ==="

# Check services
echo "Checking services..."
docker-compose ps | grep -E "(authenc|captcha)"

# Check endpoints
echo "Checking endpoints..."
curl -f http://localhost:8088/api/v1/captcha/health || echo "CAPTCHA health check failed"

# Check database
echo "Checking database..."
pg_isready -h captcha-analytics-db -p 5432 -U captcha_user || echo "Database not ready"

# Check Redis
echo "Checking Redis..."
redis-cli ping || echo "Redis not responding"

# Check recent metrics
echo "Recent metrics..."
curl -s http://localhost:8088/api/v1/captcha/metrics | grep -E "(success_rate|bot_detection)"

echo "=== Health Check Complete ==="
```

### Performance Analysis Script
```bash
#!/bin/bash
# performance-analysis.sh

echo "=== CAPTCHA Performance Analysis ==="

# Response time analysis
echo "Response time analysis (last hour):"
curl -s "http://localhost:9090/api/v1/query?query=histogram_quantile(0.95,captcha_validation_duration_seconds[1h])"

# Success rate analysis
echo "Success rate (last hour):"
curl -s "http://localhost:9090/api/v1/query?query=captcha_success_rate[1h]"

# Resource usage
echo "Resource usage:"
docker stats --no-stream --format "table {{.Container}}\t{{.CPUPerc}}\t{{.MemUsage}}"

echo "=== Performance Analysis Complete ==="
```

### Emergency Response Script
```bash
#!/bin/bash
# emergency-response.sh

ACTION=${1:-status}

case $ACTION in
  "disable")
    echo "Disabling CAPTCHA system..."
    redis-cli set "captcha_disabled" "true" EX 1800
    ;;
  "enable")
    echo "Enabling CAPTCHA system..."
    redis-cli del "captcha_disabled"
    ;;
  "reduce-difficulty")
    echo "Reducing CAPTCHA difficulty..."
    redis-cli set "emergency_difficulty" "1" EX 3600
    ;;
  "block-ip")
    IP=${2}
    echo "Blocking IP: $IP"
    redis-cli sadd "blocked_ips" "$IP"
    ;;
  "status")
    echo "CAPTCHA system status:"
    redis-cli get "captcha_disabled"
    redis-cli get "emergency_difficulty"
    ;;
  *)
    echo "Usage: $0 {disable|enable|reduce-difficulty|block-ip <ip>|status}"
    ;;
esac
```

## Contact Information

### Emergency Contacts
- **Security Team**: security@kejaksaan.go.id, +62-xxx-xxx-xxxx
- **Operations Team**: ops@kejaksaan.go.id, +62-xxx-xxx-xxxx
- **On-Call Engineer**: oncall@kejaksaan.go.id, +62-xxx-xxx-xxxx

### Support Channels
- **Slack**: #captcha-support, #security-alerts
- **Email**: support@kejaksaan.go.id
- **Ticketing**: https://helpdesk.kejaksaan.go.id

---

**Document Version**: 1.0
**Last Updated**: $(date)
**Next Review**: $(date -d "+1 month")
**Owner**: SIMPelv2 Operations Team
