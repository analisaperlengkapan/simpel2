# Known Issues & Limitations

## Current Issues

### 1. ⚠️ CRITICAL: Unseal Endpoint Extension Missing
**Status**: BLOCKING
**Severity**: CRITICAL
**Affected Endpoints**: `POST /v1/sys/unseal`

**Issue**: The unseal endpoint requires `Extension<Option<String>>` for client IP extraction (for rate limiting), but this extension is not being injected by the middleware layer.

**Error**:
```
Missing request extension: Extension of type `core::option::Option<alloc::string::String>` was not found
```

**Root Cause**:
- Middleware layer not configured in `api_server.rs`
- Extensions need to be injected via Axum middleware
- Client IP extraction middleware not implemented

**Impact**:
- Cannot unseal engine via REST API
- Unseal operations fail with 500 error
- Secret Vault remains sealed and unusable

**Workaround** (Temporary):
1. Modify `unseal_engine` function to make `client_ip` optional
2. Or implement middleware layer in `api_server.rs`

**Fix** (Recommended):
Implement middleware layer in `api_server.rs`:

```rust
use axum::middleware::Next;
use axum::http::Request;

async fn extract_client_ip<B>(
    req: Request<B>,
    next: Next,
) -> Response {
    let client_ip = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
        .or_else(|| {
            req.extensions()
                .get::<std::net::SocketAddr>()
                .map(|addr| addr.ip().to_string())
        })
        .unwrap_or_else(|| "unknown".to_string());

    let mut req = req;
    req.extensions_mut().insert(Some(client_ip));
    next.run(req).await
}

// Add to router:
app.layer(axum::middleware::from_fn(extract_client_ip))
```

---

### 2. ⚠️ Initialize Endpoint Returns No Response
**Status**: BLOCKING
**Severity**: HIGH
**Affected Endpoints**: `POST /v1/sys/init`

**Issue**: Initialize endpoint accepts requests but returns empty response body.

**Error**:
```
curl: (52) Empty reply from server
```

**Root Cause**:
- Response serialization issue
- Possible panic in handler not being caught
- Response not being properly formatted

**Impact**:
- Cannot initialize engine
- Cannot get unseal keys and root token
- Secret Vault initialization workflow broken

**Workaround**:
- Check container logs for panic messages
- Manually inspect SealService state

**Fix** (Recommended):
1. Add better error handling in `initialize_engine` function
2. Ensure response is properly serialized before sending
3. Add logging for debugging

---

### 3. ⚠️ Secret Vault Already Initialized on Restart
**Status**: EXPECTED BEHAVIOR
**Severity**: LOW
**Affected Endpoints**: `POST /v1/sys/init`

**Issue**: Once engine is initialized, subsequent init calls fail with "Attempted to initialize already initialized engine"

**Error**:
```json
{
  "error": "Attempted to initialize already initialized engine"
}
```

**Root Cause**:
- This is correct behavior - engine can only be initialized once
- In-memory storage persists during container lifetime
- On container restart, engine state is lost

**Impact**:
- Cannot reinitialize without resetting engine state
- Need to implement engine reset/rekey operations

**Workaround**:
- Restart container to reset engine state
- Implement `/v1/sys/reset` endpoint for development

**Fix** (Recommended):
- Implement persistent storage backend (PostgreSQL)
- Implement `/v1/sys/rekey` for key rotation
- Add admin endpoint to reset engine (development only)

---

## Recommended Fixes (Priority Order)

### P0 - CRITICAL (Blocks Core Functionality)

1. **Fix Unseal Extension**
   - Implement middleware layer for client IP extraction
   - Make Extension optional or provide default
   - Test unseal workflow end-to-end

2. **Fix Initialize Response**
   - Debug response serialization
   - Add proper error handling
   - Return valid JSON response

### P1 - HIGH (Affects Production Readiness)

3. **Implement Persistent Storage**
   - Replace in-memory storage with PostgreSQL
   - Persist engine state across restarts
   - Implement backup/restore

4. **Implement Middleware Layer**
   - Client IP extraction
   - Request/response logging
   - Rate limiting
   - CORS handling

### P2 - MEDIUM (Nice to Have)

5. **Implement Admin Endpoints**
   - Secret Vault reset (development only)
   - Secret Vault rekey
   - State inspection

6. **Implement Monitoring**
   - Prometheus metrics
   - Health check improvements
   - Audit logging

---

## Testing Checklist

- [ ] Initialize engine successfully
- [ ] Get unseal keys and root token
- [ ] Unseal engine with threshold shares
- [ ] Verify engine is unsealed
- [ ] Create secret
- [ ] Read secret
- [ ] List secrets
- [ ] Seal engine
- [ ] Verify engine is sealed
- [ ] Unseal engine again
- [ ] Container restart preserves state
- [ ] Audit logging works
- [ ] Rate limiting works

---

## Development Notes

### Middleware Implementation

The following middleware needs to be implemented in `api_server.rs`:

```rust
use axum::middleware::Next;
use axum::http::Request;
use tower::ServiceBuilder;

// 1. Client IP extraction
async fn extract_client_ip<B>(req: Request<B>, next: Next) -> Response { ... }

// 2. Request logging
async fn log_request<B>(req: Request<B>, next: Next) -> Response { ... }

// 3. Error handling
async fn handle_errors<B>(req: Request<B>, next: Next) -> Response { ... }

// Apply to router:
let app = router
    .layer(ServiceBuilder::new()
        .layer(axum::middleware::from_fn(extract_client_ip))
        .layer(axum::middleware::from_fn(log_request))
        .layer(axum::middleware::from_fn(handle_errors)));
```

### Extension Types

Extensions needed:
- `Extension<Option<String>>` - Client IP
- `Extension<String>` - Request ID
- `Extension<Instant>` - Request start time

---

## References

- [Axum Middleware Documentation](https://docs.rs/axum/latest/axum/middleware/)
- [Axum Extensions](https://docs.rs/axum/latest/axum/extract/struct.Extension.html)
- [Tower Middleware](https://docs.rs/tower/latest/tower/layer/trait.Layer.html)
