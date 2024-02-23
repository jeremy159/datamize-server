use serde::{Deserialize, Serialize};

pub type YnabResult<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0:?}")]
    Http(#[from] reqwest::Error),
    #[error("YNAB API Error: {0:?}")]
    Api(ApiError),
    #[error("{0:?}")]
    Conversion(#[from] serde_json::Error),
    #[error("Invalid URL Error: {0:?}")]
    UrlParse(url::ParseError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub id: String,
    pub name: String,
    pub detail: String,
}

impl ApiError {
    pub fn is_resource_not_found(&self) -> bool {
        self.id == "404.2"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    pub error: ApiError,
}
