# Database Schema Strategy Analysis

**Date:** 2025-11-06
**Module:** `layanan/shared/integrasi`
**Issue:** Inconsistent schema usage across migrations

---

## Current State

### Schema Distribution

| Migration Files     | Schema      | Tables       | Status                      |
| ------------------- | ----------- | ------------ | --------------------------- |
| 001-009 (MonSAKTI)  | `public`    | 34+ tables   | ❌ No schema isolation      |
| 010 (MySIMKARI)     | `public`    | 2 tables     | ❌ No schema isolation      |
| 011 (Audit)         | `public`    | 3 tables     | ❌ No schema isolation      |
| 012 (Token Mgmt)    | `public`    | 4 tables     | ❌ No schema isolation      |
| **013-014 (SIMAN)** | **`siman`** | **2 tables** | ✅ **Has dedicated schema** |

### Problem Identified

**INCONSISTENT SCHEMA STRATEGY:**

1. ✅ **SIMAN** uses dedicated schema: `siman`
2. ❌ **Everything else** uses default schema: `public`

This creates:

- **Namespace pollution** in public schema (40+ tables)
- **No logical separation** between different API modules
- **Inconsistent architecture** - only SIMAN is isolated
- **Harder maintenance** - all tables mixed together
- **Permission management complexity** - can't grant per-module access easily

---

## Current Table List (Public Schema)

### MonSAKTI Tables (34+ tables)

```sql
-- ADM (Administrasi) - 3 tables
adm_ref_admin, adm_pejabat, adm_ref_jns_spp

-- ANG (Anggaran) - 2 tables
ang_data_ang, ang_ref_sts

-- PEM (Pembayaran) - 5 tables
pem_realisasi, pem_spp_header, pem_spp_pengeluaran,
pem_spp_potongan, pem_penerima_spm

-- BEN (Bendahara) - 8 tables
ben_kas_tunai, ben_kas_bank, ben_spby, ben_kuitansi,
ben_drpp, ben_pungut_pajak, ben_setor_pajak, ben_pnbp

-- KOM (Komitmen) - 11 tables
kom_capaian_ro, kom_kontrak_header, kom_kontrak_line,
kom_kontrak_termin, kom_kontrak_coa, kom_bast_kontrak_header,
kom_bast_kontrak_detail_barang, kom_bast_kontrak_coa,
kom_bast_non_kontrak_header, kom_bast_non_kontrak_detail_barang,
kom_bast_non_kontrak_coa, kom_supplier_header,
kom_supplier_address, kom_supplier_bank

-- AST/PER (Aset & Persediaan) - 2 tables
ast_aset_trx, per_persedia_trx

-- GLP (General Ledger) - 3 tables
glp_buku_besar, glp_neraca_sawal, glp_fa_detail
```

### MySIMKARI Tables (2 tables)

```sql
mysimkari_satker, mysimkari_pegawai
```

### Infrastructure Tables (7 tables)

```sql
-- Audit Logging
api_call_log, batch_processing_log, data_sync_log

-- Token Management
api_tokens, api_token_history, token_reset_log, token_rotation_policy
```

### SIMAN Tables (Separate Schema ✅)

```sql
-- Schema: siman
siman.siman_sync_log, siman.siman_aset
```

**Total:** 45+ tables, with 43 in `public` and 2 in `siman`

---

## Recommended Solutions

### 🎯 Option 1: Full Schema Separation (IDEAL)

**Structure:**

```
monsakti (schema)
├── adm_* (3 tables)
├── ang_* (2 tables)
├── pem_* (5 tables)
├── ben_* (8 tables)
├── kom_* (14 tables)
├── ast_* (1 table)
├── per_* (1 table)
└── glp_* (3 tables)

mysimkari (schema)
├── mysimkari_satker
└── mysimkari_pegawai

siman (schema) ✅ ALREADY IMPLEMENTED
├── siman_sync_log
└── siman_aset

integrasi (schema)
├── api_call_log
├── batch_processing_log
├── data_sync_log
├── api_tokens
├── api_token_history
├── token_reset_log
└── token_rotation_policy
```

