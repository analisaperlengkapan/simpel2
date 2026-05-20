# Panduan Administrator SIMPEL

## Daftar Isi

1. [Pengenalan](#pengenalan)
2. [Manajemen Pengguna](#manajemen-pengguna)
3. [Konfigurasi Sistem](#konfigurasi-sistem)
4. [Master Data](#master-data)
5. [Workflow Management](#workflow-management)
6. [Monitoring dan Logging](#monitoring-dan-logging)
7. [Backup dan Recovery](#backup-dan-recovery)
8. [Troubleshooting](#troubleshooting)

---

## Pengenalan

**Peran Administrator:**

- Manajemen pengguna dan role
- Konfigurasi sistem
- Maintenance master data
- Monitoring sistem
- Troubleshooting

**Akses:**

- Dashboard admin: https://simpel.kejaksaan.go.id/admin
- Login dengan akun admin

---

## Manajemen Pengguna

### Menambah Pengguna Baru

**Langkah:**

1. **Buka Menu User Management**
   - Menu **"Admin"** → **"User Management"**
   - Klik **"Tambah Pengguna"**

2. **Input Data Pengguna**
   - NIP (wajib, unique)
   - Nama lengkap
   - Email
   - Nomor telepon
   - Satker

3. **Assign Role**
   - Pilih role:
     - **Super Admin**: Full access
     - **Admin Pusat**: Validator Pusat
     - **Admin Wilayah**: Validator Wilayah
     - **Operator Satker**: Input data satker
     - **Verifikator**: Review dan approval
     - **Auditor**: Read-only access

4. **Set Password**
   - Generate password otomatis
   - Atau set manual
   - Kirim email notifikasi ke pengguna

5. **Aktivasi**
   - Klik **"Simpan dan Aktivasi"**
   - Pengguna dapat login

### Mengelola Role dan Permission

**Role-Based Access Control (RBAC):**

| Role | Permissions |
|------|-------------|
| Super Admin | Full access ke semua modul |
| Admin Pusat | Inisiasi periode, review pusat, laporan nasional |
| Admin Wilayah | Review wilayah, monitoring wilayah |
| Operator Satker | Input data, submit pengajuan |
| Verifikator | Review dan approval |
| Auditor | Read-only, audit trail |

**Mengubah Role:**

1. Buka **"User Management"**
2. Cari pengguna
3. Klik **"Edit"**
4. Ubah role
5. Klik **"Simpan"**

### Reset Password

**Langkah:**

1. Buka **"User Management"**
2. Cari pengguna
3. Klik **"Reset Password"**
4. Pilih metode:
   - **Generate otomatis**: Sistem generate dan kirim email
   - **Set manual**: Admin set password baru
5. Klik **"Reset"**

### Menonaktifkan Pengguna

**Langkah:**

1. Buka **"User Management"**
2. Cari pengguna
3. Klik **"Nonaktifkan"**
4. Konfirmasi
5. Pengguna tidak dapat login

**Catatan:** Data pengguna tidak dihapus, hanya dinonaktifkan.

---

## Konfigurasi Sistem

### Konfigurasi Umum

**Akses:** Menu **"Admin"** → **"System Configuration"**

**Parameter:**

1. **Application Settings**
   - Application name
   - Base URL
   - Timezone (Asia/Jakarta)
   - Default language (Bahasa Indonesia)

2. **Email Settings**
   - SMTP server
   - SMTP port
   - SMTP username
   - SMTP password
   - From email
   - From name

3. **File Upload Settings**
   - Max file size (default: 10MB)
   - Allowed file types
   - Storage path

4. **Session Settings**
   - Session timeout (default: 30 minutes)
   - Remember me duration (default: 7 days)

### Konfigurasi Integrasi

**SIMAN Integration:**

1. Buka **"Admin"** → **"Integration Settings"** → **"SIMAN"**
2. Input:
   - API endpoint
   - API key (dari Secreton)
   - Sync schedule (cron expression)
   - Retry policy
3. Test connection
4. Klik **"Simpan"**

**MySIMKARI Integration:**

1. Buka **"Admin"** → **"Integration Settings"** → **"MySIMKARI"**
2. Input:
   - API endpoint
   - API key (dari Secreton)
   - Sync schedule
   - Retry policy
3. Test connection
4. Klik **"Simpan"**

**Sync Schedule:**

- **Full sync**: Daily at 02:00 WIB
- **Incremental sync**: Every 6 hours
- **On-demand sync**: Manual trigger

### Konfigurasi Notifikasi

**Akses:** Menu **"Admin"** → **"Notification Settings"**

**Channel:**

1. **In-App Notification**
   - Enabled by default
   - Real-time via WebSocket

2. **Email Notification**
   - SMTP configuration (lihat Email Settings)
   - Email templates

3. **SMS Notification** (optional)
   - SMS gateway configuration
   - SMS templates

**Notification Rules:**

| Event | Recipients | Channel |
|-------|-----------|---------|
| Pengajuan submitted | Validator | In-App, Email |
| Pengajuan approved | Operator | In-App, Email |
| Pengajuan rejected | Operator | In-App, Email |
| Izin akan berakhir (H-30) | Pegawai | In-App, Email |
| Izin akan berakhir (H-14) | Pegawai | In-App, Email, SMS |
| Izin akan berakhir (H-7) | Pegawai | In-App, Email, SMS |
| SLA breach | Admin | In-App, Email |

---

## Master Data

### Kode Barang

**Akses:** Menu **"Admin"** → **"Master Data"** → **"Kode Barang"**

**Menambah Kode Barang:**

1. Klik **"Tambah Kode Barang"**
2. Input:
   - Kode barang (format BMN: X.X.XX.XX.XXX)
   - Nama barang
   - Kategori
   - Is SBSK (Ya/Tidak)
   - Deskripsi
3. Klik **"Simpan"**

**Import dari Excel:**

1. Download template Excel
2. Isi data sesuai template
3. Klik **"Import"**
4. Upload file Excel
5. Review preview
6. Klik **"Import"**

### Standar Spesifikasi

**Akses:** Menu **"Admin"** → **"Master Data"** → **"Standar Spesifikasi"**

**Menambah Standar Spesifikasi:**

1. Klik **"Tambah Standar"**
2. Input:
   - Kode barang (pilih dari dropdown)
   - Tahun berlaku
   - Spesifikasi (JSONB format)
   - Contoh:

     ```json
     {
       "processor": "Intel Core i5",
       "ram": "8GB",
       "storage": "256GB SSD",
       "display": "14 inch"
     }
     ```

3. Klik **"Simpan"**

**Versioning:**

- Setiap perubahan membuat versi baru
- Versi lama tetap tersimpan untuk audit

### Standar Jumlah

**Akses:** Menu **"Admin"** → **"Master Data"** → **"Standar Jumlah"**

**Menambah Standar Jumlah:**

1. Klik **"Tambah Standar"**
2. Input:
   - Kode barang
   - Tahun berlaku
   - Tipe perhitungan:
     - **Per pegawai**: Jumlah = pegawai × rasio
     - **Per satker**: Jumlah tetap per satker
     - **Per eselon**: Berbeda per eselon
   - Rasio/jumlah
3. Klik **"Simpan"**

**Contoh:**

- Laptop: 1 per pegawai eselon III ke atas
- Kendaraan: 1 per satker + 1 per eselon II
- Printer: 1 per 10 pegawai

### Mapping Kodefikasi

**Akses:** Menu **"Admin"** → **"Master Data"** → **"Mapping Kodefikasi"**

**Fungsi:**

- Mapping kode barang non-standar dari MonSAKTI ke kode standar

**Proses:**

1. **Auto-detect Non-Standard Codes**
   - Sistem otomatis deteksi kode non-standar
   - Tampil di dashboard mapping

2. **Propose Mapping**
   - Operator propose mapping
   - Contoh: "3.1.01.01.999" → "3.1.01.01.001"

3. **Verify Mapping**
   - Admin verify dan approve
   - Klik **"Approve"**

4. **Apply Mapping**
   - Mapping diterapkan ke data SIMAN
   - Sinkronisasi otomatis

---

## Workflow Management

### Konfigurasi Workflow

**Akses:** Menu **"Admin"** → **"Workflow Management"**

**Workflow yang Tersedia:**

1. **Kebutuhan BMN Workflow**
   - DRAFT → SUBMITTED → REVIEWED_WILAYAH → REVIEWED_PUSAT → APPROVED/REJECTED

2. **Pakaian Dinas Workflow**
   - DRAFT → SUBMITTED → APPROVED_KEJARI → APPROVED_KEJATI → APPROVED_KEJAGUNG

3. **Pemakaian BMN Workflow**
   - DRAFT → DOCUMENT_GENERATED → SIGNED → ACTIVE → EXPIRED/REVOKED

4. **Penghapusan BMN Workflow**
   - DRAFT → SUBMITTED → REVIEWED_WILAYAH → REVIEWED_PUSAT → SK_GENERATED → COMPLETED

**Mengubah Workflow:**

1. Pilih workflow
2. Klik **"Edit"**
3. Ubah:
   - Step name
   - Role yang berwenang
   - SLA (Service Level Agreement)
   - Auto-escalation rules
4. Klik **"Simpan"**

### SLA Configuration

**Akses:** Menu **"Admin"** → **"Workflow Management"** → **"SLA Configuration"**

**Set SLA per Step:**

| Workflow Step | SLA | Auto-Escalation |
|---------------|-----|-----------------|
| Review Wilayah | 3 hari | Ke Admin Wilayah |
| Review Pusat | 5 hari | Ke Admin Pusat |
| Generate Document | 1 hari | Ke Admin |

**Auto-Escalation:**

- Jika SLA breach, sistem otomatis escalate
- Notifikasi ke level lebih tinggi
- Log di audit trail

### Monitoring Workflow

**Akses:** Menu **"Admin"** → **"Workflow Monitoring"**

**Metrics:**

1. **Workflow Performance**
   - Average completion time
   - SLA compliance rate
   - Bottleneck analysis

2. **Workflow Status**
   - Total instances per status
   - Pending instances
   - Completed instances

3. **SLA Breach**
   - List of breached instances
   - Breach reason
   - Escalation history

**Export:**

- Download report (Excel, PDF)
- Schedule automated reports

---

## Monitoring dan Logging

### System Monitoring

**Akses:** Menu **"Admin"** → **"System Monitoring"**

**Dashboard:**

1. **System Health**
   - CPU usage
   - Memory usage
   - Disk usage
   - Network traffic

2. **Application Metrics**
   - Active users
   - Request per second
   - Response time
   - Error rate

3. **Database Metrics**
   - Connection pool
   - Query performance
   - Slow queries

4. **Integration Health**
   - SIMAN API status
   - MySIMKARI API status
   - Last sync time
   - Sync errors

**Alerts:**

- Email alert jika CPU > 80%
- Email alert jika disk > 90%
- Email alert jika error rate > 5%

### Audit Logging

**Akses:** Menu **"Admin"** → **"Audit Log"**

**Log Events:**

| Event Type | Description |
|------------|-------------|
| USER_LOGIN | User login |
| USER_LOGOUT | User logout |
| USER_CREATED | New user created |
| USER_UPDATED | User data updated |
| ROLE_CHANGED | User role changed |
| PASSWORD_RESET | Password reset |
| PENGAJUAN_CREATED | Pengajuan created |
| PENGAJUAN_SUBMITTED | Pengajuan submitted |
| PENGAJUAN_APPROVED | Pengajuan approved |
| PENGAJUAN_REJECTED | Pengajuan rejected |
| WORKFLOW_TRANSITION | Workflow state changed |
| DOCUMENT_GENERATED | Document generated |
| DOCUMENT_UPLOADED | Document uploaded |
| MASTER_DATA_CHANGED | Master data changed |
| CONFIG_CHANGED | System config changed |

**Filter:**

- By user
- By event type
- By date range
- By IP address

**Export:**

- Download audit log (CSV, Excel)
- For compliance and audit purposes

### Error Logging

**Akses:** Menu **"Admin"** → **"Error Log"**

**Error Levels:**

- **CRITICAL**: System down
- **ERROR**: Operation failed
- **WARNING**: Potential issue
- **INFO**: Informational

**Error Details:**

- Timestamp
- Error message
- Stack trace
- User context
- Request details

**Actions:**

- View error details
- Mark as resolved
- Add notes
- Export for debugging

---

## Backup dan Recovery

### Database Backup

**Akses:** Menu **"Admin"** → **"Backup & Recovery"** → **"Database Backup"**

**Automated Backup:**

- **Full backup**: Daily at 01:00 WIB
- **Incremental backup**: Every 6 hours
- **Retention**: 30 days

**Manual Backup:**

1. Klik **"Create Backup Now"**
2. Pilih tipe:
   - **Full backup**: Semua data
   - **Partial backup**: Pilih schema
3. Klik **"Start Backup"**
4. Download backup file

**Backup Location:**

- Primary: Local storage
- Secondary: S3-compatible storage (MinIO)
- Tertiary: Off-site backup

### File Backup

**Akses:** Menu **"Admin"** → **"Backup & Recovery"** → **"File Backup"**

**Files to Backup:**

- Uploaded documents
- Generated documents
- System configuration files

**Backup Schedule:**

- Daily at 02:00 WIB
- Retention: 90 days

### Recovery

**Database Recovery:**

1. Buka **"Backup & Recovery"** → **"Database Recovery"**
2. Pilih backup point
3. Preview backup details
4. Klik **"Restore"**
5. Konfirmasi
6. Wait for restoration
7. Verify data

**File Recovery:**

1. Buka **"Backup & Recovery"** → **"File Recovery"**
2. Pilih backup date
3. Browse files
4. Select files to restore
5. Klik **"Restore"**

**⚠️ Warning:**

- Recovery akan overwrite data existing
- Backup data current sebelum recovery
- Test recovery di staging environment dulu

---

## Troubleshooting

### Common Issues

#### 1. User Cannot Login

**Symptoms:**

- Invalid credentials error
- Account locked

**Solutions:**

1. Check user status (active/inactive)
2. Reset password
3. Check account lock (after 5 failed attempts)
4. Unlock account: **"User Management"** → Select user → **"Unlock"**

#### 2. Integration Sync Failed

**Symptoms:**

- SIMAN/MySIMKARI data not updated
- Sync error in log

**Solutions:**

1. Check integration status: **"Integration Settings"**
2. Test API connection
3. Check API credentials in Secreton
4. Check network connectivity
5. Retry sync manually: **"Integration"** → **"Trigger Sync"**

#### 3. Slow Performance

**Symptoms:**

- Page load time > 5 seconds
- Timeout errors

**Solutions:**

1. Check system resources: **"System Monitoring"**
2. Check database slow queries
3. Clear cache: **"System"** → **"Clear Cache"**
4. Restart application (if needed)
5. Scale up resources (if persistent)

#### 4. Document Generation Failed

**Symptoms:**

- Document not generated
- Error in document service

**Solutions:**

1. Check document service status
2. Check template availability
3. Check storage space
4. Check MinIO/S3 connection
5. Retry generation

#### 5. Email Notification Not Sent

**Symptoms:**

- Users not receiving emails
- Email in queue

**Solutions:**

1. Check SMTP configuration
2. Test email connection
3. Check email queue: **"System"** → **"Email Queue"**
4. Retry failed emails
5. Check spam folder (user side)

### Logs Location

**Application Logs:**

- Path: `/var/log/simpel/application.log`
- Rotation: Daily
- Retention: 30 days

**Error Logs:**

- Path: `/var/log/simpel/error.log`
- Rotation: Daily
- Retention: 30 days

**Access Logs:**

- Path: `/var/log/simpel/access.log`
- Rotation: Daily
- Retention: 7 days

**Audit Logs:**

- Database: `audit_log` table
- Retention: 1 year

### Support Escalation

**Level 1: Helpdesk**

- Email: helpdesk@simpel.kejaksaan.go.id
- Telepon: (021) 1234-5678
- Response time: 4 jam

**Level 2: Technical Support**

- Email: support@simpel.kejaksaan.go.id
- Response time: 2 jam

**Level 3: Development Team**

- Email: dev@simpel.kejaksaan.go.id
- For critical issues only
- Response time: 1 jam

---

## Maintenance Schedule

### Regular Maintenance

**Daily:**

- Automated backup (01:00 WIB)
- Log rotation
- Cache cleanup

**Weekly:**

- Database optimization
- Index rebuild
- Performance review

**Monthly:**

- Security patch update
- Dependency update
- Full system health check

**Quarterly:**

- Disaster recovery drill
- Security audit
- Performance tuning

### Planned Downtime

**Notification:**

- Notify users 7 days before
- Send reminder 1 day before
- Display maintenance banner

**Maintenance Window:**

- Preferred: Sunday 00:00-04:00 WIB
- Duration: Max 4 hours

**Post-Maintenance:**

- Verify all services running
- Check integration status
- Monitor for issues

---

## Security Best Practices

### Password Policy

- Minimum 12 characters
- Must include: uppercase, lowercase, number, special char
- Password expiry: 90 days
- Cannot reuse last 5 passwords
- Account lock after 5 failed attempts

### Access Control

- Principle of least privilege
- Regular access review (quarterly)
- Remove access for inactive users (30 days)
- MFA for admin accounts

### Data Protection

- Encryption at rest (AES-256)
- Encryption in transit (TLS 1.3)
- Sensitive data masking in logs
- PII data protection

### Audit and Compliance

- Enable audit logging for all critical operations
- Regular audit log review
- Compliance report generation
- Incident response plan

---

## Kontak

**Technical Support:**

- Email: support@simpel.kejaksaan.go.id
- Telepon: (021) 1234-5678 ext. 100

**Development Team:**

- Email: dev@simpel.kejaksaan.go.id

**Documentation:**

- https://docs.simpel.kejaksaan.go.id

---

**Versi:** 1.0.0
**Terakhir Diperbarui:** Februari 2026
