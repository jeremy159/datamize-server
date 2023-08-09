use axum::{
    extract::{Query, State},
    Json,
};

use crate::{
    error::HttpJsonDatamizeResult,
    models::budget_template::{BudgetSummary, MonthQueryParam},
    services::budget_template::{TemplateSummaryService, TemplateSummaryServiceExt},
};

/// Returns a budget template summary.
/// Can specify the month to get summary from.
/// /template/summary?month=previous
/// Possible values to pass in query params are `previous` and `next`. If nothing is specified,
/// the current month will be used.
pub async fn template_summary(
    State(mut template_summary_service): State<TemplateSummaryService>,
    month: Option<Query<MonthQueryParam>>,
) -> HttpJsonDatamizeResult<BudgetSummary> {
    let Query(MonthQueryParam(month)) = month.unwrap_or_default();

    Ok(Json(
        template_summary_service.get_template_summary(month).await?,
    ))
}