**Benefits:**

- ✅ **Clean namespace separation** - each module isolated
- ✅ **Easy permission management** - grant access per schema
- ✅ **Logical organization** - matches code structure
- ✅ **Better backup/restore** - per module granularity
- ✅ **Follows microservices principles** - loose coupling
- ✅ **Consistent with SIMAN** - all modules use own schema

**Drawbacks:**

- ⚠️ **Breaking change** - existing queries need schema qualification
- ⚠️ **More complex queries** - cross-schema JOINs need explicit schema names
- ⚠️ **Migration effort** - need to move existing tables

**Implementation:**

```sql
-- New migration: 015_reorganize_schemas.sql
CREATE SCHEMA IF NOT EXISTS monsakti;
CREATE SCHEMA IF NOT EXISTS mysimkari;
CREATE SCHEMA IF NOT EXISTS integrasi;

-- Move tables (example)
ALTER TABLE api_call_log SET SCHEMA integrasi;
ALTER TABLE mysimkari_satker SET SCHEMA mysimkari;
-- ... etc

-- Update search_path in application
SET search_path TO monsakti, mysimkari, siman, integrasi, public;
```

---

### 🎯 Option 2: Pragmatic Two-Schema Approach (RECOMMENDED)

**Structure:**

```
integrasi (schema) - Internal operations
├── MonSAKTI tables (adm_*, ang_*, pem_*, ben_*, kom_*, ast_*, per_*, glp_*)
├── MySIMKARI tables (mysimkari_*)
├── Audit tables (api_call_log, batch_processing_log, data_sync_log)
└── Token tables (api_tokens, token_reset_log, etc.)

siman (schema) - External BMN system ✅ ALREADY IMPLEMENTED
├── siman_sync_log
└── siman_aset
```

**Benefits:**

- ✅ **Consistent approach** - SIMAN already separate, make integrasi separate too
- ✅ **Simpler than Option 1** - only 2 schemas instead of 4
- ✅ **Groups related functionality** - all integration tables together
- ✅ **Easier migration** - move all at once
- ✅ **Cleaner public schema** - only PostgreSQL defaults remain

**Drawbacks:**

- ⚠️ **Still a breaking change** - queries need updates
- ⚠️ **Less granular** - can't separate MonSAKTI from MySIMKARI

**Implementation:**

```sql
-- Migration: 015_create_integrasi_schema.sql
CREATE SCHEMA IF NOT EXISTS integrasi;

-- Move all tables to integrasi schema
ALTER TABLE api_call_log SET SCHEMA integrasi;
ALTER TABLE adm_ref_admin SET SCHEMA integrasi;
ALTER TABLE mysimkari_satker SET SCHEMA integrasi;
-- ... etc (40+ tables)

-- Update search_path
SET search_path TO integrasi, siman, public;
```

---

### 🎯 Option 3: Phased Migration (SAFEST)

**Phase 1 (Immediate - Non-breaking):**

```
public (schema) - Keep existing tables
├── All MonSAKTI tables (unchanged)
├── All MySIMKARI tables (unchanged)
├── Audit tables → MOVE to integrasi schema
└── Token tables → MOVE to integrasi schema

siman (schema) ✅ ALREADY DONE
├── siman_sync_log
└── siman_aset

integrasi (schema) NEW
├── api_call_log
├── batch_processing_log
├── data_sync_log
├── api_tokens
├── api_token_history
├── token_reset_log
└── token_rotation_policy
```

**Phase 2 (Future - Breaking change):**

```
Move MonSAKTI and MySIMKARI to integrasi or separate schemas
```

**Benefits:**

- ✅ **Minimal breaking changes** - only audit/token tables move
- ✅ **Quick to implement** - fewer tables to move
- ✅ **Consistent with SIMAN** - infrastructure isolated
- ✅ **Low risk** - audit/token tables are internal only
- ✅ **Provides time** - can plan bigger migration later

