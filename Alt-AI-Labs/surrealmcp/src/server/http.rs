use axum::extract::Request;
use axum::http::{HeaderValue, StatusCode, header};
use axum::middleware::Next;
use axum::response::Response;

/// Health check endpoint for load balancer health status checking
pub async fn health() -> StatusCode {
    StatusCode::OK
}

/// `rmcp` streamable HTTP clients reject responses with no `Content-Type` (e.g. 429 from governor).
/// Stateful MCP POST success is `text/event-stream`; errors are plain text from rmcp or this server.
pub async fn ensure_mcp_response_content_type(req: Request, next: Next) -> Response {
    let path = req.uri().path().to_owned();
    let mut response = next.run(req).await;
    if !path.starts_with("/mcp") || response.headers().get(header::CONTENT_TYPE).is_some() {
        return response;
    }
    let value = if response.status().is_success() {
        HeaderValue::from_static("text/event-stream")
    } else {
        HeaderValue::from_static("text/plain; charset=utf-8")
    };
    response.headers_mut().insert(header::CONTENT_TYPE, value);
    response
}
