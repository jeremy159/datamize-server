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
    domain::{FinancialResourceYearly, Month, SaveResource},
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

#[tracing::instrument(skip_all)]
pub async fn create_balance_sheet_resource(
    State(app_state): State<AppState>,
    WithRejection(Json(body), _): WithRejection<Json<SaveResource>, JsonError>,
) -> Result<impl IntoResponse, AppError> {
    let db_conn_pool = app_state.db_conn_pool;
    let resource: FinancialResourceYearly = body.into();

    // TODO: Add test to cover that month gets created when sending resource with non-existing month.
    if !resource.balance_per_month.is_empty() {
        for month in resource.balance_per_month.keys() {
            if let Err(sqlx::Error::RowNotFound) =
                db::get_month_data(&db_conn_pool, *month, resource.year).await
            {
                // If month doesn't exist, create it
                let month = Month::new(*month, resource.year);
                db::add_new_month(&db_conn_pool, &month, resource.year).await?;
            }
        }
    }

    db::update_financial_resource(&db_conn_pool, &resource).await?;

    // If balance data was received, update month and year net totals
    if !resource.balance_per_month.is_empty() {
        update_month_net_totals(
            &db_conn_pool,
            *resource.balance_per_month.first_key_value().unwrap().0,
            resource.year,
        )
        .await?;
    }

    update_year_net_totals(&db_conn_pool, resource.year).await?;

    Ok((StatusCode::CREATED, Json(resource)))
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
