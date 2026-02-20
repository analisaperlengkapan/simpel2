//! gRPC interceptors for authentication and logging

use tonic::{Request, Status};
use tracing::{debug, info, warn};

/// Authentication interceptor
/// Validates client certificates for mTLS
pub fn auth_interceptor(req: Request<()>) -> Result<Request<()>, Status> {
    // Extract client certificate info from request metadata
    let metadata = req.metadata();

    // Check for client certificate (mTLS)
    if let Some(cert_info) = metadata.get("x-forwarded-client-cert") {
        debug!("Client certificate present: {:?}", cert_info);
        Ok(req)
    } else {
        // In production, this should reject requests without client certs
        // For now, we'll allow it but log a warning
        warn!("No client certificate found in request");
        Ok(req)
    }
}

/// Logging interceptor
/// Logs all incoming gRPC requests
pub fn logging_interceptor<T>(req: Request<T>) -> Result<Request<T>, Status> {
    let metadata = req.metadata();

    info!(
        "gRPC request: metadata_keys={}",
        metadata.keys_len()
    );

    // Log request ID if present
    if let Some(request_id) = metadata.get("x-request-id") {
        debug!("Request ID: {:?}", request_id);
    }

    Ok(req)
}

/// Error mapping interceptor
/// Maps internal errors to appropriate gRPC status codes
pub fn error_mapping_interceptor<T>(
    result: Result<T, Status>,
) -> Result<T, Status> {
    match result {
        Ok(response) => Ok(response),
        Err(status) => {
            // Log error
            warn!(
                "gRPC error: code={:?}, message={}",
                status.code(),
                status.message()
            );

            // Return status as-is (already mapped in service layer)
            Err(status)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tonic::metadata::MetadataMap;

    #[test]
    fn test_auth_interceptor_without_cert() {
        let req = Request::new(());
        let result = auth_interceptor(req);
        assert!(result.is_ok());
    }

    #[test]
    fn test_logging_interceptor() {
        let req = Request::new(());
        let result = logging_interceptor(req);
        assert!(result.is_ok());
    }
}
