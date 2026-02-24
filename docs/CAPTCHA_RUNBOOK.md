# CAPTCHA System Runbook

## Quick Reference

### Emergency Procedures
```bash
# Disable CAPTCHA system (emergency)
redis-cli set "captcha_disabled" "true" EX 1800

# Enable emergency mode (reduced difficulty)
redis-cli set "emergency_mode" "true" EX 3600

# Block suspicious IP
redis-cli sadd "blocked_ips" "192.168.1.100"

# Check system status
curl -f http://localhost:8088/api/v1/captcha/health
```

### Key Contacts
- **On-Call Engineer**: +62-xxx-xxx-xxxx
- **Security Team**: security@kejaksaan.go.id
- **Operations Team**: ops@kejaksaan.go.id

## Standard Operating Procedures

### 1. Daily Health Check

**Frequency**: Every morning at 08:00 WIB
**Duration**: 10 minutes
**Owner**: Operations Team

#### Checklist
- [ ] Check service status
- [ ] Verify key metrics
- [ ] Reviewnight alerts
- [ ] Check backup status
- [ ] Validate integrations

#### Procedure
```bash
#!/bin/bash
# Daily health check script

echo "=== CAPTCHA Daily Health Check - $(date) ==="

# 1. Check service status
echo "1. Checking service status..."
docker-compose ps | grep -E "(authenc|captcha)" | grep -v "Up" && echo "❌ Service issues detected" || echo "✅ All services running"

# 2. Check health endpoints
echo "2. Checking health endpoints..."
curl -f http://localhost:8088/api/v1/captcha/health > /dev/null 2>&1 && echo "✅ CAPTCHA health OK" || echo "❌ CAPTCHA health check failed"

# 3. Check key metrics (last 24 hours)
echo "3. Checking key metrics..."
SUCCESS_RATE=$(curl -s "http://localhost:9090/api/v1/query?query=avg_over_time(captcha_success_rate[24h])" | jq -r '.data.result[0].value[1]')
echo "Success rate (24h): ${SUCCESS_RATE}%"

BOT_DETECTION=$(curl -s "http://localhost:9090/api/v1/query?query=avg_over_time(captcha_bot_detection_accuracy[24h])" | jq -r '.data.result[0].value[1]')
echo "Bot detection accuracy (24h): ${BOT_DETECTION}%"

# 4. Check database status
echo "4. Checking database..."
pg_isready -h captcha-analytics-db -p 5432 -U captcha_user > /dev/null 2>&1 && echo "✅ Database OK" || echo "❌ Database connection failed"

# 5. Check Redis status
echo "5. Checking Redis..."
redis-cli ping > /dev/null 2>&1 && echo "✅ Redis OK" || echo "❌ Redis connection failed"

# 6. Check recent alerts
echo "6. Checking recent alerts..."
ALERT_COUNT=$(curl -s "http://localhost:9093/api/v1/alerts" | jq '.data | length')
echo "Active alerts: $ALERT_COUNT"

# 7. Check backup status
echo "7. Checking backup status..."
LATEST_BACKUP=$(ls -t /backups/captcha_backup_*.sql.gz 2>/dev/null | head -1)
if [ -n "$LATEST_BACKUP" ]; then
    BACKUP_AGE=$(( ($(date +%s) - $(stat -c %Y "$LATEST_BACKUP")) / 3600 ))
    echo "Latest backup: $BACKUP_AGE hours old"
    [ $BACKUP_AGE -lt 25 ] && echo "✅ Backup OK" || echo "❌ Backup too old"
else
    echo "❌ No backup found"
fi

echo "=== Health Check Complete ==="
```

#### Success Criteria
- All services running (Up status)
- Health endpoints responding (HTTP 200)
- Success rate > 80%
- Bot detection accuracy > 85%
- Database and Redis accessible
- Backup < 25 hours old
- No critical alerts

#### Escalation
If any check fails:
1. **Minor Issues**: Create ticket, investigate within 2 hours
2. **Major Issues**: Page on-call engineer immediately
3. **Critical Issues**: Activate incident response

### 2. Weekly Performance Review

**Frequency**: Every Monday at 10:00 WIB
**Duration**: 30 minutes
**Owner**: Operations Team

#### Checklist
- [ ] Review performance trends
- [ ] Analyze security metrics
- [ ] Check capacity utilization
- [ ] Review user feedback
- [ ] Update documentation

