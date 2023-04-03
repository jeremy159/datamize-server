use axum::{
    extract::{Path, State},
    Json,
};
use axum_extra::extract::WithRejection;
use uuid::Uuid;

use crate::{
    common::{update_month_net_totals, update_year_net_totals},
    db,
    domain::{FinancialResourceYearly, Month, SaveResource},
    error::{AppError, HttpJsonAppResult, JsonError},
    startup::AppState,
};

/// Returns a specific resource.
#[tracing::instrument(name = "Get a resource", skip_all)]
pub async fn balance_sheet_resource(
    Path(resource_id): Path<Uuid>,
    State(app_state): State<AppState>,
) -> HttpJsonAppResult<FinancialResourceYearly> {
    let db_conn_pool = app_state.db_conn_pool;

    Ok(Json(
        db::get_financial_resource(&db_conn_pool, resource_id)
            .await
            .map_err(AppError::from_sqlx)?,
    ))
}

/// Updates the resource.
/// Will also update the months' and year's net totals.
#[tracing::instrument(skip_all)]
pub async fn update_balance_sheet_resource(
    Path(resource_id): Path<Uuid>,
    State(app_state): State<AppState>,
    WithRejection(Json(body), _): WithRejection<Json<SaveResource>, JsonError>,
) -> HttpJsonAppResult<FinancialResourceYearly> {
    let db_conn_pool = app_state.db_conn_pool;

    let mut resource: FinancialResourceYearly = body.into();
    resource.base.id = resource_id;

    db::get_financial_resource(&db_conn_pool, resource_id)
        .await
        .map_err(AppError::from_sqlx)?;

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

    Ok(Json(resource))
}

/// Deletes the resource and returns the entity
#[tracing::instrument(skip_all)]
pub async fn delete_balance_sheet_resource(
    Path(resource_id): Path<Uuid>,
    State(app_state): State<AppState>,
) -> HttpJsonAppResult<FinancialResourceYearly> {
    let db_conn_pool = app_state.db_conn_pool;

    let resource = db::get_financial_resource(&db_conn_pool, resource_id)
        .await
        .map_err(AppError::from_sqlx)?;
    db::delete_financial_resource(&db_conn_pool, resource_id).await?;

    Ok(Json(resource))
}
