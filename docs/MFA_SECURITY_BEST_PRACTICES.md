# Panduan Best Practices Keamanan MFA SIMPEL

## Daftar Isi
1. [Prinsip Keamanan MFA](#prinsip-keamanan-mfa)
2. [Konfigurasi Keamanan](#konfigurasi-keamanan)
3. [Manajemen Kunci dan Secret](#manajemen-kunci-dan-secret)
4. [Monitoring dan Deteksi Ancaman](#monitoring-dan-deteksi-ancaman)
5. [Incident Response](#incident-response)
6. [Compliance dan Audit](#compliance-dan-audit)
7. [Deployment Security](#deployment-security)

---

## Prinsip Keamanan MFA

### Defense in Depth
MFA adalah bagian dari strategi keamanan berlapis:
- **Layer 1**: Network Security (Firewall, VPN)
- **Layer 2**: Application Security (WAF, Rate Limiting)
- **Layer 3**: Authentication (Password + MFA)
- **Layer 4**: Authorization (Role-based Access)
- **Layer 5**: Data Protection (Encryption at Rest/Transit)

### Zero Trust Architecture
- Tidak ada trust implisit berdasarkan lokasi network
- Setiap request harus diverifikasi dan diotorisasi
- Continuous monitoring dan validation
- Principle of least privilege

### Cryptographic Standards
- **TOTP Algorithm**: RFC 6238 compliant
- **Hash Function**: HMAC-SHA1 (minimum), SHA-256 (recommended)
- **Secret Length**: Minimum 160 bits (20 bytes)
- **Time Window**: 30 seconds standard
- **Clock Skew Tolerance**: ±1 time window (30 seconds)

---

## Konfigurasi Keamanan

### Secure TOTP Configuration
```toml
# /config/mfa_security.toml
[totp]
algorithm = "SHA256"  # Use SHA-256 instead of SHA-1
digits = 6           # Standard 6-digit codes
period = 30          # 30-second time window
skew = 1            # Allow ±1 time window for clock skew
secret_length = 32   # 32 bytes (256 bits) for stronger secrets

[rate_limiting]
max_attempts_per_minute = 5
max_attempts_per_hour = 20
lockout_duration_minutes = 15
progressive_delay = true

[session_security]
mfa_session_timeout_minutes = 480  # 8 hours
require_mfa_for_sensitive_operations = true
invalidate_sessions_on_mfa_reset = true
```
### Database Security
```sql
-- Encrypt MFA secrets at database level
ALTER TABLE mfa_secrets
ADD COLUMN encrypted_secret BYTEA,
ADD COLUMN encryption_key_id UUID;

-- Create audit triggers for MFA tables
CREATE OR REPLACE FUNCTION audit_mfa_changes()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO mfa_audit_log (
        table_name, operation, old_values, new_values,
        changed_by, changed_at
    ) VALUES (
        TG_TABLE_NAME, TG_OP,
        row_to_json(OLD), row_to_json(NEW),
        current_setting('app.current_user_id'), NOW()
    );
    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

-- Apply audit triggers
CREATE TRIGGER mfa_secrets_audit
    AFTER INSERT OR UPDATE OR DELETE ON mfa_secrets
    FOR EACH ROW EXECUTE FUNCTION audit_mfa_changes();
```

### Network Security
```nginx
# /etc/nginx/conf.d/mfa_security.conf
# Rate limiting for MFA endpoints
limit_req_zone $binary_remote_addr zone=mfa_setup:10m rate=1r/m;
limit_req_zone $binary_remote_addr zone=mfa_verify:10m rate=10r/m;

location /api/auth/mfa/setup {
    limit_req zone=mfa_setup burst=2 nodelay;
    # Additional security headers
    add_header X-Frame-Options DENY;
    add_header X-Content-Type-Options nosniff;
    add_header Referrer-Policy strict-origin-when-cross-origin;
}

location /api/auth/mfa/verify {
    limit_req zone=mfa_verify burst=5 nodelay;
    # Block suspicious user agents
    if ($http_user_agent ~* (bot|crawler|spider)) {
        return 403;
    }
}
```

---

## Manajemen Kunci dan Secret

### Secret Generation
```rust
// Secure secret generation
use rand::rngs::OsRng;
use rand::RngCore;

pub fn generate_secure_totp_secret() -> Result<Vec<u8>, CryptoError> {
    let mut secret = vec![0u8; 32]; // 256-bit secret
    OsRng.fill_bytes(&mut secret);

    // Validate entropy
    if is_low_entropy(&secret) {
        return Err(CryptoError::InsufficientEntropy);
    }

    Ok(secret)
}

fn is_low_entropy(data: &[u8]) -> bool {
    // Check for patterns that indicate low entropy
    let unique_bytes = data.iter().collect::<std::collections::HashSet<_>>();
    unique_bytes.len() < data.len() / 2
}
```
### Key Rotation Strategy
```bash
#!/bin/bash
# Script: /scripts/security/mfa_key_rotation.sh

# Rotate MFA encryption keys quarterly
ROTATION_SCHEDULE="0 2 1 */3 *"  # 02:00 on 1st day of every 3rd month

rotate_mfa_keys() {
    echo "Starting MFA key rotation at $(date)"

    # 1. Generate new encryption key in Secreton
    NEW_KEY_ID=$(./scripts/cli/target/release/simipelv2-cli secreton generate-key \
        --purpose mfa_encryption \
        --algorithm AES-256-GCM)

    # 2. Re-encrypt all MFA secrets with new key
    ./scripts/cli/target/release/simipelv2-cli mfa re-encrypt-secrets \
        --new-key-id $NEW_KEY_ID \
        --batch-size 100

    # 3. Update key reference in configuration
    ./scripts/cli/target/release/simipelv2-cli config set \
        mfa.encryption_key_id $NEW_KEY_ID

    # 4. Schedule old key for deletion (after 30 days)
    ./scripts/cli/target/release/simipelv2-cli secreton schedule-key-deletion \
        --key-id $OLD_KEY_ID \
        --delay-days 30

    echo "MFA key rotation completed at $(date)"
}

# Backup Codes Security
generate_secure_backup_codes() {
    local user_id=$1
    local codes=()

    for i in {1..10}; do
        # Generate cryptographically secure 8-digit codes
        local code=$(openssl rand -hex 4 | tr '[:lower:]' '[:upper:]')
        codes+=($code)
    done

    # Hash codes before storage
    local hashed_codes=()
    for code in "${codes[@]}"; do
        local hash=$(echo -n "$code" | sha256sum | cut -d' ' -f1)
        hashed_codes+=($hash)
    done

    # Store hashed codes in Secreton
    ./scripts/cli/target/release/simipelv2-cli secreton store \
        --path "mfa/backup_codes/$user_id" \
        --data "$(printf '%s\n' "${hashed_codes[@]}" | jq -R . | jq -s .)"

    # Return plaintext codes to admin (one-time display)
    printf '%s\n' "${codes[@]}"
}
```

### Secure Storage Architecture
```yaml
# Secreton MFA Policy Configuration
apiVersion: v1
kind: SecretonPolicy
metadata:
  name: mfa-secrets-policy
spec:
  path: "mfa/*"
  capabilities:
    - read
    - create
    - update
    - delete
  conditions:
    - type: "service_account"
      values: ["authenc-service"]
    - type: "ip_whitelist"
      values: ["10.0.0.0/8", "172.16.0.0/12"]
  encryption:
    algorithm: "AES-256-GCM"
    key_rotation_days: 90
  audit:
    enabled: true
    log_level: "detailed"
```
---

## Monitoring dan Deteksi Ancaman

### Security Event Detection
```sql
-- Real-time threat detection queries
CREATE OR REPLACE FUNCTION detect_mfa_threats()
RETURNS TABLE(threat_type TEXT, user_id UUID, details JSONB) AS $$
BEGIN
    -- Brute force detection
    RETURN QUERY
    SELECT
        'BRUTE_FORCE'::TEXT,
        ml.user_id,
        json_build_object(
            'failed_attempts', COUNT(*),
            'time_window', '10 minutes',
            'ip_addresses', array_agg(DISTINCT ml.ip_address)
        )::JSONB
    FROM mfa_logs ml
    WHERE ml.event_type = 'verify_failed'
        AND ml.created_at >= NOW() - INTERVAL '10 minutes'
    GROUP BY ml.user_id
    HAVING COUNT(*) >= 5;

    -- Impossible travel detection
    RETURN QUERY
    WITH location_changes AS (
        SELECT
            ml.user_id,
            ml.created_at,
            ig.country,
            ig.city,
            LAG(ig.country) OVER (PARTITION BY ml.user_id ORDER BY ml.created_at) as prev_country,
            LAG(ml.created_at) OVER (PARTITION BY ml.user_id ORDER BY ml.created_at) as prev_time
        FROM mfa_logs ml
        JOIN ip_geolocation ig ON ml.ip_address = ig.ip_address
        WHERE ml.event_type = 'verify_success'
            AND ml.created_at >= NOW() - INTERVAL '1 hour'
    )
    SELECT
        'IMPOSSIBLE_TRAVEL'::TEXT,
        lc.user_id,
        json_build_object(
            'from_country', lc.prev_country,
            'to_country', lc.country,
            'time_diff_minutes', EXTRACT(EPOCH FROM (lc.created_at - lc.prev_time))/60
        )::JSONB
    FROM location_changes lc
    WHERE lc.prev_country IS NOT NULL
        AND lc.country != lc.prev_country
        AND EXTRACT(EPOCH FROM (lc.created_at - lc.prev_time))/60 < 60; -- Less than 1 hour
END;
$$ LANGUAGE plpgsql;
```

### Automated Response System
```bash
#!/bin/bash
# Script: /scripts/security/automated_mfa_response.sh

handle_security_event() {
    local event_type=$1
    local user_id=$2
    local details=$3

    case $event_type in
        "BRUTE_FORCE")
            # Immediate account lockout
            ./scripts/cli/target/release/simipelv2-cli mfa lock-account \
                --user-id $user_id \
                --duration 30m \
                --reason "Brute force attack detected"

            # Block IP addresses
            local ips=$(echo $details | jq -r '.ip_addresses[]')
            for ip in $ips; do
                ./scripts/security/block_ip.sh $ip "MFA brute force"
            done

            # Alert security team
            ./scripts/alerts/security_alert.sh \
                "CRITICAL: MFA brute force attack on user $user_id"
            ;;

        "IMPOSSIBLE_TRAVEL")
            # Require additional verification
            ./scripts/cli/target/release/simipelv2-cli mfa require-step-up \
                --user-id $user_id \
                --reason "Suspicious geographic activity"

            # Flag for manual review
            ./scripts/cli/target/release/simipelv2-cli security flag-user \
                --user-id $user_id \
                --priority high \
                --reason "Impossible travel detected"
            ;;
    esac
}
```
---

## Incident Response

### MFA Security Incident Playbook

#### Phase 1: Detection and Analysis
```bash
# Incident detection checklist
INCIDENT_ID=$(date +%Y%m%d_%H%M%S)_MFA
INCIDENT_LOG="/var/log/security/mfa_incident_${INCIDENT_ID}.log"

echo "=== MFA Security Incident Response ===" | tee $INCIDENT_LOG
echo "Incident ID: $INCIDENT_ID" | tee -a $INCIDENT_LOG
echo "Start Time: $(date)" | tee -a $INCIDENT_LOG

# 1. Assess scope of incident
./scripts/security/assess_mfa_incident.sh --incident-id $INCIDENT_ID

# 2. Collect evidence
./scripts/security/collect_mfa_evidence.sh --incident-id $INCIDENT_ID

# 3. Determine impact level
IMPACT_LEVEL=$(./scripts/security/determine_impact.sh --type mfa)
echo "Impact Level: $IMPACT_LEVEL" | tee -a $INCIDENT_LOG
```

#### Phase 2: Containment
```bash
# Containment procedures based on impact level
case $IMPACT_LEVEL in
    "CRITICAL")
        # System-wide MFA emergency procedures
        ./scripts/security/emergency_mfa_lockdown.sh
        # Notify executive team
        ./scripts/alerts/executive_alert.sh "CRITICAL MFA security incident"
        ;;
    "HIGH")
        # Targeted containment
        ./scripts/security/targeted_mfa_containment.sh --incident-id $INCIDENT_ID
        # Notify security team
        ./scripts/alerts/security_team_alert.sh "HIGH severity MFA incident"
        ;;
    "MEDIUM"|"LOW")
        # Standard containment procedures
        ./scripts/security/standard_mfa_containment.sh --incident-id $INCIDENT_ID
        ;;
esac
```

#### Phase 3: Eradication and Recovery
```bash
# Eradication procedures
eradicate_mfa_threat() {
    local incident_id=$1

    # 1. Remove compromised secrets
    ./scripts/security/revoke_compromised_mfa_secrets.sh --incident-id $incident_id

    # 2. Force MFA re-enrollment for affected users
    ./scripts/security/force_mfa_reenrollment.sh --incident-id $incident_id

    # 3. Update security controls
    ./scripts/security/update_mfa_security_controls.sh

    # 4. Patch vulnerabilities
    ./scripts/security/apply_mfa_security_patches.sh
}

# Recovery procedures
recover_mfa_services() {
    # 1. Gradual service restoration
    ./scripts/recovery/gradual_mfa_restore.sh

    # 2. Monitor for anomalies
    ./scripts/monitoring/enhanced_mfa_monitoring.sh --duration 72h

    # 3. Validate security controls
    ./scripts/security/validate_mfa_controls.sh
}
```

### Communication Plan
```yaml
# Incident communication matrix
communication_plan:
  critical_incidents:
    - role: "Executive Team"
      contact: "executive@kejaksaan.go.id"
      notification_time: "Immediate (< 15 minutes)"
      method: ["email", "sms", "phone"]

    - role: "Security Team"
      contact: "security@kejaksaan.go.id"
      notification_time: "Immediate (< 5 minutes)"
      method: ["slack", "email", "sms"]

    - role: "IT Operations"
      contact: "ops@kejaksaan.go.id"
      notification_time: "Immediate (< 10 minutes)"
      method: ["slack", "email"]

  high_incidents:
    - role: "Security Team"
      notification_time: "< 30 minutes"
    - role: "IT Management"
      notification_time: "< 1 hour"
```
---

## Compliance dan Audit

### Regulatory Compliance Framework

#### Indonesian Government Standards
```yaml
# Compliance mapping for Indonesian regulations
compliance_standards:
  permenkominfo_4_2016:
    requirements:
      - "Multi-factor authentication for government systems"
      - "Audit trail for all authentication events"
      - "Secure storage of authentication credentials"
    implementation:
      - mfa_mandatory: true
      - audit_logging: comprehensive
      - encryption: AES-256-GCM

  se_menpanrb_3_2018:
    requirements:
      - "Data security for government employee information"
      - "Backup and recovery procedures"
      - "Regular security assessments"
    implementation:
      - data_encryption: at_rest_and_transit
      - backup_frequency: daily
      - security_assessment: quarterly
```

#### Audit Requirements
```sql
-- Comprehensive audit logging for compliance
CREATE TABLE mfa_compliance_audit (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    audit_date DATE NOT NULL,
    compliance_standard VARCHAR(50) NOT NULL,
    requirement_id VARCHAR(100) NOT NULL,
    status VARCHAR(20) NOT NULL, -- COMPLIANT, NON_COMPLIANT, PARTIAL
    evidence JSONB,
    auditor_id UUID REFERENCES users(id),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Daily compliance check
INSERT INTO mfa_compliance_audit (
    audit_date, compliance_standard, requirement_id, status, evidence
)
SELECT
    CURRENT_DATE,
    'PERMENKOMINFO_4_2016',
    'MFA_MANDATORY',
    CASE
        WHEN mfa_adoption_rate >= 95 THEN 'COMPLIANT'
        WHEN mfa_adoption_rate >= 80 THEN 'PARTIAL'
        ELSE 'NON_COMPLIANT'
    END,
    json_build_object(
        'adoption_rate', mfa_adoption_rate,
        'total_users', total_users,
        'mfa_enabled_users', mfa_enabled_users
    )
FROM (
    SELECT
        COUNT(*) as total_users,
        COUNT(CASE WHEN mfa_enabled THEN 1 END) as mfa_enabled_users,
        ROUND(COUNT(CASE WHEN mfa_enabled THEN 1 END) * 100.0 / COUNT(*), 2) as mfa_adoption_rate
    FROM users WHERE active = true
) stats;
```

### Security Assessment Procedures
```bash
#!/bin/bash
# Quarterly MFA security assessment

conduct_mfa_security_assessment() {
    local assessment_id="MFA_ASSESS_$(date +%Y%m%d)"
    local report_file="/var/reports/security/mfa_assessment_${assessment_id}.pdf"

    echo "Starting MFA Security Assessment: $assessment_id"

    # 1. Vulnerability scanning
    ./scripts/security/mfa_vulnerability_scan.sh --output-format json

    # 2. Penetration testing
    ./scripts/security/mfa_pentest.sh --scope limited --output-format json

    # 3. Configuration review
    ./scripts/security/mfa_config_review.sh --baseline security_baseline.yml

    # 4. Compliance validation
    ./scripts/compliance/mfa_compliance_check.sh --standards all

    # 5. Generate comprehensive report
    ./scripts/reports/generate_security_assessment.sh \
        --assessment-id $assessment_id \
        --output $report_file \
        --format pdf

    # 6. Submit to management
    ./scripts/reports/submit_assessment.sh \
        --file $report_file \
        --recipients "security@kejaksaan.go.id,management@kejaksaan.go.id"
}
```

---

## Deployment Security

### Secure Deployment Pipeline
```yaml
# CI/CD Security Gates for MFA components
security_gates:
  code_analysis:
    - static_analysis: "SonarQube with security rules"
    - dependency_check: "OWASP Dependency Check"
    - secret_scanning: "GitLeaks"

  security_testing:
    - unit_tests: "Security-focused unit tests"
    - integration_tests: "MFA security integration tests"
    - penetration_tests: "Automated security tests"

  deployment_validation:
    - configuration_check: "Validate security configurations"
    - certificate_validation: "Check TLS certificates"
    - access_control_test: "Validate RBAC settings"
```
### Production Security Hardening
```bash
#!/bin/bash
# Production MFA security hardening script

harden_mfa_production() {
    echo "Applying MFA production security hardening..."

    # 1. File system permissions
    chmod 600 /etc/simipelv2/mfa_config.toml
    chown simipelv2:simipelv2 /etc/simipelv2/mfa_config.toml

    # 2. Database security
    psql -c "REVOKE ALL ON mfa_secrets FROM PUBLIC;"
    psql -c "GRANT SELECT, INSERT, UPDATE ON mfa_secrets TO mfa_service_user;"

    # 3. Network security
    ufw allow from 10.0.0.0/8 to any port 8080 comment "MFA service internal"
    ufw deny 8080 comment "Block external MFA service access"

    # 4. Service security
    systemctl edit simipelv2-mfa --full << EOF
[Service]
User=simipelv2
Group=simipelv2
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/simipelv2
PrivateTmp=true
EOF

    # 5. Log security
    chmod 640 /var/log/simipelv2/mfa.log
    chown simipelv2:adm /var/log/simipelv2/mfa.log

    echo "MFA production hardening completed"
}

# Container security (if using Docker/Kubernetes)
apply_container_security() {
    # Security context for MFA pods
    kubectl patch deployment mfa-service -p '{
        "spec": {
            "template": {
                "spec": {
                    "securityContext": {
                        "runAsNonRoot": true,
                        "runAsUser": 1000,
                        "fsGroup": 1000
                    },
                    "containers": [{
                        "name": "mfa-service",
                        "securityContext": {
                            "allowPrivilegeEscalation": false,
                            "readOnlyRootFilesystem": true,
                            "capabilities": {
                                "drop": ["ALL"]
                            }
                        }
                    }]
                }
            }
        }
    }'
}
```

### Security Monitoring in Production
```yaml
# Prometheus alerting rules for MFA security
groups:
  - name: mfa_security_alerts
    rules:
      - alert: MFASecurityBreach
        expr: |
          (
            rate(mfa_failed_attempts_total[5m]) > 10
          ) or (
            increase(mfa_account_lockouts_total[10m]) > 20
          )
        for: 1m
        labels:
          severity: critical
          service: mfa
        annotations:
          summary: "Potential MFA security breach detected"
          description: "High rate of MFA failures or account lockouts detected"

      - alert: MFAServiceCompromise
        expr: |
          (
            rate(mfa_admin_actions_total{action="reset"}[1h]) > 5
          ) or (
            mfa_unusual_geographic_activity > 0
          )
        for: 0m
        labels:
          severity: high
          service: mfa
        annotations:
          summary: "Potential MFA service compromise"
          description: "Unusual administrative activity or geographic patterns detected"
```

---

**Kontak Security Team**:
- **Email**: security@kejaksaan.go.id
- **Emergency**: +62-812-SECURITY (24/7)
- **Incident Response**: incident-response@kejaksaan.go.id

**Dokumen Terkait**:
- [MFA Admin Guide](./MFA_ADMIN_GUIDE.md)
- [MFA Incident Response Playbook](./MFA_INCIDENT_RESPONSE.md)
- [MFA Compliance Documentation](./MFA_COMPLIANCE.md)

**Security Standards**:
- ISO 27001:2013
- NIST Cybersecurity Framework
- Indonesian Government Security Guidelines

**Terakhir diperbarui**: [Tanggal Update]
**Versi**: 1.0
**Classification**: CONFIDENTIAL
