use axum::{
    extract::{Path, State},
    Json,
};
use axum_extra::extract::WithRejection;
use datamize_domain::{SavingRate, Uuid};

use crate::{
    error::{HttpJsonDatamizeResult, JsonError},
    services::balance_sheet::DynSavingRateService,
};

/// Returns a specific saving rate.
#[tracing::instrument(name = "Get a saving rate", skip_all)]
pub async fn balance_sheet_saving_rate(
    Path(saving_rate_id): Path<Uuid>,
    State(saving_rate_service): State<DynSavingRateService>,
) -> HttpJsonDatamizeResult<SavingRate> {
    Ok(Json(
        saving_rate_service.get_saving_rate(saving_rate_id).await?,
    ))
}

/// Updates the saving rate.
#[tracing::instrument(skip_all)]
pub async fn update_balance_sheet_saving_rate(
    Path(_saving_rate_id): Path<Uuid>,
    State(saving_rate_service): State<DynSavingRateService>,
    WithRejection(Json(body), _): WithRejection<Json<SavingRate>, JsonError>,
) -> HttpJsonDatamizeResult<SavingRate> {
    Ok(Json(saving_rate_service.update_saving_rate(body).await?))
}

/// Deletes the saving rate and returns the entity.
#[tracing::instrument(skip_all)]
pub async fn delete_balance_sheet_saving_rate(
    Path(saving_rate_id): Path<Uuid>,
    State(saving_rate_service): State<DynSavingRateService>,
) -> HttpJsonDatamizeResult<SavingRate> {
    Ok(Json(
        saving_rate_service
            .delete_saving_rate(saving_rate_id)
            .await?,
    ))
}
