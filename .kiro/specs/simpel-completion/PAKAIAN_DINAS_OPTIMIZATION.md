# Pakaian Dinas Module - Optimization Summary

## Executive Summary

After analyzing the existing Laravel implementation in `simpel_laravel` and `simpel_web-main`, I've identified key optimizations for the Rust migration. The current implementation is functional but has several areas for improvement following best practices.

## Current Implementation Analysis

### Strengths
1. **Complete workflow**: 3-level hierarchical approval (Kejari → Kejati → Kejagung)
2. **Comprehensive reporting**: Laporan Daftar (individual) and Laporan Rekap (aggregated)
3. **Performance optimization**: Uses CTE-based SQL queries
4. **Master data tracking**: Updates `pegawai_pakaian_dinas` on completion
5. **Proper database schema**: 8 tables with relationships and indexes

### Issues Found
1. **Satker selection**: Uses string matching (`LIKE '0200%'`) instead of proper hierarchical structure
2. **Mixed ID types**: Uses both `inst_satkerkd` (string) and UUID inconsistently
3. **Complex pusat handling**: `ms_satker_pusat_id` and `is_pusat` flag creates confusion
4. **Workflow validation**: State transitions not clearly validated in code
5. **No MonSAKTI integration**: Wilayah structure not utilized

## Optimization Recommendations

### 1. Satker Selection - MonSAKTI Wilayah Structure

**Current Approach (Laravel):**
```php
// String matching - fragile and inefficient
$satkerQ->where('inst_satkerkd', 'like', "{$kdSatker}%");
```

**Optimized Approach (Rust):**
```rust
// Use MonSAKTI wilayah hierarchy (2-level tree)
// Level 1: Wilayah codes (0100=Kejagung, 0200=Jawa Barat, 3400=Sulawesi Barat)
// Level 2: Satker codes under each wilayah (008310=Kejaksaan Negeri Mamuju under 3400)

pub struct SatkerSelectionTree {
    pub wilayah_code: String,  // "0200", "3400", etc.
    pub wilayah_name: String,  // "Jawa Barat", "Sulawesi Barat"
    pub satkers: Vec<SatkerNode>,
}

pub struct SatkerNode {
    pub satker_id: String,     // "008310"
    pub satker_name: String,   // "Kejaksaan Negeri Mamuju"
    pub is_selected: bool,
    pub is_auto_included: bool, // true if parent wilayah selected
}

// Selection logic
impl SatkerSelectionTree {
    pub fn select_wilayah(&mut self, wilayah_code: &str) {
        // Auto-select all satkers in wilayah
        for satker in &mut self.satkers {
            satker.is_selected = true;
            satker.is_auto_included = true;
        }
    }

    pub fn select_satker(&mut self, satker_id: &str) {
        // Manually select specific satker
        if let Some(satker) = self.satkers.iter_mut().find(|s| s.satker_id == satker_id) {
            satker.is_selected = true;
            satker.is_auto_included = false;
        }
    }
}

// Database storage
// is_show_in_form: 1 = manually selected, 0 = auto-included from wilayah selection
```

**Benefits:**
- Clear hierarchical structure (2 levels, not 4)
- No string matching - uses proper codes
- Efficient parent-child selection
- Aligns with MonSAKTI standard

### 2. Workflow State Machine

**Current Approach (Laravel):**
```php
// State transitions scattered across controller methods
// No clear validation of allowed transitions
```

**Optimized Approach (Rust):**
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowState {
    Draft = 1000,
    KejariSubmitted = 1009,
    KejatiSubmitted = 1011,
    KejatiApproved = 1012,
    KejagungReviewed = 1010,
    Completed = 1008,
    KejatiRevision = 1005,
    KejagungRevision = 1007,
}

impl WorkflowState {
    pub fn allowed_transitions(&self) -> Vec<WorkflowState> {
        match self {
            WorkflowState::Draft => vec![WorkflowState::KejariSubmitted],
            WorkflowState::KejariSubmitted => vec![
                WorkflowState::KejatiApproved,
                WorkflowState::KejatiRevision,
            ],
            WorkflowState::KejatiApproved => vec![WorkflowState::KejagungReviewed],
            WorkflowState::KejagungReviewed => vec![
                WorkflowState::Completed,
                WorkflowState::KejagungRevision,
            ],
            WorkflowState::KejatiRevision => vec![WorkflowState::Draft],
            WorkflowState::KejagungRevision => vec![
                WorkflowState::KejatiSubmitted,
                WorkflowState::Draft,
            ],
            WorkflowState::Completed => vec![], // Terminal state
        }
    }

    pub fn can_transition_to(&self, target: WorkflowState) -> bool {
        self.allowed_transitions().contains(&target)
    }

