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

impl FromRef<AppState> for YearService<PostgresYearRepo> {
    fn from_ref(state: &AppState) -> Self {
        let fin_res_repo = PostgresFinResRepo::new(state.db_conn_pool.clone());
        let month_repo = PostgresMonthRepo::new(state.db_conn_pool.clone(), fin_res_repo.clone());
        Self {
            year_repo: PostgresYearRepo::new(state.db_conn_pool.clone(), month_repo, fin_res_repo),
        }
    }
}

impl FromRef<AppState> for MonthService<PostgresMonthRepo> {
    fn from_ref(state: &AppState) -> Self {
        let fin_res_repo = PostgresFinResRepo::new(state.db_conn_pool.clone());
        Self {
            month_repo: PostgresMonthRepo::new(state.db_conn_pool.clone(), fin_res_repo),
        }
    }
}

impl FromRef<AppState> for FinResService<PostgresFinResRepo, PostgresMonthRepo, PostgresYearRepo> {
    fn from_ref(state: &AppState) -> Self {
        let fin_res_repo = PostgresFinResRepo::new(state.db_conn_pool.clone());
        let month_repo = PostgresMonthRepo::new(state.db_conn_pool.clone(), fin_res_repo.clone());
        Self {
            year_repo: PostgresYearRepo::new(
                state.db_conn_pool.clone(),
                month_repo.clone(),
                fin_res_repo.clone(),
            ),
            fin_res_repo,
            month_repo,
        }
    }
}

pub fn get_balance_sheets_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/years",
            get(balance_sheet_years::<YearService<PostgresYearRepo>>)
                .post(create_balance_sheet_year::<YearService<PostgresYearRepo>>),
        )
        .route(
            "/years/:year",
            get(balance_sheet_year::<YearService<PostgresYearRepo>>)
                .put(update_balance_sheet_year::<YearService<PostgresYearRepo>>)
                .delete(delete_balance_sheet_year::<YearService<PostgresYearRepo>>),
        )
        .route(
            "/months",
            get(all_balance_sheet_months::<MonthService<PostgresMonthRepo>>),
        )
        .route(
            "/resources",
            get(all_balance_sheet_resources::<
                FinResService<PostgresFinResRepo, PostgresMonthRepo, PostgresYearRepo>,
            >)
            .post(
                create_balance_sheet_resource::<
                    FinResService<PostgresFinResRepo, PostgresMonthRepo, PostgresYearRepo>,
                >,
            ),
        )
        .route(
            "/resources/:resource_id",
            get(balance_sheet_resource::<
                FinResService<PostgresFinResRepo, PostgresMonthRepo, PostgresYearRepo>,
            >)
            .put(
                update_balance_sheet_resource::<
                    FinResService<PostgresFinResRepo, PostgresMonthRepo, PostgresYearRepo>,
                >,
            )
            .delete(
                delete_balance_sheet_resource::<
                    FinResService<PostgresFinResRepo, PostgresMonthRepo, PostgresYearRepo>,
                >,
            ),
        )
        .route(
            "/resources/refresh",
            post(
                refresh_balance_sheet_resources::<
                    FinResService<PostgresFinResRepo, PostgresMonthRepo, PostgresYearRepo>,
                >,
            ),
        )
        .route(
            "/years/:year/resources",
            get(balance_sheet_resources::<
                FinResService<PostgresFinResRepo, PostgresMonthRepo, PostgresYearRepo>,
            >),
        )
        .route(
            "/years/:year/months",
            get(balance_sheet_months::<MonthService<PostgresMonthRepo>>)
                .post(create_balance_sheet_month::<MonthService<PostgresMonthRepo>>),
        )
        .route(
            "/years/:year/months/:month",
            get(balance_sheet_month::<MonthService<PostgresMonthRepo>>)
                .delete(delete_balance_sheet_month::<MonthService<PostgresMonthRepo>>),
        )
}
