use crate::{error::AppJson, services::budget_template::DynTemplateDetailService};
use axum::extract::{Query, State};
use datamize_domain::{BudgetDetails, TemplateParams};

use crate::error::HttpJsonDatamizeResult;

/// Returns a budget template details
/// Can specify the month to get details from.
/// /template/details?month=previous
/// Possible values to pass in query params are `previous` and `next`. If nothing is specified,
/// the current month will be used.
pub async fn template_details(
    State(template_detail_service): State<DynTemplateDetailService>,
    template_params: Query<TemplateParams>,
) -> HttpJsonDatamizeResult<BudgetDetails> {
    let month = template_params.month.unwrap_or_default();

    Ok(AppJson(
        template_detail_service.get_template_details(month).await?,
    ))
}
