## 2025-05-15 - Request Size Limit Bypass via Chunked Encoding
**Vulnerability:** Request body size limits implemented solely via checking the `Content-Length` header can be bypassed by using chunked transfer encoding or by simply omitting the header, leading to potential Denial of Service (DoS) through resource exhaustion.
**Learning:** In the Axum/Hyper ecosystem, middleware must wrap the request body stream with a limiting layer (e.g., `http_body_util::Limited`) to ensure enforcement during data transfer, regardless of what the client claims in headers.
**Prevention:** Always use streaming-level body limits in addition to pre-emptive header checks. Use `http_body_util::Limited` to wrap the body in Axum middleware.