    pub fn validate_transition(&self, target: WorkflowState) -> Result<(), WorkflowError> {
        if !self.can_transition_to(target) {
            return Err(WorkflowError::InvalidTransition {
                from: *self,
                to: target,
            });
        }
        Ok(())
    }
}

// Usage in service layer
pub async fn transition_state(
    &self,
    pengajuan_satker_id: Uuid,
    target_state: WorkflowState,
    user_id: Uuid,
    comment: Option<String>,
) -> Result<()> {
    let current = self.get_current_state(pengajuan_satker_id).await?;

    // Validate transition
    current.validate_transition(target_state)?;

    // Update state
    self.update_state(pengajuan_satker_id, target_state).await?;

    // Log activity
    self.log_activity(pengajuan_satker_id, target_state, user_id, comment).await?;

    // Send notifications
    self.send_notifications(pengajuan_satker_id, target_state).await?;

    // Trigger side effects (e.g., master data update on COMPLETED)
    if target_state == WorkflowState::Completed {
        self.update_master_data(pengajuan_satker_id).await?;
    }

    Ok(())
}
```

**Benefits:**
- Type-safe state transitions
- Clear validation rules
- Centralized transition logic
- Easy to test and maintain

### 3. Master Data Update (UPSERT Logic)

**Current Approach (Laravel):**
```php
// Delete then insert - not atomic
PegawaiPakaianDinas::whereIn('nip', $nips)->delete();
DB::table('pegawai_pakaian_dinas')->insert($data);
```

**Optimized Approach (Rust):**
```rust
pub async fn update_master_data(&self, pengajuan_satker_id: Uuid) -> Result<()> {
    let pegawai_data = self.get_pegawai_with_ukuran(pengajuan_satker_id).await?;

    // Batch UPSERT for performance
    let query = r#"
        INSERT INTO pegawai_pakaian_dinas (
            nip, nama, ukuran_baju, ukuran_celana, ukuran_sepatu,
            with_hijab, pangkat, jabatan, last_pengajuan_satker_pegawai_id, updated_at
        )
        SELECT * FROM UNNEST(
            $1::varchar[], $2::varchar[], $3::varchar[], $4::varchar[], $5::varchar[],
            $6::boolean[], $7::varchar[], $8::varchar[], $9::uuid[], $10::timestamptz[]
        )
        ON CONFLICT (nip) DO UPDATE SET
            nama = EXCLUDED.nama,
            ukuran_baju = EXCLUDED.ukuran_baju,
            ukuran_celana = EXCLUDED.ukuran_celana,
            ukuran_sepatu = EXCLUDED.ukuran_sepatu,
            with_hijab = EXCLUDED.with_hijab,
            pangkat = EXCLUDED.pangkat,
            jabatan = EXCLUDED.jabatan,
            last_pengajuan_satker_pegawai_id = EXCLUDED.last_pengajuan_satker_pegawai_id,
            updated_at = EXCLUDED.updated_at
    "#;

    // Prepare arrays for batch insert
    let nips: Vec<String> = pegawai_data.iter().map(|p| p.nip.clone()).collect();
    let namas: Vec<String> = pegawai_data.iter().map(|p| p.nama.clone()).collect();
    // ... other fields

    self.db_pool.get().await?
        .execute(query, &[&nips, &namas, /* ... */])
        .await?;

    // Audit log
    self.audit_logger.log(
        AuditEvent::MasterDataUpdated {
            pengajuan_satker_id,
            count: pegawai_data.len(),
        },
        user_id,
        ip_address,
    ).await?;

    Ok(())
}
```

**Benefits:**
- Atomic operation (no delete-then-insert race condition)
- Batch processing for performance
- Proper audit trail
- Handles conflicts gracefully

### 4. Reporting Optimization

**Current Approach (Laravel):**
```php
// Good: Uses CTE-based SQL
// Issue: No caching, generates on every request
```

**Optimized Approach (Rust):**
```rust
pub struct ReportService {
    db_pool: deadpool_postgres::Pool,
    cache: CacheManager,
}

