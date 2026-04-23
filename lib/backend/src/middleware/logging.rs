use lib_core::context::RequestContext;
use axum::{
    extract::Request,
    http::{Method, Response, StatusCode},
};
use futures_util::future::BoxFuture;
use http_body::Body as HttpBody;
use pin_project::pin_project;
use std::{
    pin::Pin,
    task::{Context, Poll},
    time::Instant,
};
use tower::{Layer, Service};
use tracing::{Instrument, debug, error, field, info_span};

/// Middleware for logging HTTP requests and responses
#[derive(Clone, Debug)]
pub struct RequestLogger;

impl<S> Layer<S> for RequestLogger {
    type Service = RequestLoggerMiddleware<S>;

    fn layer(&self, service: S) -> Self::Service {
        RequestLoggerMiddleware { inner: service }
    }
}

#[derive(Clone, Debug)]
pub struct RequestLoggerMiddleware<S> {
    inner: S,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for RequestLoggerMiddleware<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    ReqBody: Send + 'static,
    ResBody: HttpBody + Send + 'static,
    ResBody::Data: Send,
    ResBody::Error: std::fmt::Display,
{
    type Response = Response<ResponseBody<ResBody>>;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        // Extract context (or create default)
        let ctx = req
            .extensions()
            .get::<RequestContext>()
            .cloned()
            .unwrap_or_else(|| crate::context::request_context_from_headers(req.headers()));

        // Don't log health checks in detail
        let path = req.uri().path();
        if path.ends_with("/health") || path.ends_with("/metrics") {
            let future = self.inner.call(req);
            return Box::pin(async move {
                let res = future.await?;
                Ok(res.map(|body| {
                    ResponseBody::new(
                        body,
                        ctx.request_id.to_string(),
                        Method::GET,
                        "/".to_string(),
                        Instant::now(),
                        StatusCode::OK,
                    )
                }))
            });
        }

        let method = req.method().clone();
        let uri = req.uri().clone();
        let version = req.version();

        let remote_addr = ctx.ip_address.as_deref().unwrap_or("unknown");
        let user_agent = ctx.user_agent.as_deref().unwrap_or("-");
        let request_id = ctx.request_id.to_string();

        let span = info_span!(
            "request",
            method = %method,
            uri = %uri,
            version = ?version,
            remote_addr = %remote_addr,
            user_agent = %user_agent,
            request_id = %request_id,
            status = field::Empty,
            latency = field::Empty,
        );

        let future = self.inner.call(req);

        Box::pin(
            async move {
                let start = Instant::now();
                let res = future.await?;
                let latency = start.elapsed();
                let status = res.status();

                let (parts, body) = res.into_parts();
                let body = ResponseBody::new(
                    body,
                    request_id.clone(),
                    method.clone(),
                    uri.path().to_string(),
                    start,
                    status,
                );
                let res = Response::from_parts(parts, body);

                let latency_ms = latency.as_millis() as u64;
                let status_code = status.as_u16();

                if status_code >= 500 {
                    error!(status = status_code, latency = latency_ms, "Request failed");
                } else {
                    debug!(
                        status = status_code,
                        latency = latency_ms,
                        "Request completed"
                    );
                }

                Ok(res)
            }
            .instrument(span),
        )
    }
}

/// Wrapper around the response body to log the response
#[pin_project]
pub struct ResponseBody<B> {
    #[pin]
    inner: B,
    request_id: String,
    method: Method,
    path: String,
    start_time: Instant,
    status: StatusCode,
}

impl<B> ResponseBody<B> {
    pub fn new(
        inner: B,
        request_id: String,
        method: Method,
        path: String,
        start_time: Instant,
        status: StatusCode,
    ) -> Self {
        Self {
            inner,
            request_id,
            method,
            path,
            start_time,
            status,
        }
    }
}

impl<B> HttpBody for ResponseBody<B>
where
    B: HttpBody + 'static,
    B::Data: Send + 'static,
    B::Error: std::fmt::Debug + 'static,
{
    type Data = B::Data;
    type Error = B::Error;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
        self.project().inner.poll_frame(cx)
    }

    fn is_end_stream(&self) -> bool {
        self.inner.is_end_stream()
    }

    fn size_hint(&self) -> http_body::SizeHint {
        self.inner.size_hint()
    }
}
