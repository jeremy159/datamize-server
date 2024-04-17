use std::collections::HashMap;

use crate::{error::DatamizeResult, services::budget_template::DynTemplateDetailService};
use askama::Template;
use askama_axum::IntoResponse;
use axum::extract::{Query, State};
use datamize_domain::{ExpenseType, MonthTarget, TemplateParams};
use itertools::Itertools;

use crate::routes::ui::{curr_month, next_month, num_to_currency, num_to_percentage, prev_month};

/// Returns a budget template details
/// Can specify the month to get details from.
/// /template/details?month=previous
/// Possible values to pass in query params are `previous` and `next`. If nothing is specified,
/// the current month will be used.
pub async fn template_details(
    State(template_detail_service): State<DynTemplateDetailService>,
    template_params: Query<TemplateParams>,
) -> DatamizeResult<impl IntoResponse> {
    let use_category_groups_as_sub_type = template_params
        .use_category_groups_as_sub_type
        .unwrap_or_default()
        .0;
    let month = template_params.month.unwrap_or_default();

    let res = template_detail_service
        .get_template_details(month, use_category_groups_as_sub_type)
        .await?;
    let total_row: Row = res.expenses().iter().fold(Row::default(), |mut tot, e| {
        tot.budgeted += e.projected_amount();
        tot.spent += e.current_amount();
        tot.difference += e.projected_amount() - e.current_amount();
        tot.proportion += e.current_proportion();
        tot
    });
    let total_income = res.global_metadata().total_monthly_income;
    let projected_income_left_over = total_income - total_row.budgeted;
    let income_left_over = total_income - total_row.spent;

    // TODO: Check https://htmx.org/examples/sortable/
    let mut expenses = res.expenses().to_vec();
    expenses.sort_by_key(|e| e.expense_type().clone());

    // TODO: To move to DB
    let group_orders = HashMap::from([
        (
            ExpenseType::Fixed,
            vec![
                "Maison",
                "Dépenses Courantes",
                "Santé & Beauté",
                "Transport",
                "Taxes",
                "Dépenses Fixes Divers",
            ],
        ),
        (
            ExpenseType::Variable,
            vec![
                "Dépenses Courantes Plaisir",
                "Abonnements",
                "Dépenses Imprévues",
            ],
        ),
        (
            ExpenseType::ShortTermSaving,
            vec![
                "Épargne Court Terme",
                "Objectifs Épargne",
                "Épargne/Personne",
            ],
        ),
    ]);

    let mut groups = vec![];
    for (key, group) in &expenses.iter().group_by(|e| e.expense_type()) {
        let rows = group
            .collect::<Vec<_>>()
            .into_iter()
            .cloned()
            .collect::<Vec<_>>();
        let mut sub_groups = vec![];

        for (sub_key, sub_group) in &rows.iter().group_by(|e| e.sub_expense_type()) {
            let sub_rows = sub_group
                .collect::<Vec<_>>()
                .into_iter()
                .cloned()
                .collect::<Vec<_>>();
            let total_counts = sub_rows.len();
            let mut total_row = Row {
                name: format!("Total {}", sub_key),
                ..Default::default()
            };
            for row in &sub_rows {
                total_row.budgeted += row.projected_amount();
                total_row.spent += row.current_amount();
                total_row.difference += row.projected_amount() - row.current_amount();
                total_row.proportion += row.current_proportion();
            }
            let sub_group = SubGroup {
                name: sub_key.to_string(),
                total_counts,
                rows: sub_rows
                    .into_iter()
                    .map(|e| Row {
                        name: e.name().to_string(),
                        budgeted: e.projected_amount(),
                        spent: e.current_amount(),
                        difference: e.projected_amount() - e.current_amount(),
                        proportion: e.current_proportion(),
                        target_proportion: None,
                    })
                    .collect::<Vec<_>>(),
                total_row,
            };
            sub_groups.push(sub_group);
        }

        let total_counts = rows.len();
        let mut total_row = Row {
            name: format!("Total {}", key.to_display_name()),
            ..Default::default()
        };
        for row in &rows {
            total_row.budgeted += row.projected_amount();
            total_row.spent += row.current_amount();
            total_row.difference += row.projected_amount() - row.current_amount();
            total_row.proportion += row.current_proportion();
            total_row.target_proportion = res
                .global_metadata()
                .proportion_target_per_expense_type
                .get(key)
                .cloned();
        }

        if let Some(order) = group_orders.get(key) {
            sub_groups.sort_by(|a, b| {
                let a_index = order.iter().position(|e| e == &a.name).unwrap();
                let b_index = order.iter().position(|e| e == &b.name).unwrap();
                a_index.cmp(&b_index)
            });
        }

        let group = Group {
            name: key.to_display_name(),
            total_counts,
            sub_groups,
            rows: rows
                .into_iter()
                .map(|e| Row {
                    name: e.name().to_string(),
                    budgeted: e.projected_amount(),
                    spent: e.current_amount(),
                    difference: e.projected_amount() - e.current_amount(),
                    proportion: e.current_proportion(),
                    target_proportion: None,
                })
                .collect(),
            total_row,
        };

        groups.push(group);
    }

    Ok(DetailsTemplate {
        month,
        groups,
        total_row,
        projected_income_left_over,
        income_left_over,
        total_income,
    })
}

#[derive(Template)]
#[template(path = "pages/budget-details.html")]
struct DetailsTemplate {
    month: MonthTarget,
    groups: Vec<Group>,
    total_row: Row,
    projected_income_left_over: i64,
    income_left_over: i64,
    total_income: i64,
}

#[derive(Debug, Clone, Default)]
struct Group {
    name: String,
    total_counts: usize,
    sub_groups: Vec<SubGroup>,
    rows: Vec<Row>,
    total_row: Row,
}

#[derive(Debug, Clone, Default)]
struct SubGroup {
    name: String,
    total_counts: usize,
    rows: Vec<Row>,
    total_row: Row,
}

// TODO: Check https://github.com/bigskysoftware/contact-app/blob/master/app.py for reference
#[derive(Debug, Clone, Default)]
struct Row {
    name: String,
    budgeted: i64,
    spent: i64,
    difference: i64,
    proportion: f64,
    target_proportion: Option<f64>,
}
