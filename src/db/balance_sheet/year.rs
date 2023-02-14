use std::collections::HashMap;

use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{NetTotal, NetTotalType, SavingRatesPerPerson, YearDetail, YearSummary};

pub async fn get_years_summary(db_conn_pool: &PgPool) -> Result<Vec<YearSummary>, sqlx::Error> {
    let mut years = HashMap::<Uuid, YearSummary>::new();

    let db_rows = sqlx::query!(
        r#"
        SELECT
            y.id as year_id,
            y.year,
            n.id as net_total_id,
            n.type,
            n.total,
            n.percent_var,
            n.balance_var,
            n.year_id as net_total_year_id
        FROM balance_sheet_years AS y
        JOIN balance_sheet_net_totals_years AS n ON year_id = n.year_id;
        "#
    )
    .fetch_all(db_conn_pool)
    .await?;

    for r in db_rows
        .into_iter()
        .filter(|v| v.year_id == v.net_total_year_id)
    {
        let net_total = NetTotal {
            id: r.net_total_id,
            net_type: r.r#type.parse().unwrap(),
            total: r.total,
            percent_var: r.percent_var,
            balance_var: r.balance_var,
        };

        years
            .entry(r.year_id)
            .and_modify(|y| {
                y.net_totals.push(net_total.clone());
            })
            .or_insert_with(|| YearSummary {
                id: r.year_id,
                year: r.year,
                net_totals: vec![net_total],
            });
    }

    let mut years = years.into_values().collect::<Vec<_>>();

    years.sort_by(|a, b| a.year.cmp(&b.year));

    Ok(years)
}

#[derive(sqlx::FromRow, Debug)]
pub struct YearData {
    pub id: Uuid,
    pub year: i32,
}

pub async fn get_year_data(
    db_conn_pool: &PgPool,
    year: i32,
) -> Result<Option<YearData>, sqlx::Error> {
    sqlx::query_as!(
        YearData,
        r#"
        SELECT *
        FROM balance_sheet_years
        WHERE year = $1;
        "#,
        year
    )
    .fetch_optional(db_conn_pool)
    .await
}

pub async fn add_new_year(db_conn_pool: &PgPool, year: &YearDetail) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO balance_sheet_years (id, year)
        VALUES ($1, $2);
        "#,
        year.id,
        year.year,
    )
    .execute(db_conn_pool)
    .await?;

    for nt in &year.net_totals {
        sqlx::query!(
            r#"
            INSERT INTO balance_sheet_net_totals_years (id, type, total, percent_var, balance_var, year_id)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            nt.id,
            nt.net_type.to_string(),
            nt.total,
            nt.percent_var,
            nt.balance_var,
            year.id,
        )
        .execute(db_conn_pool)
        .await?;
    }

    for sr in &year.saving_rates {
        sqlx::query!(
            r#"
            INSERT INTO balance_sheet_saving_rates (id, name, savings, employer_contribution, employee_contribution, mortgage_capital, incomes, rate, year_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
            sr.id,
            sr.name,
            sr.savings,
            sr.employer_contribution,
            sr.employee_contribution,
            sr.mortgage_capital,
            sr.incomes,
            sr.rate,
            year.id,
        )
        .execute(db_conn_pool)
        .await?;
    }

    Ok(())
}

pub async fn get_year_net_totals_for(
    db_conn_pool: &PgPool,
    year_id: Uuid,
) -> Result<Vec<NetTotal>, sqlx::Error> {
    sqlx::query_as!(
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
        year_id,
    )
    .fetch_all(db_conn_pool)
    .await
}

pub async fn update_year_net_totals(
    db_conn_pool: &PgPool,
    net_totals: &[NetTotal],
) -> Result<(), sqlx::Error> {
    for nt in net_totals {
        sqlx::query!(
            r#"
            INSERT INTO balance_sheet_net_totals_years (id, type, total, percent_var, balance_var)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (id) DO UPDATE
            SET type = EXCLUDED.type,
            total = EXCLUDED.total,
            percent_var = EXCLUDED.percent_var,
            balance_var = EXCLUDED.balance_var;
            "#,
            nt.id,
            nt.net_type.to_string(),
            nt.total,
            nt.percent_var,
            nt.balance_var,
        )
        .execute(db_conn_pool)
        .await?;
    }

    Ok(())
}

pub async fn get_saving_rates_for(
    db_conn_pool: &PgPool,
    year_id: Uuid,
) -> Result<Vec<SavingRatesPerPerson>, sqlx::Error> {
    sqlx::query_as!(
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
        year_id,
    )
    .fetch_all(db_conn_pool)
    .await
}

pub async fn update_saving_rates(
    db_conn_pool: &PgPool,
    saving_rates: &[SavingRatesPerPerson],
) -> Result<(), sqlx::Error> {
    for sr in saving_rates {
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
        .execute(db_conn_pool)
        .await?;
    }

    Ok(())
}
