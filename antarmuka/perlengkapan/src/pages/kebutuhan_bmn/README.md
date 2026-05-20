# Kebutuhan BMN Pages

Frontend pages for BMN needs analysis workflow.

## Pages

### 1. Period Management (`period_management.rs`)

**Actor:** Validator Pusat
**Status:** Planned (not yet implemented)

Allows Validator Pusat to:

- Create kebutuhan BMN periods
- Select eligible BMN items
- Select eligible satkers
- Set period constraints (start date, end date, deadline)

**Requirements:** REQ-K001, REQ-K002, REQ-K003

---

### 2. Submission Form (`submission_form.rs`)

**Actor:** Operator Satker
**Status:** ✅ Implemented

Allows Operator Satker to:

- View active pengajuan period information
- Add BMN items with justification (required per REQ-K005)
- Upload supporting documents (surat permohonan and attachments per REQ-K006)
- Submit kebutuhan BMN to Validator Wilayah (REQ-K007)
- Enforces period constraints (REQ-K004, REQ-K017)

**Requirements:** REQ-K004, REQ-K005, REQ-K006, REQ-K007, REQ-K017

**API Endpoints Used:**

- `GET /api/v1/kebutuhan-bmn/pengajuan?status_kode=2001` - Fetch active pengajuan
- `GET /api/v1/kebutuhan-bmn/pengajuan/{id}/satker` - Fetch satker detail
- `GET /api/v1/kebutuhan-bmn/satker/{id}` - Fetch barang items
- `POST /api/v1/kebutuhan-bmn/satker/{id}/barang` - Create barang item
- `POST /api/v1/kebutuhan-bmn/satker/{id}/submit-wilayah` - Submit to Validator Wilayah

**Components Used:**

- `Card` - Layout container from lib-ui
- `Alert` - Error/success messages from lib-ui
- `Spinner` - Loading indicator from lib-ui
- `Input` - Text input from lib-ui
- `Textarea` - Multi-line text input from lib-ui
- `Button` - Action buttons from lib-ui
- `FileUpload` - File upload with preview from lib-ui

**State Management:**

- Uses Leptos 0.8.x `signal()` pattern (NOT `create_signal()`)
- Reactive state for form inputs, loading, errors, and success messages
- Async data fetching with `spawn_local`

**Validation:**

- Nama barang required
- Justifikasi required (REQ-K005)
- Jumlah must be > 0
- Period deadline enforcement (REQ-K017)
- Minimum 1 barang item before submission

**File Upload:**

- Supports multiple files
- Accepted formats: PDF, DOC, DOCX, JPG, PNG
- Shows file preview
- TODO: Implement actual upload to server via POST /api/v1/kebutuhan-bmn/pengajuan/{id}/attachments

---

## Architecture

### Communication Pattern

- **Frontend → Backend:** REST API (JSON/HTTP)
- **NO direct gRPC calls** from frontend (per AGENTS.md)
- All API calls use `gloo_net::http::Request`

### Leptos 0.8.x Patterns

```rust
// ✅ Correct: Use signal()
let (count, set_count) = signal(0);

// ❌ Wrong: Don't use create_signal()
let (count, set_count) = create_signal(0);
```

### Error Handling

- Network errors caught and displayed to user
- HTTP errors (non-200 status) caught and displayed
- Parse errors caught and displayed
- Validation errors shown before API calls

### Loading States

- Global loading indicator during API calls
- Disabled buttons during loading
- Prevents duplicate submissions

---

## TODO

### High Priority

1. **File Upload Implementation** - Complete the file upload to server
   - POST /api/v1/kebutuhan-bmn/pengajuan/{id}/attachments
   - Handle upload progress
   - Handle upload errors
   - Store file metadata

2. **Period Management Page** - Implement for Validator Pusat
   - Create period form
   - BMN item selection
   - Satker selection
   - Period constraints

3. **Review Page** - Implement for Validator Wilayah
   - View submitted kebutuhan
   - Forward to Validator Pusat
   - Return to Operator Satker with revision notes

4. **Analysis Page** - Implement for Validator Pusat
   - View kebutuhan with SIMAN/MySIMKARI data
   - Gap analysis display
   - Approve/reject actions
   - Generate analysis reports

### Medium Priority

5. **Routing Integration** - Add routes to main app router
6. **Authentication** - Integrate with Authenc for role-based access
7. **Notifications** - Show real-time notifications on workflow transitions
8. **Pagination** - Add pagination for barang items list
9. **Search/Filter** - Add search and filter for barang items

### Low Priority

10. **Offline Support** - Cache data for offline viewing
11. **Export** - Export barang list to Excel/PDF
12. **Bulk Operations** - Bulk add/delete barang items

---

## Testing

### Manual Testing Checklist

- [ ] Load active pengajuan successfully
- [ ] Display period information correctly
- [ ] Add barang item with all fields
- [ ] Add barang item with optional kode_barang empty
- [ ] Validation: Empty nama barang shows error
- [ ] Validation: Empty justifikasi shows error
- [ ] Validation: Jumlah < 1 shows error
- [ ] Upload single file
- [ ] Upload multiple files
- [ ] Submit to Validator Wilayah successfully
- [ ] Period deadline enforcement (past deadline shows warning)
- [ ] Loading states display correctly
- [ ] Error messages display correctly
- [ ] Success messages display correctly

### Unit Tests (TODO)

- Test form validation logic
- Test API response parsing
- Test state management

### Integration Tests (TODO)

- Test complete submission workflow
- Test file upload workflow
- Test error handling

---

## References

- **Requirements:** `.kiro/specs/simpel-completion/requirements.md`
- **Design:** `.kiro/specs/simpel-completion/design.md`
- **Tasks:** `.kiro/specs/simpel-completion/tasks.md` (Task 7.5.2)
- **Backend API:** `layanan/perlengkapan/crates/api/src/kebutuhan_bmn/handlers.rs`
- **AGENTS.md:** Root workspace AGENTS.md for architecture patterns
