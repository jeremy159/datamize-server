mod month;
mod months;
mod refresh_resources;
mod resource;
mod resources;
mod year;
mod years;

use axum::{
    extract::FromRef,
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

use crate::{
    db::balance_sheet::{PostgresFinResRepo, PostgresMonthRepo, PostgresYearRepo},
    services::balance_sheet::{FinResService, MonthService, YearService},
    startup::AppState,
};

impl FromRef<AppState> for YearService {
    fn from_ref(state: &AppState) -> Self {
        let fin_res_repo = PostgresFinResRepo::new(state.db_conn_pool.clone());
        let month_repo = PostgresMonthRepo::new(state.db_conn_pool.clone(), fin_res_repo.clone());
        Self {
            year_repo: Box::new(PostgresYearRepo::new(
                state.db_conn_pool.clone(),
                month_repo,
                fin_res_repo,
            )),
        }
    }
}

impl FromRef<AppState> for MonthService {
    fn from_ref(state: &AppState) -> Self {
        let fin_res_repo = PostgresFinResRepo::new(state.db_conn_pool.clone());
        Self {
            month_repo: Box::new(PostgresMonthRepo::new(
                state.db_conn_pool.clone(),
                fin_res_repo,
            )),
        }
    }
}

impl FromRef<AppState> for FinResService {
    fn from_ref(state: &AppState) -> Self {
        let fin_res_repo = PostgresFinResRepo::new(state.db_conn_pool.clone());
        let month_repo = PostgresMonthRepo::new(state.db_conn_pool.clone(), fin_res_repo.clone());
        Self {
            year_repo: Box::new(PostgresYearRepo::new(
                state.db_conn_pool.clone(),
                month_repo.clone(),
                fin_res_repo.clone(),
            )),
            fin_res_repo: Box::new(fin_res_repo),
            month_repo: Box::new(month_repo),
        }
    }
}

pub fn get_balance_sheets_routes(app_state: &AppState) -> Router<AppState> {
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
