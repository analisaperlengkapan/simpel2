# Task 28.5.6 Implementation Summary

## Integrate Pemakaian BMN Activation with Document Service

### Requirements
- REQ-P006: Generate permit documents via dokumen service
- REQ-D002: Generate PDF with official letterhead
- REQ-D004: Generate PDF with official letterhead
- REQ-D005: Provide API for document generation requests

### Implementation Completed

#### 1. Database Migration
**File:** `layanan/perlengkapan/crates/api/migrations/20260211_add_document_fields_to_pemakaian_bmn.sql`

Added two new columns to `izin_pemakaian_bmn` table:
- `document_id UUID` - Reference to generated document in dokumen service
- `document_url TEXT` - Download URL for the permit document
- Added index on `document_id` for performance

#### 2. Model Updates
**File:** `layanan/perlengkapan/crates/api/src/pemakaian_bmn/models.rs`

Updated `IzinPemakaianBmn` struct to include:
```rust
pub document_id: Option<Uuid>,
pub document_url: Option<String>,
```

#### 3. Repository Updates
**File:** `layanan/perlengkapan/crates/api/src/pemakaian_bmn/repository.rs`

Added new method:
- `update_document_fields()` - Updates document_id and document_url after document generation
- Updated `row_to_permit()` to map document fields from database rows

#### 4. Service Layer Updates
**File:** `layanan/perlengkapan/crates/api/src/pemakaian_bmn/services.rs`

**Added dokumen_client field:**
```rust
dokumen_client: Option<Arc<tokio::sync::Mutex<crate::workflow::DokumenClient>>>,
```

**Added builder method:**
- `with_dokumen_client()` - Sets the dokumen service client

**Enhanced activate_permit() method:**
- After generating permit number and transitioning to ACTIVE
- Calls `generate_permit_document()` to create the document
- Implements retry logic (3 attempts with exponential backoff)
- Updates permit with document reference
- Continues activation even if document generation fails (manual fallback)

**Added generate_permit_document() method:**
- Prepares document data with all permit fields
- Calls dokumen service via gRPC
- Implements retry logic with exponential backoff (2^retry_count seconds)
- Returns document_id and download_url
- Handles errors gracefully

**Document data includes:**
- Permit information (nomor_izin, dates, purpose)
- Pegawai information (NIP, name, position, satker)
- BMN information (NUP, code, name, specifications)
- Type-specific fields (vehicle, housing, laptop)
- Approval information

#### 5. Handler Updates
**File:** `layanan/perlengkapan/crates/api/src/pemakaian_bmn/handlers.rs`

Added new endpoint handler:
- `get_permit_document()` - GET /pemakaian-bmn/:id/document
- Validates user has access to permit
- Returns redirect to document URL
- Returns 404 if document doesn't exist

#### 6. Routes Updates
**File:** `layanan/perlengkapan/crates/api/src/routes.rs`

Added new route:
```rust
.route("/pemakaian-bmn/:id/document", get(pemakaian_bmn::get_permit_document))
```

#### 7. Test Suite
**File:** `layanan/perlengkapan/crates/api/tests/pemakaian_bmn_document_tests.rs`

Created comprehensive test suite with placeholders for:
- Document generation after activation
- Document stored in database
- Document download endpoint
- Error handling (service unavailable)
- Retry logic (3 attempts with exponential backoff)
- Max retries behavior
- Document data completeness
- Authentication requirements
- Not found scenarios

### Integration Flow

1. **Permit Activation:**
   ```
   User → POST /pemakaian-bmn/:id/activate
   → activate_permit()
   → generate_permit_number()
   → transition_permit_status() to ACTIVE
   → generate_permit_document() [NEW]
   → update_document_fields() [NEW]
   → Return permit with document reference
   ```

2. **Document Generation:**
   ```
   generate_permit_document()
   → Prepare document data (all permit fields)
   → Call dokumen_client.generate_document()
   → Retry up to 3 times on failure
   → Return (document_id, document_url)
   → Update permit in database
   ```

3. **Document Download:**
   ```
   User → GET /pemakaian-bmn/:id/document
   → get_permit_document()
   → Verify user access
   → Check document exists
   → Redirect to document_url
   ```

### Error Handling

1. **Document Service Unavailable:**
   - Retry 3 times with exponential backoff (2s, 4s, 8s)
   - Log warning if all retries fail
   - Continue with activation (permit is still ACTIVE)
   - Document can be generated manually later

2. **Document Not Found:**
   - Return 404 with clear error message
   - User can request manual document generation

3. **Authorization:**
   - Verify user is authenticated
   - Check user has access to permit (creator or same satker)

### Configuration Required

The implementation uses a placeholder template_id. In production:
1. Create permit document template in dokumen service
2. Configure template_id in application config
3. Template should include:
   - Official letterhead
   - Permit details (number, dates, purpose)
   - Pegawai information
   - BMN information
   - Terms and conditions
   - Signature block

### Testing Notes

The test file contains placeholder tests that document the expected behavior. Full implementation requires:
- Mock dokumen service client
- Test database with sample permits
- Verification of retry logic
- End-to-end integration tests

### Dependencies

This implementation depends on:
- `crate::workflow::DokumenClient` - Already implemented
- `crate::workflow::DocumentGenerationResult` - Already implemented
- Dokumen service gRPC endpoint - Must be running
- Document template - Must be created in dokumen service

### Next Steps

1. Run database migration to add document fields
2. Configure dokumen service endpoint in application config
3. Create permit document template in dokumen service
4. Update template_id in `generate_permit_document()` method
5. Implement full integration tests with mock dokumen service
6. Test end-to-end flow: create → approve → activate → download document