impl ReportService {
    pub async fn generate_laporan_daftar(
        &self,
        pengajuan_id: Uuid,
        filters: ReportFilters,
    ) -> Result<LaporanDaftarData> {
        // Check cache first (5-minute TTL)
        let cache_key = format!("laporan_daftar:{}:{:?}", pengajuan_id, filters);
        if let Some(cached) = self.cache.get::<LaporanDaftarData>(&cache_key).await? {
            return Ok(cached);
        }

        // Generate report using CTE-based SQL (same as Laravel)
        let query = r#"
            WITH pegawai_data AS (
                SELECT psp.id, psp.nip, psp.nama, psp.pangkat, psp.jabatan, psp.eselon, psp.jenis,
                       ps.ms_satker_id, s.nama_satker,
                       ppu.pengajuan_pakaian_id, pp.nama_pakaian, ppu.ukuran
                FROM pengajuan_pakaian_dinas_satker_pegawai psp
                JOIN pengajuan_pakaian_dinas_satker ps ON psp.pengajuan_satker_id = ps.id
                JOIN ms_satker s ON ps.ms_satker_id = s.id
                LEFT JOIN pengajuan_pakaian_dinas_satker_pegawai_ukuran ppu
                    ON psp.id = ppu.pengajuan_satker_pegawai_id
                LEFT JOIN pengajuan_pakaian_dinas_pakaian pp
                    ON ppu.pengajuan_pakaian_id = pp.id
                WHERE ps.pengajuan_id = $1
                  AND ($2::int IS NULL OR psp.jenis = $2)
                  AND ($3::text IS NULL OR psp.eselon = $3)
                  AND ($4::char IS NULL OR psp.jenis_kelamin = $4)
            )
            SELECT * FROM pegawai_data ORDER BY nama_satker, nip
        "#;

        let rows = self.db_pool.get().await?
            .query(query, &[&pengajuan_id, &filters.jenis, &filters.eselon, &filters.jenis_kelamin])
            .await?;

        let data = self.process_laporan_daftar_rows(rows)?;

        // Cache for 5 minutes
        self.cache.set(&cache_key, &data, SensitivityLevel::Internal).await?;

        Ok(data)
    }

    pub async fn export_to_excel(
        &self,
        data: &LaporanDaftarData,
    ) -> Result<Vec<u8>> {
        use rust_xlsxwriter::*;

        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();

        // Headers
        let header_format = Format::new().set_bold().set_background_color(Color::RGB(0xD3D3D3));
        worksheet.write_string_with_format(0, 0, "No", &header_format)?;
        worksheet.write_string_with_format(0, 1, "NIP", &header_format)?;
        worksheet.write_string_with_format(0, 2, "Nama", &header_format)?;
        // ... other headers

        // Data rows
        for (idx, row) in data.rows.iter().enumerate() {
            worksheet.write_number(idx as u32 + 1, 0, (idx + 1) as f64)?;
            worksheet.write_string(idx as u32 + 1, 1, &row.nip)?;
            worksheet.write_string(idx as u32 + 1, 2, &row.nama)?;
            // ... other columns
        }

        // Auto-width columns
        worksheet.autofit();

        // Freeze panes (first row and first column)
        worksheet.set_freeze_panes(1, 1)?;

        let buffer = workbook.save_to_buffer()?;
        Ok(buffer)
    }
}
```

**Benefits:**
- 5-minute cache TTL reduces database load
- Efficient Excel generation with rust_xlsxwriter
- Same CTE-based SQL as Laravel (proven performance)
- Type-safe data structures

## Implementation Priority

1. **Phase 1 - Core Workflow** (Week 1-2)
   - Database schema migration (already done)
   - Workflow state machine implementation
   - Basic CRUD operations

2. **Phase 2 - Satker Selection** (Week 3)
   - MonSAKTI wilayah tree structure
   - Hierarchical selection logic
   - Frontend tree component

3. **Phase 3 - Reporting** (Week 4)
   - Laporan Daftar with caching
   - Laporan Rekap with L/P breakdown
   - Excel export

4. **Phase 4 - Master Data** (Week 5)
   - UPSERT logic for pegawai_pakaian_dinas
   - Audit trail
   - Rollback capability

## Testing Strategy

1. **Unit Tests**
   - Workflow state transitions
   - Satker selection logic
   - Report data processing

2. **Integration Tests**
   - Database operations (UPSERT)
   - API endpoints
   - Excel generation

3. **Property-Based Tests**
   - Workflow state machine properties
   - Report aggregation correctness
   - Cache consistency

## Migration Path from Laravel

1. **Database**: Already migrated (20260202_create_pakaian_dinas_tables.sql)
2. **API Endpoints**: Map Laravel routes to Axum handlers
3. **Business Logic**: Port controller methods to service layer
4. **Frontend**: Rewrite Blade templates to Leptos components
5. **Testing**: Port PHPUnit tests to Rust tests

## Conclusion

The optimized approach maintains the strengths of the Laravel implementation while addressing its weaknesses:
- **Better structure**: MonSAKTI wilayah hierarchy instead of string matching
- **Type safety**: Workflow state machine with compile-time validation
- **Performance**: Caching, batch operations, efficient SQL
- **Maintainability**: Clear separation of concerns, testable code

The Rust migration is an opportunity to implement these best practices from the start, resulting in a more robust and maintainable system.
