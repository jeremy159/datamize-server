use axum::{async_trait, extract::FromRequestParts, response::Redirect, routing::get, Router};
use http::request::Parts;

use crate::startup::AppState;

mod balance_sheet;
mod budget_providers;
mod budget_template;
mod fmt;
mod utils;

use balance_sheet::*;
// use budget_providers::*;
use budget_template::*;
use fmt::*;
use utils::*;

pub fn get_ui_routes(app_state: &AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(|| async { Redirect::to("/budget/summary") }))
        .nest("/budget", get_budget_template_routes(app_state))
        .nest("/balance_sheet", get_balance_sheets_routes(app_state))
    // .nest("/budget_providers", get_budget_providers_routes(app_state))
}

/// Always `true`.
pub const HX_REQUEST: &str = "HX-Request";

/// The `HX-Request` header.
///
/// This is set on every request made by htmx itself. It won't be present on
/// requests made manually, or by other libraries.
///
/// This extractor will always return a value. If the header is not present, it
/// will return `false`.
#[derive(Debug, Clone, Copy)]
pub struct HxRequest(pub bool);

#[async_trait]
impl<S> FromRequestParts<S> for HxRequest
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        if parts.headers.contains_key(HX_REQUEST) {
            return Ok(HxRequest(true));
        } else {
            return Ok(HxRequest(false));
        }
    }
}

/// Indicates that the request is via an element using `hx-boost` attribute.
///
/// See <https://htmx.org/attributes/hx-boost/> for more information.
pub const HX_BOOSTED: &str = "HX-Boosted";

/// The `HX-Boosted` header.
///
/// This is set when a request is made from an element where its parent has the
/// `hx-boost` attribute set to `true`.
///
/// This extractor will always return a value. If the header is not present, it
/// will return `false`.
///
/// See <https://htmx.org/attributes/hx-boost/> for more information.
#[derive(Debug, Clone, Copy)]
pub struct HxBoosted(pub bool);

#[async_trait]
impl<S> FromRequestParts<S> for HxBoosted
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        if parts.headers.contains_key(HX_BOOSTED) {
            return Ok(HxBoosted(true));
        } else {
            return Ok(HxBoosted(false));
        }
    }
}
