# Migration Plan: Legacy Config → Secure Config System

## Status: IN PROGRESS

## Overview

Migrate from legacy config system (`config/default.toml`, `config/production.toml`) to secure two-layer config system (`secreton.toml` + encrypted ApplicationConfig).

## Current State

### Legacy System (TO BE REMOVED)

- `config/default.toml` - All config including secrets
- `config/production.toml` - All config including secrets
- Used by: `ApiConfig::load()` in `crates/api/src/config.rs`
- Copied in: `Dockerfile`

### New System (IMPLEMENTED BUT NOT USED)

- `secreton.toml.example` - Bootstrap config template
- `BootstrapConfig` - Infrastructure only (storage, listener, seal)
- `ApplicationConfig` - Encrypted config in storage (auth, database, MFA)
- Implemented in: `crates/core/src/config/bootstrap.rs`, `crates/core/src/config/application.rs`

## Migration Steps

### Phase 1: Integration ✅

- [x] Create migration plan
- [ ] Create config adapter to bridge ApiConfig ↔ BootstrapConfig + ApplicationConfig
- [ ] Update ServiceContainer to use BootstrapConfig
- [ ] Update api_server.rs to load BootstrapConfig first
- [ ] Add migration command to CLI

### Phase 2: Testing

- [ ] Test with file backend (development)
- [ ] Test with Raft backend (production)
- [ ] Test seal/unseal workflow
- [ ] Test config migration from legacy files
- [ ] Verify all secrets encrypted in storage

### Phase 3: Cleanup

- [ ] Remove `config/` directory
- [ ] Remove legacy ApiConfig::load() method
- [ ] Update Dockerfile to only copy secreton.toml.example
- [ ] Update docker-compose files
- [ ] Update Kubernetes manifests
- [ ] Update all documentation

### Phase 4: Validation

- [ ] Run full test suite
- [ ] Test Docker build
- [ ] Test Kubernetes deployment
- [ ] Update deployment guides

## Technical Details

### Config Flow (New System)

```
1. Load BootstrapConfig from secreton.toml
   ├─ Storage backend config
   ├─ Listener config (HTTP/gRPC)
   └─ Seal config (Shamir/KMS)

2. Initialize storage backend

3. Initialize SealService

4. Check seal status
   ├─ If SEALED: Block until unsealed
   └─ If UNSEALED: Continue

5. Load ApplicationConfig from encrypted storage
   ├─ Auth config (JWT secrets)
   ├─ Database config (connection URLs)
   ├─ MFA config
   ├─ Rate limiting
   └─ CORS, logging, audit

6. Start API servers with combined config
```

### Mapping: ApiConfig → BootstrapConfig + ApplicationConfig

| ApiConfig Field | New Location | Notes |
|----------------|--------------|-------|
| `http` | BootstrapConfig.listener.http | Infrastructure |
| `grpc` | BootstrapConfig.listener.grpc | Infrastructure |
| `storage` | BootstrapConfig.storage | Infrastructure |
| `auth.jwt.secret` | ApplicationConfig.auth.jwt_secret | **ENCRYPTED** |
| `database.password` | ApplicationConfig.database.url | **ENCRYPTED** |
| `auth.mfa` | ApplicationConfig.mfa | Encrypted |
| `rate_limit` | ApplicationConfig.rate_limit | Encrypted |
| `cors` | ApplicationConfig.cors | Encrypted |
| `logging` | ApplicationConfig.logging | Encrypted |

### Files to Modify

1. **crates/api/src/config.rs**
   - Add `from_bootstrap_and_application()` method
   - Keep legacy `load()` for backward compatibility (deprecated)

2. **crates/api/src/bin/api_server.rs**
   - Load BootstrapConfig first
   - Initialize storage and seal
   - Load ApplicationConfig after unseal
   - Merge into ApiConfig

3. **crates/api/src/services.rs** (ServiceContainer)
   - Accept BootstrapConfig in constructor
   - Load ApplicationConfig internally

4. **Dockerfile**
   - Remove: `COPY config/default.toml`
   - Remove: `COPY config/production.toml`
   - Keep: `COPY secreton.toml.example`

5. **docker-compose.yml**
   - Update volume mounts
   - Update environment variables

## Backward Compatibility

During migration, support both systems:

- If `secreton.toml` exists → Use new system
- If `config/default.toml` exists → Use legacy system (with deprecation warning)
- After migration complete → Remove legacy support

## Security Improvements

✅ **Before (Legacy)**

- All secrets in plain text TOML files
- JWT secrets, database passwords visible
- Config files must be protected with file permissions

✅ **After (New System)**

- Only infrastructure config in secreton.toml (NO SECRETS)
- All secrets encrypted with AES-256-GCM
- Master key protected by Shamir Secret Sharing
- Secret Vault starts SEALED
- Secrets only accessible after unseal

## Rollback Plan

If migration fails:

1. Restore `config/` directory from backup
2. Revert code changes
3. Rebuild Docker image
4. Redeploy

Backup created automatically by migration command:

- `config_backup_YYYYMMDD_HHMMSS/`

## Timeline

- Phase 1 (Integration): 2-3 hours
- Phase 2 (Testing): 1-2 hours
- Phase 3 (Cleanup): 1 hour
- Phase 4 (Validation): 1 hour

**Total: 5-7 hours**

## Success Criteria

- [ ] API server starts with secreton.toml only
- [ ] No secrets in bootstrap config
- [ ] All secrets encrypted in storage
- [ ] Seal/unseal workflow works
- [ ] Docker build succeeds
- [ ] All tests pass
- [ ] Documentation updated
- [ ] Legacy config files removed
