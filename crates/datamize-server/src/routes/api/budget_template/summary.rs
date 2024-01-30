use axum::{
    extract::{Query, State},
    Json,
};
use datamize_domain::{BudgetSummary, TemplateParams};

use crate::{error::HttpJsonDatamizeResult, services::budget_template::DynTemplateSummaryService};

/// Returns a budget template summary.
/// Can specify the month to get summary from.
/// /template/summary?month=previous
/// Possible values to pass in query params are `previous` and `next`. If nothing is specified,
/// the current month will be used.
pub async fn template_summary(
    State(template_summary_service): State<DynTemplateSummaryService>,
    template_params: Query<TemplateParams>,
) -> HttpJsonDatamizeResult<BudgetSummary> {
    let month = template_params.month.unwrap_or_default();

    Ok(Json(
        template_summary_service.get_template_summary(month).await?,
    ))
}
