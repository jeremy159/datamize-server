use axum::extract::{Query, State};
use datamize_domain::{BudgetSummary, TemplateParams};

use crate::{
    error::{AppJson, HttpJsonDatamizeResult},
    services::budget_template::DynTemplateSummaryService,
};

/// Returns a budget template summary.
/// Can specify the month to get summary from.
/// /template/summary?month=previous
/// Possible values to pass in query params are `previous` and `next`. If nothing is specified,
/// the current month will be used.
pub async fn template_summary(
    State(template_summary_service): State<DynTemplateSummaryService>,
    template_params: Query<TemplateParams>,
) -> HttpJsonDatamizeResult<BudgetSummary> {
    let use_category_groups_as_sub_type = template_params
        .use_category_groups_as_sub_type
        .unwrap_or_default()
        .0;
    let month = template_params.month.unwrap_or_default();

    Ok(AppJson(
        template_summary_service
            .get_template_summary(month, use_category_groups_as_sub_type)
            .await?,
    ))
}
