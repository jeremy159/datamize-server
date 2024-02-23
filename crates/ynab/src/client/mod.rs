use crate::error::{ApiErrorResponse, Error, YnabResult};
use reqwest::{header, Client as ReqwestClient, RequestBuilder, Url};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

mod accounts;
mod budgets;
mod categories;
mod hybrid_transactions;
mod months;
mod payee_locations;
mod payees;
mod scheduled_transactions;
mod transactions;
mod user;

pub use accounts::*;
pub use budgets::*;
pub use categories::*;
pub use hybrid_transactions::*;
pub use months::*;
pub use payee_locations::*;
pub use payees::*;
pub use scheduled_transactions::*;
pub use transactions::*;
pub use user::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Response<T> {
    data: T,
}

#[derive(Debug)]
pub struct Client {
    ynab_api_token: String,
    ynab_base_url: Url,
    default_budget_id: Option<String>,
    http_client: ReqwestClient,
}

impl Client {
    pub fn new(ynab_api_token: &str, ynab_base_url: Option<&str>) -> YnabResult<Self> {
        let ynab_base_url = ynab_base_url
            .unwrap_or("https://api.youneedabudget.com/v1/")
            .parse()
            .map_err(Error::UrlParse)?;
        let http_client = Client::build_http_client()?;

        Ok(Self {
            ynab_api_token: ynab_api_token.into(),
            ynab_base_url,
            http_client,
            default_budget_id: None,
        })
    }

    /// Returns the `default_budget_id` if previously set, or the special
    /// string `"last-used"` to be used in YNAB's API.
    fn get_budget_id(&self) -> &str {
        match self.default_budget_id.as_ref() {
            Some(b_id) => b_id,
            None => "last-used",
        }
    }

    pub fn with_default_budget_id(mut self, default_budget_id: &str) -> Self {
        self.default_budget_id = Some(default_budget_id.to_string());

        self
    }

    /// Builds the ReqwestClient with some default headers staying the same for all requests.
    fn build_http_client() -> YnabResult<ReqwestClient> {
        let mut headers = header::HeaderMap::new();
        let cargo_pkg_name = env!("CARGO_PKG_NAME");
        let cargo_pkg_version = env!("CARGO_PKG_VERSION");
        let user_agent_value = format!("{}/{}", cargo_pkg_name, cargo_pkg_version);
        headers.insert(
            reqwest::header::USER_AGENT,
            header::HeaderValue::from_str(&user_agent_value).unwrap(),
        );

        let http_client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;

        Ok(http_client)
    }

    /// Builds a `GET` request by joining the path to `ynab_base_url` and setting `ynab_api_token` as a bearer_auth token.
    fn get(&self, path: &str) -> RequestBuilder {
        self.http_client
            .get(self.ynab_base_url.join(path).unwrap())
            .bearer_auth(self.ynab_api_token.as_str())
    }

    /// Builds a `GET` request by joining the path to `ynab_base_url` and setting `ynab_api_token` as a bearer_auth token.
    /// Also adds some query string to the request.
    /// # Example
    ///
    /// ```rust
    /// # use ynab::YnabResult;
    /// # use ynab::Client;
    /// # async fn run() -> YnabResult<()> {
    /// let body = get_with_query("path/to/resource", &[("key", "val")]).send().await?.text().await?;
    /// # Ok(())
    /// # }
    /// ```
    /// Calling `get_with_query("path/to/resource", &[("foo", "a"), ("boo", "b")])` gives `"path/to/resource?foo=a&boo=b"`
    fn get_with_query<T: Serialize + ?Sized>(&self, path: &str, query: &T) -> RequestBuilder {
        self.http_client
            .get(self.ynab_base_url.join(path).unwrap())
            .query(query)
            .bearer_auth(self.ynab_api_token.as_str())
    }

    /// Builds a `POST` request by joining the path to `ynab_base_url` and setting the body (if present) as json data.
    /// Also sets `ynab_api_token` as a bearer_auth token.
    fn post<T>(&self, path: &str, body: Option<&T>) -> RequestBuilder
    where
        T: Serialize,
    {
        match body {
            Some(b) => self
                .http_client
                .post(self.ynab_base_url.join(path).unwrap())
                .bearer_auth(self.ynab_api_token.as_str())
                .json(b),
            None => self
                .http_client
                .post(self.ynab_base_url.join(path).unwrap())
                .bearer_auth(self.ynab_api_token.as_str()),
        }
    }

    /// Builds a `PATCH` request by joining the path to `ynab_base_url` and setting the body as json data.
    /// Also sets `ynab_api_token` as a bearer_auth token.
    fn patch<T>(&self, path: &str, body: &T) -> RequestBuilder
    where
        T: Serialize,
    {
        self.http_client
            .patch(self.ynab_base_url.join(path).unwrap())
            .bearer_auth(self.ynab_api_token.as_str())
            .json(body)
    }

    /// Builds a `PUT` request by joining the path to `ynab_base_url` and setting the body as json data.
    /// Also sets `ynab_api_token` as a bearer_auth token.
    fn put<T>(&self, path: &str, body: &T) -> RequestBuilder
    where
        T: Serialize,
    {
        self.http_client
            .put(self.ynab_base_url.join(path).unwrap())
            .bearer_auth(self.ynab_api_token.as_str())
            .json(body)
    }

    /// Converts a string body into a rust's T representation of it.
    /// If the body contains an error from the api, it will return an `Error::Api()` enum.
    /// If the conversion fails using serde, it will return an `Error::Conversion()` enum.
    /// See https://github.com/serde-rs/json/issues/450#issuecomment-506505388 for an explanation on why T
    /// has to implement DeserializeOwned and not Deserialize<'de>
    fn convert_resp<T, B>(body: B) -> YnabResult<T>
    where
        T: DeserializeOwned,
        B: AsRef<str>,
    {
        let resp: T = serde_json::from_str(body.as_ref()).map_err(|e| {
            let err = serde_json::from_str::<ApiErrorResponse>(body.as_ref());

            match err {
                Ok(api_err) => Error::Api(api_err.error),
                Err(_err) => Error::Conversion(e),
            }
        })?;

        Ok(resp)
    }
}
