use thiserror::Error;

use crate::{HttpRequest, HttpResponse};

///
/// Error type returned by failed reqwest HTTP requests.
///
#[derive(Debug, Error)]
pub enum Error<T>
where
    T: std::error::Error + 'static,
{
    /// Error returned by reqwest crate.
    #[error("request failed")]
    Reqwest(#[source] T),
    /// Non-reqwest HTTP error.
    #[error("HTTP error")]
    Http(#[source] http::Error),
    /// I/O error.
    #[error("I/O error")]
    Io(#[source] std::io::Error),
    /// Other error.
    #[error("Other error: {}", _0)]
    Other(String),
}

pub async fn async_http_client(
    request: HttpRequest,
) -> Result<HttpResponse, Error<reqwest::Error>> {
    let client = {
        let builder = reqwest::Client::builder();

        // Following redirects opens the client up to SSRF vulnerabilities.
        let builder = builder.redirect(reqwest::redirect::Policy::none());

        builder.build().map_err(Error::Reqwest)?
    };

    let mut request_builder = client
        .request(request.method, request.url.as_str())
        .body(request.body);
    for (name, value) in &request.headers {
        request_builder = request_builder.header(name.as_str(), value.as_bytes());
    }
    let request = request_builder.build().map_err(Error::Reqwest)?;

    let response = client.execute(request).await.map_err(Error::Reqwest)?;

    let status_code = response.status();
    let headers = response.headers().to_owned();
    let chunks = response.bytes().await.map_err(Error::Reqwest)?;

    Ok(HttpResponse {
        status_code,
        headers,
        body: chunks.to_vec(),
    })
}