#### Procedure
```bash
#!/bin/bash
# Weekly performance review script

echo "=== CAPTCHA Weekly Performance Review - $(date) ==="

# 1. Performance trends (last 7 days)
echo "1. Performance Trends (7 days):"
curl -s "http://localhost:9090/api/v1/query_range?query=captcha_success_rate&start=$(date -d '7 days ago' -Iseconds)&end=$(date -Iseconds)&step=1h" | jq -r '.data.result[0].values[] | "\(.[0]): \(.[1])%"' | tail -5

# 2. Security metrics
echo "2. Security Metrics (7 days):"
BOT_DETECTIONS=$(curl -s "http://localhost:9090/api/v1/query?query=sum(increase(captcha_bot_detections_total[7d]))" | jq -r '.data.result[0].value[1]')
ATTACK_ATTEMPTS=$(curl -s "http://localhost:9090/api/v1/query?query=sum(increase(captcha_attack_attempts_total[7d]))" | jq -r '.data.result[0].value[1]')
echo "Bot detections: $BOT_DETECTIONS"
echo "Attack attempts: $ATTACK_ATTEMPTS"

# 3. Capacity utilization
echo "3. Capacity Utilization:"
AVG_CPU=$(curl -s "http://localhost:9090/api/v1/query?query=avg_over_time(captcha_cpu_usage[7d])" | jq -r '.data.result[0].value[1]')
AVG_MEMORY=$(curl -s "http://localhost:9090/api/v1/query?query=avg_over_time(captcha_memory_usage[7d])" | jq -r '.data.result[0].value[1]')
echo "Average CPU: ${AVG_CPU}%"
echo "Average Memory: ${AVG_MEMORY}%"

# 4. Database statistics
echo "4. Database Statistics:"
psql -h captcha-analytics-db -U captcha_user -d captcha_analytics -t -c "
SELECT
    'Total challenges: ' || COUNT(*),
    'Solved challenges: ' || COUNT(*) FILTER (WHERE solved = true),
    'Bot detections: ' || (SELECT COUNT(*) FROM captcha_behavioral_metrics WHERE classification = 'Bot' AND created_at > NOW() - INTERVAL '7 days')
FROM captcha_challenges
WHERE created_at > NOW() - INTERVAL '7 days';
"

echo "=== Weekly Review Complete ==="
```

#### Action Items
Based on review results:
- **Performance < 80%**: Investigate and optimize
- **Security incidents**: Review and update defenses
- **Capacity > 80%**: Plan scaling
- **User complaints**: Address usability issues

### 3. Monthly Security Review

**Frequency**: First Monday of each month at 14:00 WIB
**Duration**: 60 minutes
**Owner**: Security Team

#### Checklist
- [ ] Review bot detection accuracy
- [ ] Analyze attack patterns
- [ ] Update threat intelligence
- [ ] Review blocked IPs
- [ ] Test security controls

#### Procedure
```sql
-- Monthly security analysis queries

-- 1. Bot detection accuracy analysis
SELECT
    DATE_TRUNC('week', created_at) as week,
    classification,
    COUNT(*) as count,
    AVG(risk_score) as avg_risk_score
FROM captcha_behavioral_metrics
WHERE created_at > NOW() - INTERVAL '30 days'
GROUP BY DATE_TRUNC('week', created_at), classification
ORDER BY week DESC, classification;

-- 2. Attack pattern analysis
SELECT
    DATE(created_at) as date,
    COUNT(*) as total_attempts,
    COUNT(*) FILTER (WHERE success = false) as failed_attempts,
    COUNT(DISTINCT ip_address) as unique_ips,
    COUNT(*) FILTER (WHERE risk_assessment IN ('High', 'Critical')) as high_risk_attempts
FROM captcha_validation_attempts
WHERE created_at > NOW() - INTERVAL '30 days'
GROUP BY DATE(created_at)
ORDER BY date DESC;

-- 3. Top attacking IPs
SELECT
    ip_address,
    COUNT(*) as attempts,
    COUNT(*) FILTER (WHERE success = false) as failures,
    MAX(created_at) as last_attempt,
    AVG(CASE WHEN bm.risk_score IS NOT NULL THEN bm.risk_score END) as avg_risk_score
FROM captcha_validation_attempts va
LEFT JOIN captcha_behavioral_metrics bm ON va.behavioral_metrics_id = bm.id
WHERE va.created_at > NOW() - INTERVAL '30 days'
GROUP BY ip_address
HAVING COUNT(*) > 100
ORDER BY attempts DESC
LIMIT 20;
```

#### Security Actions
1. **Update IP blocklist** based on analysis
2. **Adjust bot detection thresholds** if needed
3. **Review and update security rules**
4. **Generate security report** for management

### 4. Incident Response Procedures

#### Severity Levels

**P1 - Critical (Response: 15 minutes)**
- CAPTCHA system completely down
- Security breach detected
- Data loss or corruption

**P2 - High (Response: 1 hour)**
- Significant performance degradation
- High bot detection failure rate
- Integration failures

**P3 - Medium (Response: 4 hours)**
- Minor performance issues
- Configuration problems
- Non-criticalilures