**Drawbacks:**

- ⚠️ **Still somewhat inconsistent** - MonSAKTI/MySIMKARI still in public
- ⚠️ **Two-step process** - need another migration later

**Implementation:**

```sql
-- Migration: 015_move_infrastructure_to_integrasi_schema.sql
CREATE SCHEMA IF NOT EXISTS integrasi;

-- Move infrastructure tables only (low risk)
ALTER TABLE api_call_log SET SCHEMA integrasi;
ALTER TABLE batch_processing_log SET SCHEMA integrasi;
ALTER TABLE data_sync_log SET SCHEMA integrasi;
ALTER TABLE api_tokens SET SCHEMA integrasi;
ALTER TABLE api_token_history SET SCHEMA integrasi;
ALTER TABLE token_reset_log SET SCHEMA integrasi;
ALTER TABLE token_rotation_policy SET SCHEMA integrasi;

-- Update application search_path
-- Old: default public
-- New: SET search_path TO public, integrasi, siman;
```

---

## Impact Analysis

### Code Changes Required

#### Option 1 (Full Separation)

```rust
// BEFORE
let query = "SELECT * FROM api_call_log WHERE module = $1";

// AFTER
let query = "SELECT * FROM integrasi.api_call_log WHERE module = $1";

// OR set search_path in connection
client.execute("SET search_path TO integrasi, monsakti, mysimkari, siman, public", &[]).await?;
let query = "SELECT * FROM api_call_log WHERE module = $1"; // Works without schema prefix
```

#### Option 2 (Two-Schema)

```rust
// BEFORE
let query = "SELECT * FROM adm_ref_admin WHERE kdsatker = $1";

// AFTER
let query = "SELECT * FROM integrasi.adm_ref_admin WHERE kdsatker = $1";

// OR set search_path
client.execute("SET search_path TO integrasi, siman, public", &[]).await?;
```

#### Option 3 (Phased - Minimal Impact)

```rust
// ONLY audit.rs needs changes
// BEFORE
let query = "INSERT INTO api_call_log (...) VALUES (...)";

// AFTER
let query = "INSERT INTO integrasi.api_call_log (...) VALUES (...)";

// OR set search_path (RECOMMENDED)
// In db.rs connection setup
pub async fn connect() -> Result<Client> {
    let client = tokio_postgres::connect(...).await?;
    client.execute("SET search_path TO public, integrasi, siman", &[]).await?;
    Ok(client)
}
// Then no code changes needed in queries!
```

### Migration Risk Assessment

| Option   | Risk Level | Affected Queries | Rollback Complexity |
| -------- | ---------- | ---------------- | ------------------- |
| Option 1 | 🔴 HIGH    | All queries      | Complex             |
| Option 2 | 🟡 MEDIUM  | All queries      | Moderate            |
| Option 3 | 🟢 LOW     | Only audit/token | Easy                |

---

## ⭐ FINAL RECOMMENDATION: All-in-One `integrasi` Schema

### Why This Is THE BEST Choice:

1. **✅ Maximum Simplicity** - Only 2 schemas total: `integrasi` + `siman`
2. **✅ Clean Architecture** - Clear separation: internal vs external
3. **✅ Easy Queries** - No schema qualification needed within integrasi
4. **✅ One-Time Migration** - Single breaking change, then done
5. **✅ Easy Permissions** - Grant at schema level covers everything
6. **✅ Easy Backup/Restore** - `pg_dump -n integrasi` gets all internal data
7. **✅ Production Ready** - Simple to maintain and understand

### Structure:

```
integrasi schema (43 tables) - All internal operations
├── MonSAKTI (34+ tables)
├── MySIMKARI (2 tables)
├── Audit (3 tables)
└── Token Management (4 tables)

siman schema (2 tables) - External BMN system (keep separate)
├── siman_sync_log
└── siman_aset
```

### Implementation Plan:

#### Step 1: Create New Migration (015)

