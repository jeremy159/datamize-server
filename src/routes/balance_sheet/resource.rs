use axum::{
    extract::{Path, State},
    Json,
};
use axum_extra::extract::WithRejection;
use uuid::Uuid;

use crate::{
    common::{update_month_net_totals, update_year_net_totals},
    db,
    domain::{BaseFinancialResource, FinancialResourceYearly, SaveResource},
    error::{AppError, HttpJsonAppResult, JsonError},
    startup::AppState,
};

/// Returns a specific resource.
#[tracing::instrument(name = "Get a resource", skip_all)]
pub async fn balance_sheet_resource(
    Path((year, resource_id)): Path<(i32, Uuid)>,
    State(app_state): State<AppState>,
) -> HttpJsonAppResult<FinancialResourceYearly> {
    let db_conn_pool = app_state.db_conn_pool;

    Ok(Json(
        db::get_financial_resource(&db_conn_pool, year, resource_id)
            .await
            .map_err(AppError::from_sqlx)?,
    ))
}

/// Updates the resource.
/// Will also update the months' and year's net totals.
#[tracing::instrument(skip_all)]
pub async fn update_balance_sheet_resource(
    Path((year, resource_id)): Path<(i32, Uuid)>,
    State(app_state): State<AppState>,
    WithRejection(Json(body), _): WithRejection<Json<SaveResource>, JsonError>,
) -> HttpJsonAppResult<FinancialResourceYearly> {
    let db_conn_pool = app_state.db_conn_pool;

    let resource = FinancialResourceYearly {
        base: BaseFinancialResource {
            id: resource_id,
            name: body.name,
            category: body.category,
            r_type: body.r_type,
            editable: body.editable,
        },
        year,
        balance_per_month: body.balance_per_month,
    };

    db::get_financial_resource(&db_conn_pool, year, resource_id)
        .await
        .map_err(AppError::from_sqlx)?;

    db::update_financial_resource(&db_conn_pool, &resource).await?;

    if !resource.balance_per_month.is_empty() {
        // If balance data was received, update month and year net totals
        for month in resource.balance_per_month.keys() {
            update_month_net_totals(&db_conn_pool, *month, year).await?;
        }

        update_year_net_totals(&db_conn_pool, year).await?;
    }

    Ok(Json(resource))
}

/// Deletes the resource and returns the entity
#[tracing::instrument(skip_all)]
pub async fn delete_balance_sheet_resource(
    Path((year, resource_id)): Path<(i32, Uuid)>,
    State(app_state): State<AppState>,
) -> HttpJsonAppResult<FinancialResourceYearly> {
    let db_conn_pool = app_state.db_conn_pool;

    let resource = db::get_financial_resource(&db_conn_pool, year, resource_id)
        .await
        .map_err(AppError::from_sqlx)?;
    db::delete_financial_resource(&db_conn_pool, resource_id).await?;

    Ok(Json(resource))
}
