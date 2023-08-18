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

use crate::{
    services::balance_sheet::{
        DynFinResService, DynMonthService, DynRefreshFinResService, DynYearService, FinResService,
        MonthService, RefreshFinResService, YearService,
    },
    startup::AppState,
};

pub fn get_balance_sheets_routes<S: Clone + Send + Sync + 'static>(
    app_state: &AppState,
) -> Router<S> {
    let year_service = YearService::new_arced(app_state.db_conn_pool.clone());
    let month_service = MonthService::new_arced(app_state.db_conn_pool.clone());
    let fin_res_service = FinResService::new_arced(app_state.db_conn_pool.clone());
    let refresh_fin_res_service = RefreshFinResService::new_boxed(
        app_state.db_conn_pool.clone(),
        app_state.redis_conn.clone(),
        app_state.ynab_client.clone(),
    );

    Router::new()
        .merge(get_year_routes(year_service))
        .merge(get_month_routes(month_service))
        .merge(get_fin_res_routes(fin_res_service))
        .merge(get_refresh_fin_res_routes(refresh_fin_res_service))
}

fn get_year_routes<S>(year_service: DynYearService) -> Router<S> {
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
        .with_state(year_service)
}

fn get_month_routes<S>(month_service: DynMonthService) -> Router<S> {
    Router::new()
        .route("/months", get(all_balance_sheet_months))
        .route(
            "/years/:year/months",
            get(balance_sheet_months).post(create_balance_sheet_month),
        )
        .route(
            "/years/:year/months/:month",
            get(balance_sheet_month).delete(delete_balance_sheet_month),
        )
        .with_state(month_service)
}

fn get_fin_res_routes<S>(fin_res_service: DynFinResService) -> Router<S> {
    Router::new()
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
        .route("/years/:year/resources", get(balance_sheet_resources))
        .with_state(fin_res_service)
}

fn get_refresh_fin_res_routes<S>(refresh_fin_res_service: DynRefreshFinResService) -> Router<S> {
    Router::new()
        .route("/resources/refresh", post(refresh_balance_sheet_resources))
        .with_state(refresh_fin_res_service)
}
