# Panduan Reporting dan Monitoring MFA SIMPEL

## Daftar Isi

1. [Dashboard dan Metrics](#dashboard-dan-metrics)
2. [Automated Reporting](#automated-reporting)
3. [Real-time Monitoring](#real-time-monitoring)
4. [Alerting System](#alerting-system)
5. [Compliance Reporting](#compliance-reporting)
6. [Performance Analytics](#performance-analytics)
7. [Security Intelligence](#security-intelligence)

---

## Dashboard dan Metrics

### Executive Dashboard

#### Key Performance Indicators (KPIs)

```sql
-- MFA Adoption Rate (Overall)
SELECT
    'MFA_ADOPTION_RATE' as metric,
    COUNT(CASE WHEN mfa_enabled THEN 1 END) as enabled_users,
    COUNT(*) as total_users,
    ROUND(
        COUNT(CASE WHEN mfa_enabled THEN 1 END) * 100.0 / COUNT(*),
        2
    ) as adoption_percentage
FROM users
WHERE active = true;

-- MFA Success Rate (Last 30 days)
SELECT
    'MFA_SUCCESS_RATE' as metric,
    COUNT(CASE WHEN event_type = 'verify_success' THEN 1 END) as successful_attempts,
    COUNT(*) as total_attempts,
    ROUND(
        COUNT(CASE WHEN event_type = 'verify_success' THEN 1 END) * 100.0 / COUNT(*),
        2
    ) as success_percentage
FROM mfa_logs
WHERE created_at >= CURRENT_DATE - INTERVAL '30 days'
  AND event_type IN ('verify_success', 'verify_failed');

-- Average Setup Time
SELECT
    'MFA_SETUP_TIME' as metric,
    AVG(EXTRACT(EPOCH FROM (mfa_setup_at - created_at))/60) as avg_setup_minutes,
    MIN(EXTRACT(EPOCH FROM (mfa_setup_at - created_at))/60) as min_setup_minutes,
    MAX(EXTRACT(EPOCH FROM (mfa_setup_at - created_at))/60) as max_setup_minutes
FROM users
WHERE mfa_setup_at IS NOT NULL
  AND created_at >= CURRENT_DATE - INTERVAL '30 days';
```

#### Operational Dashboard Queries

```sql
-- Daily Active MFA Users
SELECT
    DATE(created_at) as date,
    COUNT(DISTINCT user_id) as active_mfa_users,
    COUNT(CASE WHEN event_type = 'verify_success' THEN 1 END) as successful_logins,
    COUNT(CASE WHEN event_type = 'verify_failed' THEN 1 END) as failed_attempts
FROM mfa_logs
WHERE created_at >= CURRENT_DATE - INTERVAL '7 days'
GROUP BY DATE(created_at)
ORDER BY date DESC;

-- MFA Adoption by Organizational Unit
SELECT
    s.nama_satker,
    s.kode_satker,
    COUNT(u.id) as total_users,
    COUNT(CASE WHEN u.mfa_enabled THEN 1 END) as mfa_enabled_users,
    ROUND(
        COUNT(CASE WHEN u.mfa_enabled THEN 1 END) * 100.0 / COUNT(u.id),
        2
    ) as adoption_rate,
    COUNT(CASE WHEN u.mfa_required AND NOT u.mfa_enabled THEN 1 END) as pending_setups
FROM users u
JOIN satkers s ON u.satker_code = s.kode_satker
WHERE u.active = true
GROUP BY s.nama_satker, s.kode_satker
ORDER BY adoption_rate DESC;

-- Support Ticket Trends
SELECT
    DATE_TRUNC('week', created_at) as week,
    COUNT(*) as total_tickets,
    COUNT(CASE WHEN category = 'mfa_setup' THEN 1 END) as setup_issues,
    COUNT(CASE WHEN category = 'mfa_login' THEN 1 END) as login_issues,
    COUNT(CASE WHEN category = 'mfa_reset' THEN 1 END) as reset_requests,
    AVG(EXTRACT(EPOCH FROM (resolved_at - created_at))/3600) as avg_resolution_hours
FROM support_tickets
WHERE category LIKE 'mfa_%'
  AND created_at >= CURRENT_DATE - INTERVAL '12 weeks'
GROUP BY DATE_TRUNC('week', created_at)
ORDER BY week DESC;
```

### Technical Dashboard

#### System Health Metrics

```sql
-- MFA Service Performance
SELECT
    DATE_TRUNC('hour', timestamp) as hour,
    AVG(response_time_ms) as avg_response_time,
    MAX(response_time_ms) as max_response_time,
    COUNT(*) as total_requests,
    COUNT(CASE WHEN status_code >= 500 THEN 1 END) as error_count,
    ROUND(
        COUNT(CASE WHEN status_code < 400 THEN 1 END) * 100.0 / COUNT(*),
        2
    ) as success_rate
FROM api_logs
WHERE endpoint LIKE '/api/auth/mfa/%'
  AND timestamp >= CURRENT_TIMESTAMP - INTERVAL '24 hours'
GROUP BY DATE_TRUNC('hour', timestamp)
ORDER BY hour DESC;

-- Database Performance
SELECT
    schemaname,
    tablename,
    seq_scan,
    seq_tup_read,
    idx_scan,
    idx_tup_fetch,
    n_tup_ins,
    n_tup_upd,
    n_tup_del
FROM pg_stat_user_tables
WHERE tablename IN ('users', 'mfa_logs', 'mfa_secrets')
ORDER BY seq_scan DESC;

-- Cache Hit Rates
SELECT
    'redis_mfa_cache' as cache_type,
    info.keyspace_hits,
    info.keyspace_misses,
    ROUND(
        info.keyspace_hits * 100.0 / (info.keyspace_hits + info.keyspace_misses),
        2
    ) as hit_rate_percentage
FROM redis_info info;
```

---

## Automated Reporting

### Daily Reports

#### Daily MFA Summary Report

```bash
#!/bin/bash
# Script: /scripts/reports/daily_mfa_summary.sh

REPORT_DATE=$(date +%Y-%m-%d)
REPORT_FILE="/var/reports/daily/mfa_summary_${REPORT_DATE}.json"

# Generate comprehensive daily report
./scripts/cli/target/release/simipelv2-cli reports generate \
  --type daily_mfa_summary \
  --date $REPORT_DATE \
  --output $REPORT_FILE \
  --format json

# Email report to stakeholders
./scripts/cli/target/release/simipelv2-cli reports email \
  --file $REPORT_FILE \
  --template daily_mfa_summary \
  --recipients "admin@kejaksaan.go.id,security@kejaksaan.go.id" \
  --subject "Daily MFA Summary - $REPORT_DATE"

# Upload to dashboard
curl -X POST https://dashboard.internal/api/reports \
  -H "Authorization: Bearer $DASHBOARD_TOKEN" \
  -H "Content-Type: application/json" \
  -d @$REPORT_FILE
```

#### Daily Security Events Report

```sql
-- Daily security events related to MFA
WITH daily_security_events AS (
    SELECT
        event_type,
        COUNT(*) as event_count,
        COUNT(DISTINCT user_id) as affected_users,
        COUNT(DISTINCT ip_address) as unique_ips,
        array_agg(DISTINCT satker_code) as affected_satkers
    FROM mfa_security_events
    WHERE DATE(created_at) = CURRENT_DATE
    GROUP BY event_type
)
SELECT
    json_build_object(
        'report_date', CURRENT_DATE,
        'report_type', 'daily_security_events',
        'summary', json_build_object(
            'total_events', (SELECT SUM(event_count) FROM daily_security_events),
            'affected_users', (SELECT SUM(affected_users) FROM daily_security_events),
            'unique_ips', (SELECT SUM(unique_ips) FROM daily_security_events)
        ),
        'events', json_agg(
            json_build_object(
                'event_type', event_type,
                'count', event_count,
                'affected_users', affected_users,
                'unique_ips', unique_ips,
                'affected_satkers', affected_satkers
            )
        )
    ) as report_json
FROM daily_security_events;
```

### Weekly Reports

#### Weekly MFA Adoption Report

```bash
#!/bin/bash
# Script: /scripts/reports/weekly_mfa_adoption.sh

WEEK_START=$(date -d "last monday" +%Y-%m-%d)
WEEK_END=$(date -d "next sunday" +%Y-%m-%d)
REPORT_FILE="/var/reports/weekly/mfa_adoption_week_$(date +%Y%W).pdf"

# Generate adoption report with charts
./scripts/cli/target/release/simipelv2-cli reports generate \
  --type weekly_adoption \
  --start-date $WEEK_START \
  --end-date $WEEK_END \
  --output $REPORT_FILE \
  --format pdf \
  --include-charts

# Email to management
./scripts/cli/target/release/simipelv2-cli reports email \
  --file $REPORT_FILE \
  --template weekly_management \
  --recipients "management@kejaksaan.go.id,it-director@kejaksaan.go.id" \
  --subject "Weekly MFA Adoption Report - Week $(date +%Y%W)"
```

#### Weekly Performance Report

```sql
-- Weekly performance metrics
WITH weekly_metrics AS (
    SELECT
        DATE_TRUNC('day', created_at) as day,
        COUNT(CASE WHEN event_type = 'verify_success' THEN 1 END) as successful_verifications,
        COUNT(CASE WHEN event_type = 'verify_failed' THEN 1 END) as failed_verifications,
        COUNT(CASE WHEN event_type = 'setup_completed' THEN 1 END) as new_setups,
        COUNT(CASE WHEN event_type = 'account_locked' THEN 1 END) as account_lockouts,
        AVG(CASE WHEN event_type = 'verify_success' THEN response_time_ms END) as avg_verify_time
    FROM mfa_logs
    WHERE created_at >= DATE_TRUNC('week', CURRENT_DATE)
    GROUP BY DATE_TRUNC('day', created_at)
)
SELECT
    json_build_object(
        'report_period', json_build_object(
            'start_date', DATE_TRUNC('week', CURRENT_DATE),
            'end_date', DATE_TRUNC('week', CURRENT_DATE) + INTERVAL '6 days'
        ),
        'summary', json_build_object(
            'total_verifications', SUM(successful_verifications + failed_verifications),
            'success_rate', ROUND(SUM(successful_verifications) * 100.0 / NULLIF(SUM(successful_verifications + failed_verifications), 0), 2),
            'new_setups', SUM(new_setups),
            'account_lockouts', SUM(account_lockouts),
            'avg_verify_time_ms', ROUND(AVG(avg_verify_time), 2)
        ),
        'daily_breakdown', json_agg(
            json_build_object(
                'date', day,
                'successful_verifications', successful_verifications,
                'failed_verifications', failed_verifications,
                'new_setups', new_setups,
                'account_lockouts', account_lockouts,
                'avg_verify_time_ms', ROUND(avg_verify_time, 2)
            ) ORDER BY day
        )
    ) as weekly_report
FROM weekly_metrics;
```

### Monthly Reports

#### Monthly Compliance Report

```bash
#!/bin/bash
# Script: /scripts/reports/monthly_compliance.sh

MONTH=$(date +%Y-%m)
REPORT_FILE="/var/reports/monthly/compliance_${MONTH}.pdf"

# Generate comprehensive compliance report
./scripts/cli/target/release/simipelv2-cli reports generate \
  --type monthly_compliance \
  --month $MONTH \
  --output $REPORT_FILE \
  --format pdf \
  --include-audit-trail \
  --include-risk-assessment

# Email to compliance team
./scripts/cli/target/release/simipelv2-cli reports email \
  --file $REPORT_FILE \
  --template monthly_compliance \
  --recipients "compliance@kejaksaan.go.id,audit@kejaksaan.go.id,management@kejaksaan.go.id" \
  --subject "Monthly MFA Compliance Report - $MONTH"

# Archive report
cp $REPORT_FILE "/var/archives/compliance/mfa_compliance_${MONTH}.pdf"
```

---

## Real-time Monitoring

### Grafana Dashboard Configuration

#### MFA Overview Dashboard

```json
{
  "dashboard": {
    "title": "MFA Overview - SIMPEL",
    "panels": [
      {
        "title": "MFA Adoption Rate",
        "type": "stat",
        "targets": [
          {
            "expr": "mfa_adoption_rate",
            "legendFormat": "Adoption Rate %"
          }
        ]
      },
      {
        "title": "Daily MFA Verifications",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(mfa_verifications_total[5m])",
            "legendFormat": "Verifications/sec"
          }
        ]
      },
      {
        "title": "MFA Success Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "mfa_success_rate",
            "legendFormat": "Success Rate %"
          }
        ]
      },
      {
        "title": "Active MFA Users",
        "type": "stat",
        "targets": [
          {
            "expr": "mfa_active_users",
            "legendFormat": "Active Users"
          }
        ]
      }
    ]
  }
}
```

#### Security Monitoring Dashboard

```json
{
  "dashboard": {
    "title": "MFA Security Monitoring",
    "panels": [
      {
        "title": "Failed MFA Attempts",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(mfa_failed_attempts_total[5m])",
            "legendFormat": "Failed Attempts/sec"
          }
        ]
      },
      {
        "title": "Account Lockouts",
        "type": "graph",
        "targets": [
          {
            "expr": "increase(mfa_account_lockouts_total[1h])",
            "legendFormat": "Lockouts/hour"
          }
        ]
      },
      {
        "title": "Suspicious Activity",
        "type": "table",
        "targets": [
          {
            "expr": "mfa_suspicious_activity",
            "format": "table"
          }
        ]
      }
    ]
  }
}
```

### Prometheus Metrics

#### Custom MFA Metrics

```yaml
# /config/prometheus/mfa_metrics.yml
groups:
  - name: mfa_metrics
    rules:
      - record: mfa_adoption_rate
        expr: |
          (
            count(users{mfa_enabled="true"}) /
            count(users{active="true"})
          ) * 100

      - record: mfa_success_rate
        expr: |
          (
            rate(mfa_verifications_total{result="success"}[5m]) /
            rate(mfa_verifications_total[5m])
          ) * 100

      - record: mfa_active_users
        expr: |
          count(
            count by (user_id) (
              mfa_verifications_total{result="success"}[24h]
            )
          )

      - record: mfa_setup_completion_rate
        expr: |
          (
            rate(mfa_setups_total{result="completed"}[1h]) /
            rate(mfa_setups_total[1h])
          ) * 100
```

### Real-time Alerting Rules

#### Critical Alerts

```yaml
# /config/prometheus/mfa_alerts.yml
groups:
  - name: mfa_critical_alerts
    rules:
      - alert: MFAServiceDown
        expr: up{job="mfa-service"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "MFA service is down"
          description: "MFA service has been down for more than 1 minute"

      - alert: HighMFAFailureRate
        expr: mfa_success_rate < 80
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High MFA failure rate detected"
          description: "MFA success rate is {{ $value }}% for the last 5 minutes"

      - alert: MassAccountLockout
        expr: increase(mfa_account_lockouts_total[10m]) > 50
        for: 0m
        labels:
          severity: critical
        annotations:
          summary: "Mass account lockout detected"
          description: "{{ $value }} accounts locked in the last 10 minutes"

  - name: mfa_warning_alerts
    rules:
      - alert: LowMFAAdoption
        expr: mfa_adoption_rate < 90
        for: 1h
        labels:
          severity: warning
        annotations:
          summary: "Low MFA adoption rate"
          description: "MFA adoption rate is {{ $value }}%"

      - alert: SlowMFAResponse
        expr: histogram_quantile(0.95, mfa_response_time_seconds) > 2
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Slow MFA response time"
          description: "95th percentile MFA response time is {{ $value }}s"
```

---

## Alerting System

### Alert Configuration

#### Email Alerts

```yaml
# /config/alertmanager/mfa_routes.yml
route:
  group_by: ['alertname', 'severity']
  group_wait: 10s
  group_interval: 10s
  repeat_interval: 1h
  receiver: 'default'
  routes:
    - match:
        service: mfa
        severity: critical
      receiver: 'mfa-critical'
    - match:
        service: mfa
        severity: warning
      receiver: 'mfa-warning'

receivers:
  - name: 'mfa-critical'
    email_configs:
      - to: 'oncall@kejaksaan.go.id'
        subject: '[CRITICAL] MFA Alert: {{ .GroupLabels.alertname }}'
        body: |
          Alert: {{ .GroupLabels.alertname }}
          Severity: {{ .GroupLabels.severity }}
          Description: {{ range .Alerts }}{{ .Annotations.description }}{{ end }}

          Dashboard: https://monitoring.internal/mfa
          Runbook: https://docs.internal/mfa-runbook

  - name: 'mfa-warning'
    email_configs:
      - to: 'admin@kejaksaan.go.id'
        subject: '[WARNING] MFA Alert: {{ .GroupLabels.alertname }}'
```

#### Slack Integration

```yaml
# Slack webhook configuration
receivers:
  - name: 'mfa-slack'
    slack_configs:
      - api_url: 'https://hooks.slack.com/services/YOUR/SLACK/WEBHOOK'
        channel: '#mfa-alerts'
        title: 'MFA Alert: {{ .GroupLabels.alertname }}'
        text: |
          *Severity:* {{ .GroupLabels.severity }}
          *Description:* {{ range .Alerts }}{{ .Annotations.description }}{{ end }}
          *Dashboard:* <https://monitoring.internal/mfa|View Dashboard>
```

### SMS Alerts for Critical Issues

```bash
#!/bin/bash
# Script: /scripts/alerts/mfa_sms_alert.sh

ALERT_TYPE=$1
ALERT_MESSAGE=$2
PHONE_NUMBERS="081234567890,081234567891"

# Send SMS via SMS gateway
for phone in $(echo $PHONE_NUMBERS | tr "," "\n"); do
    curl -X POST https://sms-gateway.internal/send \
        -H "Authorization: Bearer $SMS_TOKEN" \
        -d "to=$phone" \
        -d "message=[MFA ALERT] $ALERT_TYPE: $ALERT_MESSAGE"
done
```

---

## Compliance Reporting

### Regulatory Compliance Reports

#### Monthly Compliance Summary

```sql
-- Monthly compliance report for regulatory requirements
WITH monthly_stats AS (
    SELECT
        DATE_TRUNC('month', CURRENT_DATE) as report_month,
        COUNT(DISTINCT u.id) as total_active_users,
        COUNT(DISTINCT CASE WHEN u.mfa_enabled THEN u.id END) as mfa_enabled_users,
        COUNT(DISTINCT CASE WHEN u.mfa_required AND NOT u.mfa_enabled THEN u.id END) as non_compliant_users,
        COUNT(DISTINCT ml.user_id) as users_with_mfa_activity
    FROM users u
    LEFT JOIN mfa_logs ml ON u.id = ml.user_id
        AND ml.created_at >= DATE_TRUNC('month', CURRENT_DATE)
    WHERE u.active = true
),
security_incidents AS (
    SELECT
        COUNT(*) as total_incidents,
        COUNT(CASE WHEN severity = 'high' THEN 1 END) as high_severity_incidents,
        COUNT(CASE WHEN resolved_at IS NOT NULL THEN 1 END) as resolved_incidents
    FROM security_incidents
    WHERE category = 'mfa'
        AND created_at >= DATE_TRUNC('month', CURRENT_DATE)
)
SELECT
    json_build_object(
        'report_type', 'monthly_compliance',
        'report_month', ms.report_month,
        'compliance_metrics', json_build_object(
            'total_users', ms.total_active_users,
            'mfa_enabled_users', ms.mfa_enabled_users,
            'compliance_rate', ROUND(ms.mfa_enabled_users * 100.0 / ms.total_active_users, 2),
            'non_compliant_users', ms.non_compliant_users,
            'active_mfa_users', ms.users_with_mfa_activity
        ),
        'security_metrics', json_build_object(
            'total_incidents', si.total_incidents,
            'high_severity_incidents', si.high_severity_incidents,
            'incident_resolution_rate', ROUND(si.resolved_incidents * 100.0 / NULLIF(si.total_incidents, 0), 2)
        ),
        'regulatory_status', CASE
            WHEN ms.mfa_enabled_users * 100.0 / ms.total_active_users >= 95 THEN 'COMPLIANT'
            WHEN ms.mfa_enabled_users * 100.0 / ms.total_active_users >= 80 THEN 'PARTIALLY_COMPLIANT'
            ELSE 'NON_COMPLIANT'
        END
    ) as compliance_report
FROM monthly_stats ms, security_incidents si;
```

#### Audit Trail Report

```sql
-- Comprehensive audit trail for compliance
SELECT
    json_build_object(
        'report_type', 'audit_trail',
        'report_period', json_build_object(
            'start_date', $1,
            'end_date', $2
        ),
        'admin_actions', (
            SELECT json_agg(
                json_build_object(
                    'timestamp', ama.created_at,
                    'admin_user', u.nama,
                    'admin_nip', u.nip,
                    'action_type', ama.action_type,
                    'target_user', tu.nama,
                    'target_nip', tu.nip,
                    'details', ama.details,
                    'ip_address', ama.ip_address
                )
            )
            FROM admin_mfa_actions ama
            JOIN users u ON ama.admin_user_id = u.id
            LEFT JOIN users tu ON ama.target_user_id = tu.id
            WHERE ama.created_at BETWEEN $1 AND $2
        ),
        'user_activities', (
            SELECT json_agg(
                json_build_object(
                    'timestamp', ml.created_at,
                    'user_nip', u.nip,
                    'user_name', u.nama,
                    'event_type', ml.event_type,
                    'ip_address', ml.ip_address,
                    'user_agent', ml.user_agent,
                    'result', ml.result
                )
            )
            FROM mfa_logs ml
            JOIN users u ON ml.user_id = u.id
            WHERE ml.created_at BETWEEN $1 AND $2
        )
    ) as audit_report;
```

### Risk Assessment Reports

#### Security Risk Assessment

```sql
-- Monthly security risk assessment
WITH risk_indicators AS (
    SELECT
        'high_failure_rate_users' as risk_type,
        COUNT(*) as count,
        'Users with >20% MFA failure rate' as description
    FROM (
        SELECT
            user_id,
            COUNT(CASE WHEN event_type = 'verify_failed' THEN 1 END) * 100.0 / COUNT(*) as failure_rate
        FROM mfa_logs
        WHERE created_at >= CURRENT_DATE - INTERVAL '30 days'
            AND event_type IN ('verify_success', 'verify_failed')
        GROUP BY user_id
        HAVING COUNT(*) >= 10
    ) user_failure_rates
    WHERE failure_rate > 20

    UNION ALL

    SELECT
        'stale_mfa_accounts' as risk_type,
        COUNT(*) as count,
        'MFA enabled but not used in 90 days' as description
    FROM users u
    LEFT JOIN mfa_logs ml ON u.id = ml.user_id
        AND ml.event_type = 'verify_success'
        AND ml.created_at >= CURRENT_DATE - INTERVAL '90 days'
    WHERE u.mfa_enabled = true
        AND ml.user_id IS NULL

    UNION ALL

    SELECT
        'multiple_lockouts' as risk_type,
        COUNT(*) as count,
        'Users locked out multiple times this month' as description
    FROM (
        SELECT user_id
        FROM mfa_logs
        WHERE event_type = 'account_locked'
            AND created_at >= DATE_TRUNC('month', CURRENT_DATE)
        GROUP BY user_id
        HAVING COUNT(*) > 2
    ) frequent_lockouts
)
SELECT
    json_build_object(
        'report_type', 'security_risk_assessment',
        'assessment_date', CURRENT_DATE,
        'risk_indicators', json_agg(
            json_build_object(
                'risk_type', risk_type,
                'count', count,
                'description', description,
                'risk_level', CASE
                    WHEN count > 50 THEN 'HIGH'
                    WHEN count > 10 THEN 'MEDIUM'
                    ELSE 'LOW'
                END
            )
        ),
        'overall_risk_score', (
            SELECT AVG(
                CASE
                    WHEN count > 50 THEN 3
                    WHEN count > 10 THEN 2
                    ELSE 1
                END
            )
            FROM risk_indicators
        )
    ) as risk_assessment
FROM risk_indicators;
```

---

## Performance Analytics

### Response Time Analysis

#### MFA Performance Metrics

```sql
-- Detailed performance analysis
WITH performance_metrics AS (
    SELECT
        DATE_TRUNC('hour', timestamp) as hour,
        endpoint,
        AVG(response_time_ms) as avg_response_time,
        PERCENTILE_CONT(0.5) WITHIN GROUP (ORDER BY response_time_ms) as p50_response_time,
        PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY response_time_ms) as p95_response_time,
        PERCENTILE_CONT(0.99) WITHIN GROUP (ORDER BY response_time_ms) as p99_response_time,
        COUNT(*) as request_count,
        COUNT(CASE WHEN status_code >= 500 THEN 1 END) as error_count
    FROM api_logs
    WHERE endpoint LIKE '/api/auth/mfa/%'
        AND timestamp >= CURRENT_TIMESTAMP - INTERVAL '24 hours'
    GROUP BY DATE_TRUNC('hour', timestamp), endpoint
)
SELECT
    json_build_object(
        'report_type', 'performance_analysis',
        'time_period', '24_hours',
        'endpoints', json_agg(
            json_build_object(
                'endpoint', endpoint,
                'avg_response_time_ms', ROUND(avg_response_time, 2),
                'p50_response_time_ms', ROUND(p50_response_time, 2),
                'p95_response_time_ms', ROUND(p95_response_time, 2),
                'p99_response_time_ms', ROUND(p99_response_time, 2),
                'total_requests', SUM(request_count),
                'error_rate', ROUND(SUM(error_count) * 100.0 / SUM(request_count), 2)
            )
        )
    ) as performance_report
FROM performance_metrics
GROUP BY endpoint;
```

### Capacity Planning

#### Usage Trend Analysis

```sql
-- Capacity planning metrics
WITH daily_usage AS (
    SELECT
        DATE(created_at) as date,
        COUNT(*) as total_mfa_operations,
        COUNT(DISTINCT user_id) as unique_users,
        MAX(hourly_peak.peak_operations) as peak_hourly_operations
    FROM mfa_logs ml
    JOIN (
        SELECT
            DATE(created_at) as date,
            DATE_TRUNC('hour', created_at) as hour,
            COUNT(*) as peak_operations
        FROM mfa_logs
        WHERE created_at >= CURRENT_DATE - INTERVAL '30 days'
        GROUP BY DATE(created_at), DATE_TRUNC('hour', created_at)
    ) hourly_peak ON DATE(ml.created_at) = hourly_peak.date
    WHERE ml.created_at >= CURRENT_DATE - INTERVAL '30 days'
    GROUP BY DATE(ml.created_at)
)
SELECT
    json_build_object(
        'report_type', 'capacity_planning',
        'analysis_period', '30_days',
        'current_metrics', json_build_object(
            'avg_daily_operations', ROUND(AVG(total_mfa_operations), 0),
            'max_daily_operations', MAX(total_mfa_operations),
            'avg_daily_users', ROUND(AVG(unique_users), 0),
            'max_daily_users', MAX(unique_users),
            'avg_peak_hourly', ROUND(AVG(peak_hourly_operations), 0),
            'max_peak_hourly', MAX(peak_hourly_operations)
        ),
        'growth_projections', json_build_object(
            'projected_6_month_daily_ops', ROUND(AVG(total_mfa_operations) * 1.5, 0),
            'projected_6_month_peak_hourly', ROUND(AVG(peak_hourly_operations) * 1.5, 0),
            'recommended_capacity_buffer', '200%'
        )
    ) as capacity_report
FROM daily_usage;
```

---

## Security Intelligence

### Threat Detection

#### Anomaly Detection Queries

```sql
-- Detect unusual MFA patterns
WITH user_baselines AS (
    SELECT
        user_id,
        AVG(daily_verifications) as avg_daily_verifications,
        STDDEV(daily_verifications) as stddev_daily_verifications
    FROM (
        SELECT
            user_id,
            DATE(created_at) as date,
            COUNT(*) as daily_verifications
        FROM mfa_logs
        WHERE event_type = 'verify_success'
            AND created_at >= CURRENT_DATE - INTERVAL '30 days'
        GROUP BY user_id, DATE(created_at)
    ) daily_stats
    GROUP BY user_id
    HAVING COUNT(*) >= 7  -- At least 7 days of data
),
today_activity AS (
    SELECT
        user_id,
        COUNT(*) as today_verifications,
        COUNT(DISTINCT ip_address) as unique_ips_today,
        array_agg(DISTINCT ip_address) as ip_addresses
    FROM mfa_logs
    WHERE event_type = 'verify_success'
        AND DATE(created_at) = CURRENT_DATE
    GROUP BY user_id
)
SELECT
    u.nip,
    u.nama,
    u.satker_code,
    ta.today_verifications,
    ub.avg_daily_verifications,
    ta.unique_ips_today,
    ta.ip_addresses,
    CASE
        WHEN ta.today_verifications > ub.avg_daily_verifications + (2 * ub.stddev_daily_verifications)
        THEN 'HIGH_VOLUME_ANOMALY'
        WHEN ta.unique_ips_today > 3
        THEN 'MULTIPLE_IP_ANOMALY'
        ELSE 'NORMAL'
    END as anomaly_type
FROM today_activity ta
JOIN user_baselines ub ON ta.user_id = ub.user_id
JOIN users u ON ta.user_id = u.id
WHERE ta.today_verifications > ub.avg_daily_verifications + (2 * ub.stddev_daily_verifications)
   OR ta.unique_ips_today > 3
ORDER BY ta.today_verifications DESC;
```

#### Geographic Analysis

```sql
-- Geographic anomaly detection
WITH user_locations AS (
    SELECT
        user_id,
        ip_address,
        country,
        city,
        COUNT(*) as login_count,
        MIN(created_at) as first_seen,
        MAX(created_at) as last_seen
    FROM mfa_logs ml
    JOIN ip_geolocation ig ON ml.ip_address = ig.ip_address
    WHERE ml.event_type = 'verify_success'
        AND ml.created_at >= CURRENT_DATE - INTERVAL '7 days'
    GROUP BY user_id, ip_address, country, city
),
suspicious_locations AS (
    SELECT
        user_id,
        COUNT(DISTINCT country) as countries_count,
        COUNT(DISTINCT city) as cities_count,
        array_agg(DISTINCT country) as countries,
        array_agg(DISTINCT city) as cities
    FROM user_locations
    GROUP BY user_id
    HAVING COUNT(DISTINCT country) > 1  -- Multiple countries
)
SELECT
    u.nip,
    u.nama,
    sl.countries_count,
    sl.cities_count,
    sl.countries,
    sl.cities,
    'GEOGRAPHIC_ANOMALY' as alert_type
FROM suspicious_locations sl
JOIN users u ON sl.user_id = u.id
ORDER BY sl.countries_count DESC, sl.cities_count DESC;
```

### Automated Threat Response

#### Auto-Response Rules

```bash
#!/bin/bash
# Script: /scripts/security/auto_threat_response.sh

THREAT_TYPE=$1
USER_ID=$2
SEVERITY=$3

case $THREAT_TYPE in
    "BRUTE_FORCE")
        # Temporarily lock account
        ./scripts/cli/target/release/simipelv2-cli mfa lock-account --user-id $USER_ID --duration 1h
        # Alert security team
        ./scripts/alerts/security_alert.sh "Brute force detected for user $USER_ID"
        ;;

    "GEOGRAPHIC_ANOMALY")
        if [ "$SEVERITY" = "HIGH" ]; then
            # Require additional verification
            ./scripts/cli/target/release/simipelv2-cli mfa require-additional-auth --user-id $USER_ID
        fi
        # Log for investigation
        ./scripts/security/log_investigation.sh "Geographic anomaly" $USER_ID
        ;;

    "MULTIPLE_DEVICE")
        # Flag for manual review
        ./scripts/cli/target/release/simipelv2-cli security flag-for-review --user-id $USER_ID --reason "Multiple device usage"
        ;;
esac
```

---

**Kontak Tim Monitoring**:

- **Email**: monitoring@kejaksaan.go.id
- **Slack**: #mfa-monitoring
- **Dashboard**: https://monitoring.internal/mfa

**Dokumen Terkait**:

- [MFA Admin Guide](./MFA_ADMIN_GUIDE.md)
- [MFA Security Policy](./MFA_SECURITY_POLICY.md)
- [Incident Response Playbook](./MFA_INCIDENT_RESPONSE.md)

**Terakhir diperbarui**: [Tanggal Update]
**Versi**: 1.0