```bash
touch layanan/shared/integrasi/migrations/015_move_infrastructure_to_integrasi_schema.sql
```

#### Step 2: Migration Content

```sql
-- =====================================================
-- Migration: 015_move_infrastructure_to_integrasi_schema.sql
-- Description: Move audit and token management tables to integrasi schema
-- Date: 2025-11-06
-- =====================================================

-- Create integrasi schema
CREATE SCHEMA IF NOT EXISTS integrasi;

-- Move audit logging tables
ALTER TABLE IF EXISTS api_call_log SET SCHEMA integrasi;
ALTER TABLE IF EXISTS batch_processing_log SET SCHEMA integrasi;
ALTER TABLE IF EXISTS data_sync_log SET SCHEMA integrasi;

-- Move token management tables
ALTER TABLE IF EXISTS api_tokens SET SCHEMA integrasi;
ALTER TABLE IF EXISTS api_token_history SET SCHEMA integrasi;
ALTER TABLE IF EXISTS token_reset_log SET SCHEMA integrasi;
ALTER TABLE IF EXISTS token_rotation_policy SET SCHEMA integrasi;

-- Views also need to be moved/updated
-- (List all views that reference these tables)

-- Success message
DO $$
BEGIN
    RAISE NOTICE '========================================';
    RAISE NOTICE 'INFRASTRUCTURE TABLES MOVED TO INTEGRASI SCHEMA';
    RAISE NOTICE '========================================';
    RAISE NOTICE 'Moved 7 tables: audit (3) + token (4)';
    RAISE NOTICE 'Schema: integrasi';
    RAISE NOTICE 'MonSAKTI/MySIMKARI remain in public (Phase 2)';
    RAISE NOTICE '========================================';
END $$;
```

#### Step 3: Update db.rs Connection

```rust
// In src/db.rs or equivalent
pub async fn connect_with_integrasi_schema(config: &Config) -> Result<Client> {
    let client = connect(config).await?;

    // Set search_path to include integrasi and siman schemas
    client.execute(
        "SET search_path TO public, integrasi, siman",
        &[]
    ).await?;

    Ok(client)
}
```

#### Step 4: Testing

```bash
# Run migration
psql -U postgres -d integrasi_db -f migrations/015_move_infrastructure_to_integrasi_schema.sql

# Verify tables moved
psql -U postgres -d integrasi_db -c "\dt integrasi.*"

# Test queries still work
cargo test
```

#### Step 5: Update Documentation

- README.md - mention schema structure
- MIGRATION_SYNC_REPORT.md - update with new schema info

---

## Future Considerations (Phase 2)

When ready for full migration:

### Option A: Move MonSAKTI to separate schema

```sql
CREATE SCHEMA monsakti;
ALTER TABLE adm_* SET SCHEMA monsakti;
ALTER TABLE ang_* SET SCHEMA monsakti;
-- etc
```

### Option B: Move to integrasi schema

```sql
ALTER TABLE adm_* SET SCHEMA integrasi;
ALTER TABLE mysimkari_* SET SCHEMA integrasi;
-- etc
```

### Decision Criteria:

- If planning to split services → Option A (separate schemas)
- If keeping as monolith → Option B (single integrasi schema)

---

## Summary

**Current State:**

- ❌ Inconsistent: SIMAN has schema, others don't
- ❌ Public schema cluttered with 40+ tables

**Recommended Solution:**

- ✅ **Phase 1:** Move audit/token to `integrasi` schema
- ✅ **Phase 2:** Evaluate moving MonSAKTI/MySIMKARI later

**Benefits:**

- Consistent architecture
- Better organization
- Low risk implementation
- Easy to expand later

**Next Step:**
Create migration 015 to move infrastructure tables to `integrasi` schema.

---

**Analysis by:** GitHub Copilot AI Coding Agent
**Date:** 2025-11-06
**Status:** READY FOR IMPLEMENTATION
**Recommended Action:** Implement Option 3 (Phased Migration)
