use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_extra::extract::WithRejection;

use crate::{
    common::{update_month_net_totals, update_year_net_totals},
    db,
    domain::{FinancialResourceYearly, SaveResource},
    error::{AppError, HttpJsonAppResult, JsonError},
    startup::AppState,
};

/// Returns all resources of all years.
#[tracing::instrument(name = "Get all resources from all years", skip_all)]
pub async fn all_balance_sheet_resources(
    State(app_state): State<AppState>,
) -> HttpJsonAppResult<Vec<FinancialResourceYearly>> {
    let db_conn_pool = app_state.db_conn_pool;

    Ok(Json(
        db::get_all_financial_resources_of_all_years(&db_conn_pool).await?,
    ))
}

/// Endpoint to get all financial resources of a particular year.
#[tracing::instrument(name = "Get all resources from a year", skip_all)]
pub async fn balance_sheet_resources(
    Path(year): Path<i32>,
    State(app_state): State<AppState>,
) -> HttpJsonAppResult<Vec<FinancialResourceYearly>> {
    let db_conn_pool = app_state.db_conn_pool;

    Ok(Json(
        db::get_financial_resources_of_year(&db_conn_pool, year).await?,
    ))
}

#[tracing::instrument(skip_all)]
pub async fn create_balance_sheet_resource(
    Path(year): Path<i32>,
    State(app_state): State<AppState>,
    WithRejection(Json(body), _): WithRejection<Json<SaveResource>, JsonError>,
) -> Result<impl IntoResponse, AppError> {
    let db_conn_pool = app_state.db_conn_pool;
    let resource = body.to_financial_resource_yearly(year);

    db::update_financial_resource(&db_conn_pool, &resource).await?;

    if !resource.balance_per_month.is_empty() {
        // If balance data was received, update month and year net totals
        for month in resource.balance_per_month.keys() {
            update_month_net_totals(&db_conn_pool, *month, year).await?;
        }

        update_year_net_totals(&db_conn_pool, year).await?;
    }

    Ok((StatusCode::CREATED, Json(resource)))
}
