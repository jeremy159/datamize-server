mod async_client;

pub use async_client::*;

///
/// An HTTP request.
///
#[derive(Clone, Debug)]
pub struct HttpRequest {
    /// URL to which the HTTP request is being made.
    pub url: url::Url,
    /// HTTP request method for this request.
    pub method: http::method::Method,
    /// HTTP request headers to send.
    pub headers: http::header::HeaderMap,
    /// HTTP request body (typically for POST requests only).
    pub body: Vec<u8>,
}

///
/// An HTTP response.
///
#[derive(Clone, Debug)]
pub struct HttpResponse {
    /// HTTP status code returned by the server.
    pub status_code: http::status::StatusCode,
    /// HTTP response headers returned by the server.
    pub headers: http::header::HeaderMap,
    /// HTTP response body returned by the server.
    pub body: Vec<u8>,
}
