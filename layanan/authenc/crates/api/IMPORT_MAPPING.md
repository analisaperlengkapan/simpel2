# Import Mapping for authenc-api Migration

## Old Import → New Import Mapping

### Error Types

- `crate::error::AuthencError` → `authenc_types::AuthencError`
- `crate::error::Result` → `authenc_types::Result`

### Database

- `crate::database::Database` → `authenc_storage::Database`

### Services

- `crate::services::*` → `authenc_core::services::*`
- `crate::services::cache::Cache` → `authenc_core::services::cache::Cache`
- `crate::services::stores::*` → `authenc_core::stores::*`
- `crate::services::user_store::UserStore` → `authenc_core::stores::UserStore`
- `crate::services::oidc_client_store::OidcClientStore` → `authenc_core::services::OidcClientStore`
- `crate::services::oidc_code_store::OidcCodeStore` → `authenc_core::services::OidcCodeStore`
- `crate::services::pg_audit_log_store::PgAuditLogStore` → `authenc_core::services::PgAuditLogStore`
- `crate::services::jwt_validator::JwtValidator` → `authenc_crypto::jwt::JwtValidator`
- `crate::services::token_exchange::*` → `authenc_core::services::TokenExchangeService`
- `crate::services::device::*` → NOT YET MIGRATED (needs implementation)

### Models

- `crate::models::*` → `authenc_types::domain::*`
- `crate::models::user::*` → `authenc_types::domain::user::*`
- `crate::models::audit_log::AuditLog` → `authenc_types::domain::audit_log::AuditLog`

### Utils

- `crate::utils::crypto_monitor::CryptoMonitor` → NOT YET MIGRATED (needs implementation)
- `crate::utils::sso_cookie::*` → `authenc_core::services::SsoCookieManager`

### Events

- `crate::events::Event` → `authenc_types::domain::Event`
- `crate::events::EventCategory` → `authenc_types::domain::EventCategory`
- `crate::events::EventType` → `authenc_types::domain::EventType`

### App State

- `crate::app::AppState` → `crate::state::ApiState`

### Handlers

- `crate::handlers::jit_admin_service` → NOT YET MIGRATED (needs implementation)

## Missing Dependencies in Cargo.toml

✅ rand - ADDED
✅ bcrypt - ADDED
✅ urlencoding - ADDED
✅ jsonwebtoken - ADDED
✅ once_cell - ADDED
✅ base64 - ADDED
✅ argon2 - ADDED

## Missing Implementations

- DeviceService (device management)
- CryptoMonitor (crypto monitoring)
- JIT Admin Service (just-in-time admin)

## Notes

- All `crate::` imports need to be updated to use the appropriate crate prefix
- Some services may not be fully migrated yet and need stub implementations
- The API state is now `ApiState` instead of `AppState`
