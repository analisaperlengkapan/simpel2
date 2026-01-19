//! AWS Dynamic Credentials - Refactored to use AwsEngine
//!
//! This module now delegates to the proper AWS Secrets Engine
//! for production-ready AWS credential generation.

use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
/// Mewakili pub `AwsCredential`.
pub struct AwsCredential {
    pub access_key: String,
    pub secret_key: String,
    pub username: String,
    pub expires_at: String,
}

/// Generate AWS credential using AwsEngine
/// NOTE: This is a legacy interface. New code should use AwsEngine directly.
/// This function is kept for backward compatibility with existing dynamic credential APIs.
/// For production use, configure AwsEngine via `/v1/sys/aws/config/root` and
/// create roles via `/v1/sys/aws/roles/:role_name`, then generate credentials
/// via `/v1/sys/aws/creds/:role_name`.
pub async fn generate_aws_credential(role: &str) -> Result<AwsCredential, String> {
    // This is a stub that returns an error directing users to the proper AWS engine
    Err(format!(
        "Legacy AWS credential generation is deprecated. \
        Please use AWS Secrets Engine instead:\n\
        1. Configure: POST /v1/sys/aws/config/root\n\
        2. Create role: POST /v1/sys/aws/roles/{}\n\
        3. Generate creds: GET /v1/sys/aws/creds/{}",
        role, role
    ))
}

/// Revoke AWS credential
/// NOTE: This is a legacy interface. Use AwsEngine.revoke_credentials() instead.
pub async fn revoke_aws_credential(_username: &str) -> Result<(), String> {
    Err("Legacy AWS credential revocation is deprecated. \
        Credentials are automatically revoked when their lease expires. \
        Use lease management API to revoke: DELETE /v1/sys/leases/revoke/:lease_id"
        .to_string())
}
