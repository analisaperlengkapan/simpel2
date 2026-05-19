# Panduan Administrator MFA SIMPEL

## Multi-Factor Authentication Management Guide

### Daftar Isi

1. [Pengenalan MFA untuk Administrator](#pengenalan-mfa-untuk-administrator)
2. [Manajemen Kebijakan MFA](#manajemen-kebijakan-mfa)
3. [Manajemen Pengguna MFA](#manajemen-pengguna-mfa)
4. [Monitoring dan Reporting](#monitoring-dan-reporting)
5. [Troubleshooting Administrator](#troubleshooting-administrator)
6. [Keamanan dan Compliance](#keamanan-dan-compliance)
7. [Operasional Harian](#operasional-harian)

---

## Pengenalan MFA untuk Administrator

### Arsitektur MFA SIMPEL

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Portal Web    │    │    Authenc      │    │    Secreton     │
│   (Frontend)    │◄──►│   (Auth Svc)    │◄──►│  (Vault Svc)    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         │              ┌─────────────────┐              │
         └──────────────►│   PostgreSQL    │◄─────────────┘
                        │   (Database)    │
                        └─────────────────┘
```

### Komponen MFA

- **Portal**: Interface pengguna untuk setup dan verifikasi MFA
- **Authenc**: Service autentikasi yang mengelola TOTP dan session
- **Secreton**: Vault service untuk penyimpanan secret terenkripsi
- **Database**: Penyimpanan metadata MFA dan audit logs

### Tanggung Jawab Administrator

1. **Kebijakan MFA**: Menentukan siapa yang wajib menggunakan MFA
2. **User Management**: Reset, disable, dan troubleshoot MFA pengguna
3. **Monitoring**: Memantau adopsi, keamanan, dan performa MFA
4. **Compliance**: Memastikan MFA memenuhi standar keamanan pemerintah
5. **Support**: Memberikan dukungan teknis kepada pengguna

---

## Manajemen Kebijakan MFA

### Konfigurasi Kebijakan Global

#### 1. Pengaturan Wajib MFA

```sql
-- Mengatur MFA wajib untuk semua pengguna
UPDATE system_config
SET config_value = 'true'
WHERE config_key = 'mfa_required_global';

-- Mengatur MFA wajib berdasarkan role
UPDATE system_config
SET config_value = 'admin,supervisor,auditor'
WHERE config_key = 'mfa_required_roles';

-- Mengatur MFA wajib berdasarkan satker
UPDATE system_config
SET config_value = 'PUSAT,KEJATI_DKI,KEJARI_JAKPUS'
WHERE config_key = 'mfa_required_satkers';
```

#### 2. Pengaturan Timeout dan Security

```sql
-- Session timeout setelah MFA (dalam menit)
UPDATE system_config
SET config_value = '480'
WHERE config_key = 'mfa_session_timeout';

-- Maksimal percobaan MFA gagal sebelum lockout
UPDATE system_config
SET config_value = '5'
WHERE config_key = 'mfa_max_attempts';

-- Durasi lockout akun (dalam menit)
UPDATE system_config
SET config_value = '15'
WHERE config_key = 'mfa_lockout_duration';

-- Grace period untuk setup MFA (dalam hari)
UPDATE system_config
SET config_value = '7'
WHERE config_key = 'mfa_setup_grace_period';
```

### Implementasi Kebijakan Bertahap

#### Phase 1: Pilot Program (Minggu 1-2)

```sql
-- Aktifkan MFA hanya untuk admin dan IT
UPDATE users
SET mfa_required = true
WHERE role IN ('admin', 'it_support', 'system_admin');
```

#### Phase 2: Management Level (Minggu 3-4)

```sql
-- Aktifkan untuk level supervisor ke atas
UPDATE users
SET mfa_required = true
WHERE jabatan LIKE '%kepala%'
   OR jabatan LIKE '%manager%'
   OR jabatan LIKE '%supervisor%';
```

#### Phase 3: Rollout Penuh (Minggu 5-8)

```sql
-- Aktifkan untuk semua pengguna
UPDATE users
SET mfa_required = true;
```

### Pengecualian dan Whitelist

#### Akun Service dan System

```sql
-- Daftar akun yang dikecualikan dari MFA
INSERT INTO mfa_exemptions (user_id, reason, approved_by, expires_at)
VALUES
  ('service-account-1', 'Automated system account', 'admin@kejaksaan.go.id', '2025-12-31'),
  ('backup-service', 'Backup automation', 'admin@kejaksaan.go.id', '2025-12-31');
```

#### Temporary Exemptions

```sql
-- Pengecualian sementara untuk situasi khusus
INSERT INTO mfa_exemptions (user_id, reason, approved_by, expires_at)
VALUES
  ('12345678', 'Ponsel rusak, menunggu penggantian', 'supervisor@kejaksaan.go.id', '2024-11-01');
```

---

## Manajemen Pengguna MFA

### Dashboard Admin MFA

#### Akses Admin Panel

```
URL: https://simipelv2.kejaksaan.go.id/admin/mfa
Akses: Hanya untuk role 'admin' dan 'mfa_admin'
```

### Operasi User Management

#### 1. Melihat Status MFA Pengguna

```sql
-- Query untuk melihat status MFA semua pengguna
SELECT
    u.nip,
    u.nama,
    u.satker_code,
    u.jabatan,
    u.mfa_enabled,
    u.mfa_setup_at,
    u.mfa_required,
    CASE
        WHEN u.mfa_required AND NOT u.mfa_enabled THEN 'Setup Required'
        WHEN u.mfa_enabled THEN 'Active'
        ELSE 'Not Required'
    END as mfa_status,
    ml.last_login_at,
    ml.last_mfa_verify_at
FROM users u
LEFT JOIN mfa_logs ml ON u.id = ml.user_id
    AND ml.created_at = (
        SELECT MAX(created_at)
        FROM mfa_logs
        WHERE user_id = u.id AND event_type = 'verify_success'
    )
ORDER BY u.satker_code, u.nama;
```

#### 2. Reset MFA Pengguna

```bash
# Via CLI tool
./scripts/cli/target/release/simipelv2-cli mfa reset --nip 12345678 --reason "Ponsel hilang"

# Via API endpoint
curl -X POST https://simipelv2.kejaksaan.go.id/api/admin/mfa/reset \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "nip": "12345678",
    "reason": "Ponsel hilang, setup ulang diperlukan",
    "notify_user": true
  }'
```

#### 3. Disable MFA Sementara

```sql
-- Disable MFA untuk pengguna tertentu
UPDATE users
SET mfa_required = false,
    mfa_disabled_reason = 'Temporary medical exemption',
    mfa_disabled_by = 'admin@kejaksaan.go.id',
    mfa_disabled_at = NOW()
WHERE nip = '12345678';

-- Log perubahan
INSERT INTO admin_actions (admin_id, action_type, target_user_id, details)
VALUES (
    (SELECT id FROM users WHERE email = 'admin@kejaksaan.go.id'),
    'mfa_disable',
    (SELECT id FROM users WHERE nip = '12345678'),
    'Temporary medical exemption - approved by medical officer'
);
```

#### 4. Generate Kode Cadangan Baru

```bash
# Generate backup codes untuk pengguna
./scripts/cli/target/release/simipelv2-cli mfa generate-backup-codes --nip 12345678

# Output akan berupa 10 kode 8-digit yang bisa diberikan ke pengguna
```

#### 5. Bulk Operations

```sql
-- Enable MFA untuk seluruh satker
UPDATE users
SET mfa_required = true
WHERE satker_code = 'KEJATI_DKI';

-- Disable MFA untuk role tertentu
UPDATE users
SET mfa_required = false,
    mfa_disabled_reason = 'Role exemption policy'
WHERE role = 'guest';
```

### User Support Workflows

#### Workflow 1: Pengguna Lupa/Hilang Ponsel

1. **Verifikasi Identitas**:
   - Minta KTP/ID pegawai
   - Konfirmasi data personal (nama, NIP, satker)
   - Verifikasi dengan atasan jika perlu

2. **Reset MFA**:

   ```bash
   ./scripts/cli/target/release/simipelv2-cli mfa reset --nip [NIP] --reason "Ponsel hilang"
   ```

3. **Generate Temporary Access**:

   ```bash
   ./scripts/cli/target/release/simipelv2-cli mfa temp-disable --nip [NIP] --duration 24h
   ```

4. **Follow-up**:
   - Instruksikan pengguna setup MFA baru
   - Monitor setup completion dalam 24 jam
   - Re-enable MFA requirement

#### Workflow 2: Masalah Teknis Authenticator

1. **Diagnosis**:
   - Periksa log error di sistem
   - Test dengan kode cadangan
   - Verifikasi sinkronisasi waktu

2. **Troubleshoot**:

   ```sql
   -- Periksa log MFA pengguna
   SELECT * FROM mfa_logs
   WHERE user_id = (SELECT id FROM users WHERE nip = '[NIP]')
   ORDER BY created_at DESC
   LIMIT 20;
   ```

3. **Solusi**:
   - Guide setup ulang authenticator
   - Atau reset MFA jika perlu

---

## Monitoring dan Reporting

### Dashboard Metrics

#### Key Performance Indicators (KPIs)

1. **Adoption Rate**: Persentase pengguna yang sudah setup MFA
2. **Success Rate**: Persentase verifikasi MFA yang berhasil
3. **Support Tickets**: Jumlah tiket terkait MFA
4. **Security Incidents**: Jumlah incident keamanan terkait MFA

#### Real-time Monitoring Queries

```sql
-- MFA Adoption Rate by Satker
SELECT
    satker_code,
    COUNT(*) as total_users,
    COUNT(CASE WHEN mfa_enabled THEN 1 END) as mfa_enabled_users,
    ROUND(
        COUNT(CASE WHEN mfa_enabled THEN 1 END) * 100.0 / COUNT(*),
        2
    ) as adoption_rate_percent
FROM users
WHERE active = true
GROUP BY satker_code
ORDER BY adoption_rate_percent DESC;

-- Daily MFA Verification Stats
SELECT
    DATE(created_at) as date,
    COUNT(CASE WHEN event_type = 'verify_success' THEN 1 END) as successful_verifications,
    COUNT(CASE WHEN event_type = 'verify_failed' THEN 1 END) as failed_verifications,
    COUNT(CASE WHEN event_type = 'account_locked' THEN 1 END) as account_lockouts
FROM mfa_logs
WHERE created_at >= CURRENT_DATE - INTERVAL '30 days'
GROUP BY DATE(created_at)
ORDER BY date DESC;

-- Top MFA Issues
SELECT
    error_type,
    COUNT(*) as occurrence_count,
    COUNT(DISTINCT user_id) as affected_users
FROM mfa_error_logs
WHERE created_at >= CURRENT_DATE - INTERVAL '7 days'
GROUP BY error_type
ORDER BY occurrence_count DESC;
```

### Automated Reports

#### Daily MFA Summary Report

```bash
#!/bin/bash
# Script: /scripts/reports/daily_mfa_report.sh

# Generate daily MFA report
./scripts/cli/target/release/simipelv2-cli reports mfa-daily \
  --output /var/reports/mfa_daily_$(date +%Y%m%d).json \
  --email admin@kejaksaan.go.id,security@kejaksaan.go.id
```

#### Weekly Security Report

```sql
-- Weekly security incidents related to MFA
SELECT
    'MFA Security Report - Week ' || EXTRACT(week FROM CURRENT_DATE) as report_title,
    COUNT(CASE WHEN event_type = 'suspicious_activity' THEN 1 END) as suspicious_activities,
    COUNT(CASE WHEN event_type = 'brute_force_attempt' THEN 1 END) as brute_force_attempts,
    COUNT(CASE WHEN event_type = 'account_locked' THEN 1 END) as account_lockouts,
    COUNT(DISTINCT user_id) as affected_users
FROM security_events
WHERE created_at >= CURRENT_DATE - INTERVAL '7 days'
  AND category = 'mfa';
```

### Alerting Configuration

#### Critical Alerts

```yaml
# /config/alerts/mfa_alerts.yml
alerts:
  - name: "High MFA Failure Rate"
    condition: "mfa_failure_rate > 20% in last 1 hour"
    severity: "critical"
    notification: ["admin@kejaksaan.go.id", "security@kejaksaan.go.id"]

  - name: "MFA Service Down"
    condition: "mfa_service_availability < 95% in last 5 minutes"
    severity: "critical"
    notification: ["oncall@kejaksaan.go.id"]

  - name: "Suspicious MFA Activity"
    condition: "failed_mfa_attempts > 100 from single IP in 10 minutes"
    severity: "high"
    notification: ["security@kejaksaan.go.id"]

  - name: "Low MFA Adoption"
    condition: "mfa_adoption_rate < 80% for any satker"
    severity: "medium"
    notification: ["admin@kejaksaan.go.id"]
```

---

## Troubleshooting Administrator

### Common Admin Issues

#### 1. MFA Service Performance Issues

**Symptoms**: Slow MFA verification, timeouts

**Diagnosis**:

```bash
# Check service health
curl -s https://simipelv2.kejaksaan.go.id/api/health/mfa | jq

# Check database performance
psql -h db-host -U admin -d simipelv2 -c "
SELECT
    schemaname,
    tablename,
    attname,
    n_distinct,
    correlation
FROM pg_stats
WHERE tablename IN ('users', 'mfa_logs', 'mfa_secrets');"

# Check Redis cache performance
redis-cli --latency-history -i 1
```

**Solutions**:

```sql
-- Add missing indexes
CREATE INDEX CONCURRENTLY idx_mfa_logs_user_created
ON mfa_logs(user_id, created_at DESC);

CREATE INDEX CONCURRENTLY idx_users_mfa_status
ON users(mfa_enabled, mfa_required)
WHERE active = true;

-- Analyze table statistics
ANALYZE users;
ANALYZE mfa_logs;
```

#### 2. High False Positive Rate

**Symptoms**: Valid codes being rejected

**Diagnosis**:

```sql
-- Check time synchronization issues
SELECT
    user_id,
    COUNT(*) as failed_attempts,
    AVG(time_skew_seconds) as avg_time_skew
FROM mfa_verification_attempts
WHERE result = 'time_sync_error'
  AND created_at >= CURRENT_DATE - INTERVAL '24 hours'
GROUP BY user_id
HAVING COUNT(*) > 5;
```

**Solutions**:

```bash
# Adjust time window tolerance
./scripts/cli/target/release/simipelv2-cli config set mfa.time_window_tolerance 2

# Sync NTP on all servers
sudo ntpdate -s time.nist.gov
sudo systemctl restart ntp
```

#### 3. Secreton Integration Issues

**Symptoms**: Cannot store/retrieve MFA secrets

**Diagnosis**:

```bash
# Test secreton connectivity
curl -H "Authorization: Bearer $AUTHENC_TOKEN" \
  https://secreton.internal/v1/health

# Check secreton logs
kubectl logs -f deployment/secreton -n simipelv2

# Test MFA secret operations
./scripts/cli/target/release/simipelv2-cli test secreton-mfa
```

**Solutions**:

```bash
# Restart secreton connection pool
kubectl rollout restart deployment/authenc -n simipelv2

# Check secreton MFA policies
./scripts/cli/target/release/simipelv2-cli secreton policy list --filter mfa
```

### Database Maintenance

#### Regular Maintenance Tasks

```sql
-- Clean up old MFA logs (keep 1 year)
DELETE FROM mfa_logs
WHERE created_at < CURRENT_DATE - INTERVAL '1 year';

-- Clean up expired temporary exemptions
DELETE FROM mfa_exemptions
WHERE expires_at < CURRENT_DATE;

-- Update statistics
ANALYZE mfa_logs;
ANALYZE users;

-- Reindex if needed
REINDEX INDEX CONCURRENTLY idx_mfa_logs_created_at;
```

#### Backup MFA Data

```bash
#!/bin/bash
# Backup MFA configuration and logs

# Backup MFA-related tables
pg_dump -h db-host -U backup_user -d simipelv2 \
  --table=users \
  --table=mfa_logs \
  --table=mfa_exemptions \
  --table=system_config \
  --where="config_key LIKE 'mfa_%'" \
  > /backups/mfa_backup_$(date +%Y%m%d).sql

# Backup secreton MFA secrets (encrypted)
./scripts/cli/target/release/simipelv2-cli secreton backup \
  --path mfa/ \
  --output /backups/mfa_secrets_$(date +%Y%m%d).enc
```

---

## Keamanan dan Compliance

### Security Best Practices

#### 1. Principle of Least Privilege

```sql
-- Create dedicated MFA admin role
CREATE ROLE mfa_admin;

-- Grant only necessary permissions
GRANT SELECT, UPDATE ON users TO mfa_admin;
GRANT SELECT, INSERT ON mfa_logs TO mfa_admin;
GRANT SELECT, INSERT, UPDATE, DELETE ON mfa_exemptions TO mfa_admin;

-- Assign role to specific admins
GRANT mfa_admin TO admin_user_1, admin_user_2;
```

#### 2. Admin Action Auditing

```sql
-- All admin actions must be logged
CREATE TABLE admin_mfa_actions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    admin_user_id UUID NOT NULL REFERENCES users(id),
    action_type VARCHAR(50) NOT NULL,
    target_user_id UUID REFERENCES users(id),
    details JSONB,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Trigger to log all MFA-related admin actions
CREATE OR REPLACE FUNCTION log_admin_mfa_action()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO admin_mfa_actions (
        admin_user_id, action_type, target_user_id, details
    ) VALUES (
        current_setting('app.current_user_id')::UUID,
        TG_ARGV[0],
        NEW.id,
        to_jsonb(NEW)
    );
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
```

#### 3. Compliance Monitoring

```sql
-- Compliance check queries
-- 1. Ensure all required users have MFA enabled
SELECT
    'MFA_COMPLIANCE_CHECK' as check_type,
    COUNT(*) as non_compliant_users,
    array_agg(nip) as non_compliant_nips
FROM users
WHERE mfa_required = true
  AND mfa_enabled = false
  AND active = true;

-- 2. Check for stale MFA setups (not used in 90 days)
SELECT
    'STALE_MFA_CHECK' as check_type,
    COUNT(*) as stale_accounts,
    array_agg(nip) as stale_nips
FROM users u
LEFT JOIN mfa_logs ml ON u.id = ml.user_id
WHERE u.mfa_enabled = true
  AND (ml.created_at IS NULL OR ml.created_at < CURRENT_DATE - INTERVAL '90 days');
```

### Government Compliance Requirements

#### Indonesian Government Security Standards

1. **Peraturan Menteri Komunikasi dan Informatika No. 4 Tahun 2016**
   - Multi-factor authentication wajib untuk sistem pemerintah
   - Audit trail lengkap untuk semua akses

2. **Surat Edaran Menteri PANRB No. 3 Tahun 2018**
   - Keamanan data pegawai dan sistem informasi
   - Backup dan recovery procedures

#### Compliance Checklist

- [ ] MFA enabled untuk semua akun privileged
- [ ] Audit logging untuk semua MFA events
- [ ] Regular security assessment dan penetration testing
- [ ] Backup dan disaster recovery procedures
- [ ] User training dan awareness program
- [ ] Incident response procedures
- [ ] Regular compliance reporting

### Security Incident Response

#### MFA-Related Security Incidents

**Level 1: Suspicious MFA Activity**

```bash
# Automated response for suspicious activity
./scripts/security/mfa_incident_response.sh --level 1 --user-id $USER_ID
```

**Level 2: Compromised MFA**

```bash
# Immediate lockdown procedures
./scripts/security/mfa_incident_response.sh --level 2 --user-id $USER_ID --lockdown
```

**Level 3: System-wide MFA Compromise**

```bash
# Emergency MFA reset for all users
./scripts/security/emergency_mfa_reset.sh --confirm-emergency
```

---

## Operasional Harian

### Daily Admin Tasks

#### Morning Checklist (08:00 - 09:00)

```bash
#!/bin/bash
# Daily morning MFA health check

echo "=== Daily MFA Health Check - $(date) ==="

# 1. Check MFA service status
echo "1. Checking MFA service health..."
curl -s https://simipelv2.kejaksaan.go.id/api/health/mfa

# 2. Check overnight MFA failures
echo "2. Checking overnight failures..."
./scripts/cli/target/release/simipelv2-cli reports mfa-failures --since "24 hours ago"

# 3. Check pending MFA setups
echo "3. Checking pending setups..."
./scripts/cli/target/release/simipelv2-cli reports mfa-pending-setups

# 4. Check locked accounts
echo "4. Checking locked accounts..."
./scripts/cli/target/release/simipelv2-cli reports mfa-locked-accounts

# 5. Check system alerts
echo "5. Checking system alerts..."
./scripts/cli/target/release/simipelv2-cli alerts list --category mfa --status active
```

#### Weekly Tasks (Setiap Senin)

```bash
#!/bin/bash
# Weekly MFA maintenance

# 1. Generate weekly report
./scripts/cli/target/release/simipelv2-cli reports mfa-weekly \
  --output /var/reports/mfa_weekly_$(date +%Y%W).pdf \
  --email management@kejaksaan.go.id

# 2. Clean up old logs
./scripts/maintenance/cleanup_mfa_logs.sh

# 3. Update MFA statistics
./scripts/maintenance/update_mfa_stats.sh

# 4. Check compliance status
./scripts/compliance/mfa_compliance_check.sh
```

#### Monthly Tasks (Tanggal 1 setiap bulan)

```bash
#!/bin/bash
# Monthly MFA review

# 1. Generate monthly compliance report
./scripts/cli/target/release/simipelv2-cli reports mfa-compliance \
  --month $(date +%Y-%m) \
  --output /var/reports/compliance/mfa_$(date +%Y%m).pdf

# 2. Review MFA exemptions
./scripts/cli/target/release/simipelv2-cli mfa review-exemptions

# 3. Update MFA policies if needed
./scripts/policy/review_mfa_policies.sh

# 4. Security assessment
./scripts/security/mfa_security_assessment.sh
```

### Emergency Procedures

#### Emergency Contact List

- **Primary Admin**: admin@kejaksaan.go.id / +62-812-1111-1111
- **Security Team**: security@kejaksaan.go.id / +62-812-2222-2222
- **On-Call Engineer**: oncall@kejaksaan.go.id / +62-812-3333-3333
- **Management**: management@kejaksaan.go.id

#### Emergency Response Procedures

**Scenario 1: MFA Service Outage**

1. Activate temporary MFA bypass (max 4 hours)
2. Notify all users via email/SMS
3. Escalate to engineering team
4. Document incident and resolution

**Scenario 2: Mass MFA Compromise**

1. Immediately disable all MFA tokens
2. Force password reset for all affected users
3. Activate incident response team
4. Coordinate with security team for forensics

**Scenario 3: Database Corruption**

1. Stop MFA service immediately
2. Restore from latest backup
3. Verify data integrity
4. Gradually re-enable service

---

**Kontak Support**:

- **Email**: admin-mfa@kejaksaan.go.id
- **Telepon**: (021) 123-4567 ext. 800
- **Emergency**: +62-812-9999-9999 (24/7)

**Dokumen Terkait**:

- [MFA Security Policy](./MFA_SECURITY_POLICY.md)
- [MFA API Documentation](./MFA_API_DOCUMENTATION.md)
- [Incident Response Playbook](./MFA_INCIDENT_RESPONSE.md)

**Terakhir diperbarui**: [Tanggal Update]
**Versi**: 1.0
**Pemilik Dokumen**: Tim IT Kejaksaan RI
