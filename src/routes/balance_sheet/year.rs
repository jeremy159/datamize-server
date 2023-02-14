use std::collections::HashMap;

use axum::{
    extract::{Path, State},
    Json,
};
use futures::try_join;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::{
    domain::{
        FinancialResource, Month, MonthNum, NetTotal, NetTotalType, ResourceCategory, ResourceType,
        SavingRatesPerPerson, UpdateYear, YearDetail,
    },
    error::HttpJsonAppResult,
    startup::AppState,
};

/// Returns a detailed year with its balance sheet and its saving rates.
pub async fn balance_sheet_year(
    Path(year): Path<i32>,
    State(app_state): State<AppState>,
) -> HttpJsonAppResult<YearDetail> {
    let db_conn_pool = app_state.db_conn_pool;

    #[derive(sqlx::FromRow, Debug)]
    struct YearData {
        id: Uuid,
        year: i32,
    }

    let Some(year_data) = sqlx::query_as!(
        YearData,
        r#"
        SELECT *
        FROM balance_sheet_years
        WHERE year = $1;
        "#,
        year
    )
    .fetch_optional(&db_conn_pool)
    .await? else {
        return Err(crate::error::AppError::ResourceNotFound);
    };

    let net_totals_query = sqlx::query_as!(
        NetTotal,
        r#"
        SELECT
            id,
            type AS "net_type: NetTotalType",
            total,
            percent_var,
            balance_var
        FROM balance_sheet_net_totals_years
        WHERE year_id = $1;
        "#,
        year_data.id,
    )
    .fetch_all(&db_conn_pool);

    let saving_rates_query = sqlx::query_as!(
        SavingRatesPerPerson,
        r#"
        SELECT
            id,
            name,
            savings,
            employer_contribution,
            employee_contribution,
            mortgage_capital,
            incomes,
            rate
        FROM balance_sheet_saving_rates
        WHERE year_id = $1;
        "#,
        year_data.id,
    )
    .fetch_all(&db_conn_pool);

    let (net_totals, saving_rates) = try_join!(net_totals_query, saving_rates_query)?;
    let months = build_months(&db_conn_pool, year_data.id).await?;

    Ok(Json(YearDetail {
        id: year_data.id,
        year: year_data.year,
        net_totals,
        saving_rates,
        months,
    }))
}

pub async fn build_months(
    db_conn_pool: &Pool<Postgres>,
    year_id: Uuid,
) -> anyhow::Result<Vec<Month>> {
    #[derive(sqlx::FromRow, Debug)]
    struct MonthData {
        id: Uuid,
        month: i16,
    }

    let months_data = sqlx::query_as!(
        MonthData,
        r#"
        SELECT 
            id,
            month
        FROM balance_sheet_months
        WHERE year_id = $1;
        "#,
        year_id
    )
    .fetch_all(db_conn_pool)
    .await?;

    let mut months = HashMap::<Uuid, Month>::with_capacity(months_data.len());

    for month_data in &months_data {
        let net_totals_query = sqlx::query_as!(
            NetTotal,
            r#"
            SELECT
                id,
                type AS "net_type: NetTotalType",
                total,
                percent_var,
                balance_var
            FROM balance_sheet_net_totals_months
            WHERE month_id = $1;
            "#,
            month_data.id,
        )
        .fetch_all(db_conn_pool);

        let financial_resources_query = sqlx::query_as!(
            FinancialResource,
            r#"
            SELECT
                id,
                name,
                category AS "category: ResourceCategory",
                type AS "resource_type: ResourceType",
                balance,
                editable
            FROM balance_sheet_resources
            WHERE month_id = $1;
            "#,
            month_data.id,
        )
        .fetch_all(db_conn_pool);

        let (net_totals, resources) = try_join!(net_totals_query, financial_resources_query)?;

        months
            .entry(month_data.id)
            .and_modify(|m| {
                m.net_totals.extend(net_totals.clone());
                m.resources.extend(resources.clone())
            })
            .or_insert_with(|| Month {
                id: month_data.id,
                month: MonthNum::try_from(month_data.month).unwrap(),
                net_totals,
                resources,
            });
    }

    let mut months = months.into_values().collect::<Vec<_>>();

    months.sort_by(|a, b| a.month.cmp(&b.month));

    Ok(months)
}

// TODO: To recompute net totals of current year after update.
/// Updates the saving rates of the received year.
pub async fn update_balance_sheet_year(
    Path(year): Path<i32>,
    State(app_state): State<AppState>,
    Json(body): Json<UpdateYear>,
) -> HttpJsonAppResult<YearDetail> {
    let db_conn_pool = app_state.db_conn_pool;

    #[derive(sqlx::FromRow, Debug)]
    struct YearData {
        id: Uuid,
        year: i32,
    }

    let Some(year_data) = sqlx::query_as!(
        YearData,
        r#"
        SELECT *
        FROM balance_sheet_years
        WHERE year = $1;
        "#,
        year
    )
    .fetch_optional(&db_conn_pool)
    .await? else {
        return Err(crate::error::AppError::ResourceNotFound);
    };

    let net_totals = sqlx::query_as!(
        NetTotal,
        r#"
        SELECT
            id,
            type AS "net_type: NetTotalType",
            total,
            percent_var,
            balance_var
        FROM balance_sheet_net_totals_years
        WHERE year_id = $1;
        "#,
        year_data.id,
    )
    .fetch_all(&db_conn_pool)
    .await?;

    for sr in &body.saving_rates {
        sqlx::query!(
            r#"
            INSERT INTO balance_sheet_saving_rates (id, name, savings, employer_contribution, employee_contribution, mortgage_capital, incomes, rate)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (id) DO UPDATE
            SET name = EXCLUDED.name,
            savings = EXCLUDED.savings,
            employer_contribution = EXCLUDED.employer_contribution,
            employee_contribution = EXCLUDED.employee_contribution,
            mortgage_capital = EXCLUDED.mortgage_capital,
            incomes = EXCLUDED.incomes,
            rate = EXCLUDED.rate;
            "#,
            sr.id,
            sr.name,
            sr.savings,
            sr.employer_contribution,
            sr.employee_contribution,
            sr.mortgage_capital,
            sr.incomes,
            sr.rate,
        )
        .execute(&db_conn_pool)
        .await?;
    }

    let months = build_months(&db_conn_pool, year_data.id).await?;

    Ok(Json(YearDetail {
        id: year_data.id,
        year: year_data.year,
        net_totals,
        saving_rates: body.saving_rates,
        months,
    }))
}
