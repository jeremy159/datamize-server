mod month;
mod months;
mod refresh_resources;
mod resource;
mod resources;
mod year;
mod years;

use axum::{
    routing::{get, post},
    Router,
};
use month::*;
use months::*;
use refresh_resources::*;
use resource::*;
use resources::*;
use year::*;
use years::*;

use crate::startup::AppState;

pub fn get_balance_sheets_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/years",
            get(balance_sheet_years).post(create_balance_sheet_year),
        )
        .route(
            "/years/:year",
            get(balance_sheet_year)
                .put(update_balance_sheet_year)
                .delete(delete_balance_sheet_year),
        )
        .route("/months", get(all_balance_sheet_months))
        .route(
            "/resources",
            get(all_balance_sheet_resources).post(create_balance_sheet_resource),
        )
        .route(
            "/resources/:resource_id",
            get(balance_sheet_resource)
                .put(update_balance_sheet_resource)
                .delete(delete_balance_sheet_resource),
        )
        .route("/resources/refresh", post(refresh_balance_sheet_resources))
        .route("/years/:year/resources", get(balance_sheet_resources))
        .route(
            "/years/:year/months",
            get(balance_sheet_months).post(create_balance_sheet_month),
        )
        .route(
            "/years/:year/months/:month",
            get(balance_sheet_month).delete(delete_balance_sheet_month),
        )
}
