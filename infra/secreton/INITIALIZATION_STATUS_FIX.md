# Fix: Vault Initialization Status Bug

## Problem

**CRITICAL SECURITY BUG**: Vault menunjukkan status `initialized: true` bahkan saat pertama kali dijalankan (fresh start), padahal seharusnya `initialized: false` sampai operator melakukan init dan generate master key.

## Root Cause

Di `crates/core/src/services/seal.rs`, fungsi `SealStatus::new()` **hardcoded** `initialized: true`:

```rust
impl SealStatus {
    fn new(config: &SealConfig, state: SealState, progress: usize) -> Self {
        Self {
            state,
            seal_type: config.seal_type.clone(),
            initialized: true,  // ❌ SELALU TRUE!
            // ...
        }
    }
}
```

## Solution

Status `initialized` sekarang ditentukan berdasarkan keberadaan **Shamir commitment** di memory:

```rust
impl SealStatus {
    fn new(config: &SealConfig, state: SealState, progress: usize, initialized: bool) -> Self {
        Self {
            state,
            seal_type: config.seal_type.clone(),
            initialized,  // ✅ Parameter dinamis
            // ...
        }
    }
}

pub async fn status(&self) -> SealStatus {
    let config = self.config.read().await;
    let state = self.state.read().await;
    let shares = self.unseal_shares.read().await;

    // Check if vault is initialized by checking if commitment exists
    let commitment = self.commitment.read().await;
    let initialized = commitment.is_some();  // ✅ Cek commitment
    drop(commitment);

    SealStatus::new(&config, state.clone(), shares.len(), initialized)
}
```

## Behavior After Fix

### Fresh Vault (Belum Initialize)
```bash
GET /v1/sys/seal-status
{
  "initialized": false,  # ✅ Correct
  "sealed": true,
  "t": 3,
  "n": 5,
  "progress": 0
}
```

### After Initialization
```bash
POST /v1/sys/init
{
  "secret_shares": 5,
  "secret_threshold": 3
}

# Response: keys + root_token

GET /v1/sys/seal-status
{
  "initialized": true,   # ✅ Now true
  "sealed": true,        # ✅ Remains sealed (security best practice)
  "t": 3,
  "n": 5,
  "progress": 0
}
```

### After Restart
```bash
# Container restart
docker-compose restart secreton

GET /v1/sys/seal-status
{
  "initialized": true,   # ✅ Persists from storage
  "sealed": true,        # ✅ Sealed after restart
  "t": 3,
  "n": 5,
  "progress": 0
}
```

## Security Implications

### Before Fix (VULNERABLE)
- Vault menunjukkan `initialized: true` pada fresh start
- Operator bisa bingung apakah vault sudah di-setup atau belum
- Potensi operator skip initialization process
- Tidak jelas apakah vault state valid atau corrupt

### After Fix (SECURE)
- ✅ Fresh vault jelas menunjukkan `initialized: false`
- ✅ Operator tahu harus melakukan init terlebih dahulu
- ✅ Status `initialized` akurat mencerminkan vault state
- ✅ Initialization state persists across restarts

## Testing

### Unit Tests
```bash
cargo test -p secreton-core --test seal_initialized_status_test
```

Tests yang dijalankan:
1. ✅ `test_fresh_vault_not_initialized` - Fresh vault NOT initialized
2. ✅ `test_vault_initialized_after_init` - Initialized after init
3. ✅ `test_vault_initialized_persists_across_restarts` - State persists
4. ✅ `test_cannot_initialize_twice` - Re-init behavior documented

### Integration Test
```bash
./test_initialized_status.sh
```

Test flow:
1. Fresh start → `initialized: false`
2. POST /v1/sys/init → Generate keys
3. Check status → `initialized: true`, `sealed: true`
4. Unseal with shares → `sealed: false`
5. Restart container → `initialized: true`, `sealed: true`

## Files Changed

1. `crates/core/src/services/seal.rs`
   - Modified `SealStatus::new()` to accept `initialized` parameter
   - Modified `status()` to check commitment existence

2. `crates/core/tests/seal_initialized_status_test.rs` (NEW)
   - Comprehensive unit tests for initialization status

3. `test_initialized_status.sh` (NEW)
   - Integration test script for Docker environment

## HashiCorp Vault Compatibility

Behavior sekarang sesuai dengan HashiCorp Vault:

```bash
# Fresh Vault
$ vault status
Initialized: false  # ✅ Matches our behavior
Sealed: true

# After init
$ vault operator init
Unseal Key 1: ...
Root Token: ...

$ vault status
Initialized: true   # ✅ Matches our behavior
Sealed: true        # ✅ Remains sealed after init
```

## Conclusion

Fix ini memastikan bahwa:
1. ✅ Vault initialization status akurat
2. ✅ Operator mendapat feedback yang jelas
3. ✅ Security best practices diikuti (vault remains sealed after init)
4. ✅ Behavior konsisten dengan HashiCorp Vault
5. ✅ State persistence bekerja dengan benar

**Status**: ✅ FIXED and TESTED