**P4 - Low (Response: 24 hours)**
- Enhancement requests
- Documentation updates
- Minor bugs

#### P1 Critical Incident Response

**Step 1: Immediate Response (0-5 minutes)**
```bash
# 1. Assess situation
curl -f http://localhost:8088/api/v1/captcha/health
docker-compose ps | grep captcha

# 2. Check for obvious issues
docker logs --tail=50 simpelv2-authenc | grep -i error
redis-cli ping
pg_isready -h captcha-analytics-db -p 5432

# 3. Notify team
# Send alert to #incident-response Slack channel
# Page on-call engineer if not already aware
```

**Step 2: Triage and Containment (5-15 minutes)**
```bash
# 1. If security breach suspected
redis-cli set "captcha_disabled" "true" EX 3600  # Disable for 1 hour

# 2. If performance issue
docker stats --no-stream  # Check resource usage
# Scale services if needed
docker-compose up -d --scale authenc=3

# 3. If database issue
# Check database connections and performance
psql -h captcha-analytics-db -U captcha_user -d captcha_analytics -c "SELECT count(*) FROM pg_stat_activity;"
```

**Step 3: Investigation (15-60 minutes)**
- Analyze logs and metrics
- Identify root cause
- Develop fix plan
- Communicate status updates

**Step 4: Resolution (1-4 hours)**
- Implement fix
- Test thoroughly
- Monitor for stability
- Document incident

**Step 5: Post-Incident (24-48 hours)**
- Conduct post-mortem
- Update procedures
- Implement preventive measures

### 5. Maintenance Procedures

#### Database Maintenance (Weekly)

```sql
-- Weekly database maintenance script

-- 1. Clean up expired challenges
SELECT cleanup_expired_captcha_challenges();

-- 2. Update table statistics
ANALYZE captcha_challenges;
ANALYZE captcha_behavioral_metrics;
ANALYZE captcha_validation_attempts;

-- 3. Refresh materialized views
REFRESH MATERIALIZED VIEW captcha_analytics;

-- 4. Check index usage
SELECT
    schemaname,
    tablename,
    indexname,
    idx_scan,
    idx_tup_read,
    idx_tup_fetch
FROM pg_stat_user_indexes
WHERE schemaname = 'public'
AND tablename LIKE 'captcha_%'
ORDER BY idx_scan DESC;

-- 5. Check table sizes
SELECT
    schemaname,
    tablename,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) as size,
    pg_size_pretty(pg_relation_size(schemaname||'.'||tablename)) as table_size,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename) - pg_relation_size(schemaname||'.'||tablename)) as index_size
FROM pg_tables
WHERE tablename LIKE 'captcha_%'
ORDER BY pg_toon_size(schemaname||'.'||tablename) DESC;
```

#### Cache Maintenance (Daily)

```bash
#!/bin/bash
# Daily Redis maintenance

echo "=== Redis Maintenance - $(date) ==="

# 1. Check memory usage
redis-cli info memory | grep -E "(used_memory_human|maxmemory_human)"

# 2. Check key statistics
redis-cli info keyspace

# 3. Clean up expired keys (if needed)
redis-cli eval "return redis.call('del', unpack(redis.call('keys', 'captcha:expired:*')))" 0

# 4. Check slow log
redis-cli slowlog get 10

echo "=== Redis Maintenance Complete ==="
```

#### Log Rotation (Daily)

```bash
#!/bin/bash
# Log rotation script

LOG_DIR="/var/log/simpelv2"
RETENTION_DAYS=30

# Rotate application logs
find $LOG_DIR -name "captcha*.log" -mtime +$RETENTION_DAYS -delete

# Compress old logs
find $LOG_DIR -name "captcha*.log" -mtime +1 -exec gzip {} \;

# Clean up Docker logs
docker system prune -f --filter "until=720h"  # 30 days

echo "Log rotation complete"
```

### 6. Monitoring and Alerting Procedures

#### Alert Response Matrix

| Alert | Severity | Response Time | Action |
|-------|----------|---------------|--------|
| Service Down | P1 | 5 minutes | Restart service, investigate |
| High Bot Detection | P1 | 15 minutes | Analyze attack, block IPs |
| Performance Degraded | P2 | 1 hour | Check resources, optimize |
| Integration Failure | P2 | 1 hour | Check connectivity, credentials |
| High Memory Usage | P3 | 4 hours | Monitor, plan scaling |

#### Alert Acknowledgment

```bash
# Acknowledge alert in AlertManager
curl -X POST http://localhost:9093/api/v1/alerts \
  -H "Content-Type: application/json" \
  -d '{
    "alerts": [{
      "labels": {
        "alertname": "CaptchaHighFailureRate",
        "severity": "warning"
      },
      "annotations":
"summary": "Acknowledged by ops team"
      }
    }]
  }'
```

