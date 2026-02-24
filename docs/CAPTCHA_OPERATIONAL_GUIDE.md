# CAPTCHA System Operational Guide

## Overview

The SIMPEL AI-Resistant CAPTCHA system is a comprehensive security solution designed to protect against automated attacks while maintaining accessibility and user experience. This guide provides operational procedures for system administrators and security teams.

## Table of Contents

1. [System Architecture](#system-architecture)
2. [Deployment Procedures](#deployment-procedures)
3. [Configuration Management](#configuration-management)
4. [Monitoring and Alerting](#monitoring-and-alerting)
5. [Troubleshooting](#troubleshooting)
6. [Security Procedures](#security-procedures)
7. [Maintenance Tasks](#maintenance-tasks)
8. [Performance Tuning](#performance-tuning)
9. [Backup and Recovery](#backup-and-recovery)
10. [Emergency Procedures](#emergency-procedures)

## System Architecture

### Components

The CAPTCHA system consists of the following components:

- **Frontend Component**: Leptos-based CAPTCHA widget integrated into portal login
- **Backend Service**: Rust-based service integrated with Authenc
- **Analytics Database**: PostgreSQL database for challenge and metrics storage
- **Cache Layer**: Redis for performance optimization and rate limiting
- **Monitoring Stack**: Prometheus, Grafana, and AlertManager
- **Log Aggregation**: Elasticsearch and Kibana

### Integration Points

- **Authenc**: Authentication and authorization service
- **Secreton**: Cryptographic key management and encryption
- **Portal**: Main user interface integration point

## Deployment Procedures

### Prerequisites

1. Docker and Docker Compose installed
2. Access to Authenc and Secreton services
3. PostgreSQL database available
4. Redis instance available
5. Monitoring infrastructure (Prometheus, Grafana)

### Initial Deployment

1. **Clone Configuration**:
   ```bash
   git clone <repository>
   cd simpelv2
   ```

2. **Configure Environment**:
   ```bash
   cp .env.captcha.production .env.captcha
   # Edit .env.captcha with production values
   ```

3. **Deploy Services**:
   ```bash
   docker-compose -f docker-compose.yml -f docker-compose.captcha.yml up -d
   ```

4. **Initialize Database**:
   ```bash
   docker exec -it simpelv2-captcha-analytics-db psql -U captcha_user -d captcha_analytics -f /docker-entrypoint-initdb.d/001_captcha_schema.sql
   ```

5. **Verify Deployment**:
   ```bash
   curl http://localhost:8088/api/v1/captcha/health
   ```

### Rolling Updates

1. **Prepare New Version**:
   ```bash
   docker pull simpelv2/authenc:latest
   ```

2. **Update Configuration** (if needed):
   ```bash
   # Update configuration files
   # Restart configuration-dependent services
   ```

3. **Rolling Update**:
   ```bash
   docker-compose -f docker-compose.yml -f docker-compose.captcha.yml up -d --no-deps authenc
   ```

4. **Verify Update**:
   ```bash
   # Check health endpoints
   # Verify metrics
   # Test CAPTCHA functionality
   ```

## Configuration Management

### Configuration Files

- `config/captcha.production.toml`: Main CAPTCHA configuration
- `config/captcha.monitoring.toml`: Monitoring and alerting configuration
- `.env.captcha`: Environment variables
- `docker-compose.captcha.yml`: Container orchestration

### Key Configuration Parameters

#### Security Settings
```toml
[captcha.security]
encrypt_challenges = true
hash_answers = true
secure_random = true
anti_replay = true
csrf_protection = true
```

#### Performance Settings
```toml
[captcha.performance]
cache_enabled = true
cache_ttl = 3600
connection_pool_size = 10
request_timeout = 5000
max_concurrent_challenges = 1000
```

#### Behavioral Analysis
```toml
[captcha.behavioral_analysis]
mouse_tracking = true
keystroke_analysis = true
timing_analysis = true
browser_fingerprinting = true
session_analysis = true
```

### Configuration Validation

Before applying configuration changes:

1. **Validate Syntax**:
   ```bash
   toml-validator config/captcha.production.toml
   ```

2. **Test Configuration**:
   ```bash
   docker run --rm -v $(pwd)/config:/config simpelv2/authenc:latest --config-test
   ```

3. **Apply Changes**:
   ```bash
   docker-compose restart authenc
   ```

## Monitoring and Alerting

### Key Metrics

#### Performance Metrics
- `captcha_challenge_generation_duration`: Time to generate challenges
- `captcha_validation_duration`: Time to validate responses
- `captcha_success_rate`: Percentage of successful validations
- `captcha_failure_rate`: Percentage of failed validations

#### Security Metrics
- `captcha_bot_detection_rate`: Percentage of bot detections
- `captcha_bot_detection_accuracy`: Accuracy of bot detection
- `captcha_attack_attempts`: Number of attack attempts
- `captcha_blocked_ips`: Number of blocked IP addresses

#### Business Metrics
- `captcha_abandonment_rate`: Percentage of abandoned challenges
- `captcha_user_satisfaction_score`: User experience score
- `captcha_accessibility_usage_rate`: Accessibility feature usage

### Dashboard Access

- **Grafana**: http://captcha-grafana:3000
  - Username: admin
  - Password: (from environment variable)

- **Prometheus**: http://captcha-prometheus:9090

- **Kibana**: http://captcha-kibana:5601

### Alert Channels

Alerts are sent to:
- Email: admin@kejaksaan.go.id, security@kejaksaan.go.id
- Slack: #security-alerts, #ops-alerts
- Webhook: Configured webhook endpoints
- PagerDuty: (if configured)

## Troubleshooting

### Common Issues

#### High Failure Rate

**Symptoms**: CAPTCHA failure rate > 70%

**Possible Causes**:
- Difficulty level too high
- Bot attack in progress
- Service performance issues
- Configuration problems

**Resolution Steps**:
1. Check current difficulty level:
   ```sql
   SELECT AVG(difficulty_level) FROM captcha_challenges WHERE created_at > NOW() - INTERVAL '1 hour';
   ```

2. Review bot detection metrics:
   ```bash
   curl http://localhost:8088/api/v1/captcha/metrics | grep bot_detection
   ```

3. Check service performance:
   ```bash
   docker stats simpelv2-authenc
   ```

4. Temporarily reduce difficulty:
   ```sql
   UPDATE captcha_difficulty_adjustments SET difficulty_level = 2 WHERE active = true;
   ```

#### Slow Response Times

**Symptoms**: Response time > 2 seconds

**Possible Causes**:
- Database performance issues
- High concurrent load
- Network latency
- Resource constraints

**Resolution Steps**:
1. Check database performance:
   ```sql
   SELECT * FROM pg_stat_activity WHERE state = 'active';
   ```

2. Monitor resource usage:
   ```bash
   docker stats
   ```

3. Check cache hit rate:
   ```bash
   redis-cli info stats | grep keyspace_hits
   ```

4. Scale services if needed:
   ```bash
   docker-compose up -d --scale authenc=3
   ```

#### Integration Issues

**Symptoms**: Secreton or Authenc connection errors

**Resolution Steps**:
1. Check service connectivity:
   ```bash
   curl -f http://secreton:8200/v1/sys/health
   curl -f http://authenc:8088/health
   ```

2. Verify authentication:
   ```bash
   # Check Secreton token
   secreton auth -method=token

   # Check Authenc credentials
   curl -X POST http://authenc:8088/realms/simpel/protocol/openid-connect/token
   ```

3. Review logs:
   ```bash
   docker logs simpelv2-authenc | grep -i error
   ```

### Log Analysis

#### Common Log Patterns

**Bot Detection**:
```
level=WARN msg="Bot behavior detected" ip=192.168.1.100 risk_score=0.95 classification=Bot
```

**Rate Limiting**:
```
level=INFO msg="Rate limit triggered" ip=192.168.1.100 attempts=5 window=60s
```

**Performance Issues**:
```
level=WARN msg="Slow challenge generation" duration=3.2s challenge_type=Visual
```

#### Log Queries

**Elasticsearch Queries**:
```json
{
  "query": {
    "bool": {
      "must": [
        {"match": {"service": "captcha"}},
        {"range": {"@timestamp": {"gte": "now-1h"}}}
      ]
    }
  }
}
```

## Security Procedures

### Incident Response

#### Security Incident Detection

1. **Automated Alerts**: Monitor for high bot detection rates, attack attempts
2. **Manual Monitoring**: Regular review of security dashboards
3. **User Reports**: Investigate user-reported issues

#### Response Procedures

1. **Immediate Response** (< 5 minutes):
   - Assess threat level
   - Activate emergency rate limiting if needed
   - Notify security team

2. **Investigation** (< 30 minutes):
   - Analyze attack patterns
   - Identify source IPs/networks
   - Review bot detection accuracy

3. **Mitigation** (< 1 hour):
   - Block malicious IPs
   - Increase difficulty for suspicious patterns
   - Adjust rate limiting parameters

4. **Recovery** (< 4 hours):
   - Restore normal operations
   - Document incident
   - Update security measures

### Security Hardening

#### Regular Security Tasks

1. **Weekly**:
   - Review blocked IPs
   - Analyze bot detection accuracy
   - Check for configuration drift

2. **Monthly**:
   - Rotate encryption keys
   - Update security signatures
   - Review access logs

3. **Quarterly**:
   - Security assessment
   - Penetration testing
   - Update threat models

## Maintenance Tasks

### Daily Tasks

1. **Health Checks**:
   ```bash
   ./scripts/health-check.sh
   ```

2. **Backup Verification**:
   ```bash
   ls -la /backups/captcha_backup_$(date +%Y%m%d)*.sql.gz
   ```

3. **Log Review**:
   - Check error logs
   - Review security events
   - Monitor performance metrics

### Weekly Tasks

1. **Database Maintenance**:
   ```sql
   -- Clean up expired challenges
   SELECT cleanup_expired_captcha_challenges();

   -- Refresh analytics
   REFRESH MATERIALIZED VIEW captcha_analytics;

   -- Analyze table statistics
   ANALYZE captcha_challenges, captcha_behavioral_metrics, captcha_validation_attempts;
   ```

2. **Performance Review**:
   - Analyze response times
   - Review cache hit rates
   - Check resource utilization

3. **Security Review**:
   - Review bot detection accuracy
   - Analyze attack patterns
   - Update threat intelligence

### Monthly Tasks

1. **Capacity Planning**:
   - Review growth trends
   - Plan resource scaling
   - Update capacity forecasts

2. **Configuration Review**:
   - Validate configuration settings
   - Update documentation
   - Review change logs

3. **Disaster Recovery Testing**:
   - Test backup procedures
   - Validate recovery processes
   - Update runbooks

## Performance Tuning

### Database Optimization

1. **Index Optimization**:
   ```sql
   -- Check index usage
   SELECT schemaname, tablename, attname, n_distinct, correlation
   FROM pg_stats
   WHERE tablename LIKE 'captcha_%';

   -- Rebuild indexes if needed
   REINDEX TABLE captcha_challenges;
   ```

2. **Query Optimization**:
   ```sql
   -- Enable query logging
   ALTER SYSTEM SET log_statement = 'all';
   ALTER SYSTEM SET log_min_duration_statement = 1000;

   -- Analyze slow queries
   SELECT query, mean_time, calls
   FROM pg_stat_statements
   WHERE query LIKE '%captcha%'
   ORDER BY mean_time DESC;
   ```

### Cache Optimization

1. **Redis Tuning**:
   ```bash
   # Check memory usage
   redis-cli info memory

   # Optimize memory settings
   redis-cli config set maxmemory-policy allkeys-lru
   redis-cli config set maxmemory 256mb
   ```

2. **Cache Strategy**:
   - Challenge templates: 1 hour TTL
   - User sessions: 30 minutes TTL
   - Rate limiting: 5 minutes TTL
   - Analytics: 15 minutes TTL

### Application Tuning

1. **Connection Pooling**:
   ```toml
   [captcha.performance]
   connection_pool_size = 20
   connection_timeout = 30
   idle_timeout = 300
   ```

2. **Async Processing**:
   - Behavioral analysis: Background processing
   - Analytics updates: Batch processing
   - Log aggregation: Async streaming

## Backup and Recovery

### Backup Procedures

1. **Automated Backups**:
   - Daily database backups
   - Configuration backups
   - Log archival

2. **Backup Verification**:
   ```bash
   # Test backup integrity
   pg_restore --list /backups/captcha_backup_latest.sql.gz

   # Verify backup size
   du -h /backups/captcha_backup_*.sql.gz
   ```

### Recovery Procedures

1. **Database Recovery**:
   ```bash
   # Stop services
   docker-compose stop authenc

   # Restore database
   pg_restore -h captcha-analytics-db -U captcha_user -d captcha_analytics /backups/captcha_backup_latest.sql.gz

   # Start services
   docker-compose start authenc
   ```

2. **Configuration Recovery**:
   ```bash
   # Restore configuration
   tar -xzf /backups/captcha_config_latest.tar.gz -C /

   # Restart services
   docker-compose restart
   ```

## Emergency Procedures

### Service Outage

1. **Immediate Actions**:
   - Check service status
   - Review recent changes
   - Activate backup systems

2. **Communication**:
   - Notify stakeholders
   - Update status page
   - Provide regular updates

3. **Recovery**:
   - Implement fixes
   - Verify functionality
   - Post-incident review

### Security Breach

1. **Containment**:
   - Isolate affected systems
   - Block malicious traffic
   - Preserve evidence

2. **Assessment**:
   - Determine scope of breach
   - Identify compromised data
   - Assess impact

3. **Recovery**:
   - Implement security fixes
   - Restore services
   - Monitor for reoccurrence

### Data Loss

1. **Assessment**:
   - Determine extent of loss
   - Identify last good backup
   - Assess recovery options

2. **Recovery**:
   - Restore from backups
   - Verify data integrity
   - Resume operations

3. **Prevention**:
   - Improve backup procedures
   - Implement additional safeguards
   - Update disaster recovery plan

## Contact Information

### Emergency Contacts

- **Security Team**: security@kejaksaan.go.id
- **Operations Team**: ops@kejaksaan.go.id
- **On-Call Engineer**: +62-xxx-xxx-xxxx

### Escalation Matrix

1. **Level 1**: Operations Team (Response: 15 minutes)
2. **Level 2**: Security Team (Response: 30 minutes)
3. **Level 3**: Management (Response: 1 hour)

### External Vendors

- **Cloud Provider**: AWS Support
- **Monitoring**: Datadog Support
- **Security**: CrowdStrike Support

---

**Document Version**: 1.0
**Last Updated**: $(date)
**Next Review**: $(date -d "+3 months")
**Owner**: SIMPEL Security Team
