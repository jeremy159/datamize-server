use std::collections::HashMap;

use askama::Template;
use askama_axum::IntoResponse;
use axum::extract::{Query, State};
use datamize_domain::{
    Budgeter, BudgeterExt, ComputedExpenses, MonthTarget, SalaryFragment, TemplateParams,
    TotalBudgeter,
};

use crate::{
    error::DatamizeResult,
    routes::ui::{curr_month, next_month, num_to_currency, num_to_percentage, prev_month},
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
) -> DatamizeResult<impl IntoResponse> {
    let use_category_groups_as_sub_type = template_params
        .use_category_groups_as_sub_type
        .unwrap_or_default()
        .0;
    let month = template_params.month.unwrap_or_default();

    let res = template_summary_service
        .get_template_summary(month, use_category_groups_as_sub_type)
        .await?;
    let budgeters = res.budgeters().to_vec();
    let total_budgeter = res.total_budgeter().to_owned();

    Ok(SummaryTemplate {
        month,
        budgeters,
        total_budgeter,
    })
}

#[derive(Template)]
#[template(path = "pages/budget-summary.html")]
struct SummaryTemplate {
    month: MonthTarget,
    budgeters: Vec<Budgeter<ComputedExpenses>>,
    total_budgeter: TotalBudgeter<ComputedExpenses>,
}

pub struct Salary {
    name: String,
    amount: String,
    dates: Vec<String>,
}

pub fn fragmented_salary(budgeter: &Budgeter<ComputedExpenses>) -> Vec<Salary> {
    let mut map: HashMap<String, SalaryFragment> = HashMap::new();
    for payees in budgeter.fragmented_salary().values() {
        for payee in payees {
            let mut payee = payee.clone();
            if let Some(current_payee) = map.get(&payee.payee_name.clone().unwrap()) {
                payee.occurrences = [current_payee.occurrences.clone(), payee.occurrences].concat();
            }
            map.insert(payee.payee_name.clone().unwrap(), payee.clone());
        }
    }

    map.into_values()
        .map(|payee| {
            let mut occurences = payee.occurrences.clone();
            occurences.sort();
            let dates = occurences
                .into_iter()
                .map(|d| d.format("%e %b").to_string())
                .collect();

            Salary {
                name: payee.payee_name.unwrap_or(String::from("")),
                amount: num_to_currency(payee.payee_amount),
                dates,
            }
        })
        .collect()
}