#### Silence Alerts (Maintenance)

```bash
# Silence alerts during maintenance
curl -X POST http://localhost:9093/api/v1/silences \
  -H "Content-Type: application/json" \
  -d '{
    "matchers": [{
      "name": "service",
      "value": "captcha",
      "isRegex": false
    }],
    "startsAt": "'$(date -Iseconds)'",
    "endsAt": "'$(date -d '+2 hours' -Iseconds)'",
    "createdBy": "ops-team",
    "comment": "Scheduled maintenance"
  }'
```

### 7. Backup and Recovery Procedures

#### Backup Verification (Daily)

```bash
#!/bin/bash
# Backup verification script

BACKUP_DIR="/backups"
LATEST_BACKUP=$(ls -t $BACKUP_DIR/captcha_backup_*.sql.gz | head -1)

if [ -z "$LATEST_BACKUP" ]; then
    echo "❌ No backup found"
    exit 1
fi

# Check backup age
BACKUP_AGE=$(( ($(date +%s) - $(stat -c %Y "$LATEST_BACKUP")) / 3600 ))
echo "Latest backup: $BACKUP_AGE hours old"

if [ $BACKUP_AGE -gt 25 ]; then
    echo "❌ Backup too old"
    exit 1
fi

# Verify backup integrity
if pg_restore --list "$LATEST_BACKUP" > /dev/null 2>&1; then
    echo "✅ Backup integrity OK"
else
    echo "❌ Backup integrity check failed"
    exit 1
fi

# Check backup size
BACKUP_SIZE=$(stat -c%s "$LATEST_BACKUP")
MIN_SIZE=1048576  # 1MB minimum

if [ $BACKUP_SIZE -lt $MIN_SIZE ]; then
    echo "❌ Backup size too small: $BACKUP_SIZE bytes"
    exit 1
else
    echo "✅ Backup size OK: $BACKUP_SIZE bytes"
fi

echo "✅ Backup verification complete"
```

#### Recovery Test (Monthly)

```bash
#!/bin/bash
# Monthly recovery test

TEST_DB="captcha_recovery_test"
LATEST_BACKUP=$(ls -t /backups/captcha_backup_*.sql.gz | head -1)

echo "=== Recovery Test - $(date) ==="

# 1. Create test database
createdb -h captcha-analytics-db -U captcha_user $TEST_DB

# 2. Restore backup
pg_restore -h captcha-analytics-db -U captcha_user -d $TEST_DB "$LATEST_BACKUP"

# 3. Verify data
RECORD_COUNT=$(psql -h captcha-analytics-db -U captcha_user -d $TEST_DB -t -c "SELECT COUNT(*) FROM captcha_challenges;")
echo "Restored records: $RECORD_COUNT"

# 4. Clean up
dropdb -h captcha-analytics-db -U captcha_user $TEST_DB

echo "✅ Recovery test complete"
```

### 8. Configuration Management

#### Configuration Backup

```bash
#!/bin/bash
# Configuration backup script

CONFIG_BACKUP_DIR="/backups/config"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")

mkdir -p $CONFIG_BACKUP_DIR

# Backup configuration files
tar -czf "$CONFIG_BACKUP_DIR/captcha_config_$TIMESTAMP.tar.gz" \
    config/captcha.production.toml \
    config/captcha.monitoring.toml \
    .env.captcha \
    docker-compose.captcha.yml

echo "Configuration backup created: captcha_config_$TIMESTAMP.tar.gz"
```

#### Configuration Validation

```bash
#!/bin/bash
# Configuration validation script

echo "=== Configuration Validation ==="

# 1. Validate TOML syntax
toml-validator config/captcha.production.toml && echo "✅ Production config valid" || echo "❌ Production config invalid"
toml-validator config/captcha.monitoring.toml && echo "✅ Monitoring config valid" || echo "❌ Monitoring config invalid"

# 2. Check environment variables
source .env.captcha
REQUIRED_VARS=(
    "CAPTCHA_DB_HOST"
    "CAPTCHA_DB_PASSWORD"
    "AUTHENC_API_URL"
    "SECRETON_API_URL"
)

for var in "${REQUIRED_VARS[@]}"; do
    if [ -z "${!var}" ]; then
        echo "❌ Missing required variable: $var"
    else
        echo "✅ $var is set"
    fi
done

# 3. Test configuration
docker run --rm -v $(pwd)/config:/config simpelv2/authenc:latest --config-test

echo "=== Configuration Validation Complete ==="
```

---

**Document Version**: 1.0
**Last Updated**: $(date)
**Next Review**: $(date -d "+1 month")
**Owner**: SIMPEL Operations Team
