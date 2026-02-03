# Database Migrations - Layanan Integrasi

Struktur database **optimal** dan **efisien** untuk integrasi dengan:
- **MonSAKTI v1.4** (8 modules)
- **MySIMKARI** API
- **SIMAN v2.0** API (Sistem Informasi Manajemen Aset Negara)

## 🎯 Filosofi Desain

**Simplified & Purpose-Driven:**
- ✅ Hanya tabel yang **benar-benar digunakan** di kodebase
- ✅ Audit & logging tables untuk compliance dan monitoring
- ✅ Core reference tables untuk data utama
- ✅ JSONB `raw_data` untuk fleksibilitas tanpa schema explosion
- ✅ Indexes optimal untuk query performance

**Menghindari:**
- ❌ Tabel kosong yang tidak pernah diisi
- ❌ Schema rigid untuk setiap field API
- ❌ Duplikasi struktur untuk semua endpoint
- ❌ Over-engineering tanpa kebutuhan nyata

## 📁 File Migration

| File | Tujuan | Baris |
|------|--------|-------|
| `001_init_schema.sql` | Complete schema - Tables, indexes, views, triggers | ~500 |
| `002_rollback.sql` | Clean rollback script | ~50 |

**Total:** 2 files (sebelumnya: 15+ files terfragmentasi)

## 🗂️ Struktur Database

### 1. Audit & Logging (Actively Used)

```sql
api_call_log           -- ✅ USED: audit.rs
batch_processing_log   -- ✅ USED: audit.rs
data_sync_log          -- ✅ USED: audit.rs
token_reset_log        -- ✅ USED: audit.rs
api_tokens             -- ✅ USED: client.rs
```

### 2. MonSAKTI Reference

```sql
adm_ref_admin          -- ✅ USED: batch/orchestrator.rs
adm_ref_bank           -- Reference data
adm_ref_jns_spp        -- Reference data
+ Dynamic tables: {module}_{endpoint}
```

### 3. MySIMKARI

```sql
mysimkari_satker       -- ✅ USED: scheduler.rs
mysimkari_pegawai      -- ✅ USED: scheduler.rs
```

### 4. SIMAN v2.0

```sql
siman_aset_tanah               -- ✅ USED: scheduler.rs
siman_aset_gedung_bangunan     -- ✅ USED: scheduler.rs
siman_aset_alat_besar          -- ✅ USED: scheduler.rs
siman_aset_angkutan_bermotor   -- ✅ USED: scheduler.rs
```

### 5. Analytical Views

```sql
v_recent_failed_calls      -- ✅ USED: audit.rs
v_token_health             -- ✅ USED: audit.rs
v_api_stats_by_module      -- ✅ USED: audit.rs
```

## 🚀 Instalasi

```bash
# Run migration
psql -U postgres -d simpelv2 -f migrations/001_init_schema.sql

# Rollback (if needed)
psql -U postgres -d simpelv2 -f migrations/002_rollback.sql
```

## 📊 Monitoring Queries

```sql
-- Token health
SELECT * FROM integrasi.v_token_health;

-- Recent failures
SELECT * FROM integrasi.v_recent_failed_calls LIMIT 20;

-- API statistics (last 7 days)
SELECT * FROM integrasi.v_api_stats_by_module;
```

## 🔧 Maintenance

```sql
-- Weekly vacuum
VACUUM ANALYZE integrasi.api_call_log;

-- Monthly cleanup (keep 90 days)
DELETE FROM integrasi.api_call_log
WHERE created_at < CURRENT_TIMESTAMP - INTERVAL '90 days';
```

---

**v2.0** - Optimized & Production-Ready
**Last Updated:** February 2, 2026
