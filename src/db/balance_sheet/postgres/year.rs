use std::collections::HashMap;

use async_recursion::async_recursion;
use async_trait::async_trait;
use futures::try_join;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    db::balance_sheet::{interface::YearData, FinResRepo, MonthRepo, YearRepo},
    error::DatamizeResult,
    models::balance_sheet::{
        NetTotal, NetTotalType, SavingRatesPerPerson, YearDetail, YearSummary,
    },
};

use super::{PostgresFinResRepo, PostgresMonthRepo};

#[derive(Debug, Clone)]
pub struct PostgresYearRepo {
    pub db_conn_pool: PgPool,
    month_repo: PostgresMonthRepo,
    fin_res_repo: PostgresFinResRepo,
}

impl PostgresYearRepo {
    pub fn new(
        db_conn_pool: PgPool,
        month_repo: PostgresMonthRepo,
        fin_res_repo: PostgresFinResRepo,
    ) -> Self {
        Self {
            db_conn_pool,
            month_repo,
            fin_res_repo,
        }
    }

    #[tracing::instrument(skip(self, net_totals))]
    async fn insert_net_totals(
        &self,
        year_id: Uuid,
        net_totals: [&NetTotal; 2],
    ) -> DatamizeResult<()> {
        for nt in net_totals {
            sqlx::query!(
                r#"
                INSERT INTO balance_sheet_net_totals_years (id, type, total, percent_var, balance_var, year_id)
                VALUES ($1, $2, $3, $4, $5, $6)
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
                year_id,
            )
            .execute(&self.db_conn_pool)
            .await?;
        }

        Ok(())
    }
}

#[async_trait]
impl YearRepo for PostgresYearRepo {
    #[tracing::instrument(skip(self))]
    async fn get_years_summary(&self) -> DatamizeResult<Vec<YearSummary>> {
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
        .fetch_all(&self.db_conn_pool)
        .await?;

        for r in db_rows
            .into_iter()
            .filter(|v| v.year_id == v.net_total_year_id)
        {
            let is_net_assets_total = r.r#type == NetTotalType::Asset.to_string();
            let net_assets = match is_net_assets_total {
                true => NetTotal {
                    id: r.net_total_id,
                    net_type: r.r#type.parse().unwrap(),
                    total: r.total,
                    percent_var: r.percent_var,
                    balance_var: r.balance_var,
                },
                false => NetTotal::new_asset(),
            };

            let net_portfolio = match r.r#type == NetTotalType::Portfolio.to_string() {
                true => NetTotal {
                    id: r.net_total_id,
                    net_type: r.r#type.parse().unwrap(),
                    total: r.total,
                    percent_var: r.percent_var,
                    balance_var: r.balance_var,
                },
                false => NetTotal::new_portfolio(),
            };

            years
                .entry(r.year_id)
                .and_modify(|y| {
                    if is_net_assets_total {
                        y.net_assets = net_assets.clone();
                    } else {
                        y.net_portfolio = net_portfolio.clone();
                    }
                })
                .or_insert_with(|| YearSummary {
                    id: r.year_id,
                    year: r.year,
                    net_assets,
                    net_portfolio,
                });
        }

        let mut years = years.into_values().collect::<Vec<_>>();

        years.sort_by(|a, b| a.year.cmp(&b.year));

        Ok(years)
    }

    #[tracing::instrument(skip(self))]
    async fn get_year_data_by_number(&self, year: i32) -> DatamizeResult<YearData> {
        sqlx::query_as!(
            YearData,
            r#"
            SELECT id, year, refreshed_at
            FROM balance_sheet_years
            WHERE year = $1;
            "#,
            year
        )
        .fetch_one(&self.db_conn_pool)
        .await
        .map_err(Into::into)
    }

    #[tracing::instrument(skip_all)]
    async fn add(&self, year: &YearDetail) -> DatamizeResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO balance_sheet_years (id, year, refreshed_at)
            VALUES ($1, $2, $3);
            "#,
            year.id,
            year.year,
            year.refreshed_at,
        )
        .execute(&self.db_conn_pool)
        .await?;

        self.insert_net_totals(year.id, [&year.net_assets, &year.net_portfolio])
            .await?;

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
            .execute(&self.db_conn_pool)
            .await?;
        }

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn get(&self, year: i32) -> DatamizeResult<YearDetail> {
        let year_data = self.get_year_data_by_number(year).await?;

        let (net_totals, saving_rates, months, resources) = try_join!(
            self.get_net_totals(year_data.id),
            self.get_saving_rates(year_data.id),
            self.month_repo.get_months_of_year(year),
            self.fin_res_repo.get_from_year(year_data.year),
        )?;

        let net_assets = match net_totals
            .clone()
            .into_iter()
            .find(|nt| nt.net_type == NetTotalType::Asset)
        {
            Some(na) => na,
            None => NetTotal::new_asset(),
        };
        let net_portfolio = match net_totals
            .into_iter()
            .find(|nt| nt.net_type == NetTotalType::Portfolio)
        {
            Some(np) => np,
            None => NetTotal::new_portfolio(),
        };

        let year = YearDetail {
            id: year_data.id,
            year: year_data.year,
            refreshed_at: year_data.refreshed_at,
            net_assets,
            net_portfolio,
            saving_rates,
            resources,
            months,
        };

        Ok(year)
    }

    #[tracing::instrument(skip(self))]
    async fn get_net_totals(&self, year_id: Uuid) -> DatamizeResult<Vec<NetTotal>> {
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
        .fetch_all(&self.db_conn_pool)
        .await
        .map_err(Into::into)
    }

    #[tracing::instrument(skip(self))]
    async fn update_net_totals(&self, year: i32) -> DatamizeResult<()> {
        update_year_net_totals(self, year).await
    }

    #[tracing::instrument(skip(self))]
    async fn get_saving_rates(&self, year_id: Uuid) -> DatamizeResult<Vec<SavingRatesPerPerson>> {
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
        .fetch_all(&self.db_conn_pool)
        .await
        .map_err(Into::into)
    }

    #[tracing::instrument(skip_all)]
    async fn update_saving_rates(&self, year: &YearDetail) -> DatamizeResult<()> {
        for sr in &year.saving_rates {
            sqlx::query!(
                r#"
                INSERT INTO balance_sheet_saving_rates (id, name, savings, employer_contribution, employee_contribution, mortgage_capital, incomes, rate, year_id)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
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
                year.id
            )
            .execute(&self.db_conn_pool)
            .await?;
        }

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn update_refreshed_at(&self, year: &YearData) -> DatamizeResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO balance_sheet_years (id, year, refreshed_at)
            VALUES ($1, $2, $3)
            ON CONFLICT (id) DO UPDATE SET
            refreshed_at = EXCLUDED.refreshed_at;
            "#,
            year.id,
            year.year,
            year.refreshed_at,
        )
        .execute(&self.db_conn_pool)
        .await?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn delete(&self, year: i32) -> DatamizeResult<()> {
        sqlx::query!(
            r#"
                DELETE FROM balance_sheet_years
                WHERE year = $1
            "#,
            year,
        )
        .execute(&self.db_conn_pool)
        .await?;

        Ok(())
    }
}

#[tracing::instrument(skip(year_repo))]
#[async_recursion]
async fn update_year_net_totals(year_repo: &PostgresYearRepo, year: i32) -> DatamizeResult<()> {
    let year_data = year_repo.get_year_data_by_number(year).await?;

    let (net_totals, saving_rates, months) = try_join!(
        year_repo.get_net_totals(year_data.id),
        year_repo.get_saving_rates(year_data.id),
        year_repo.month_repo.get_months_of_year(year),
    )?;

    let net_assets = match net_totals
        .clone()
        .into_iter()
        .find(|nt| nt.net_type == NetTotalType::Asset)
    {
        Some(na) => na,
        None => NetTotal::new_asset(),
    };
    let net_portfolio = match net_totals
        .into_iter()
        .find(|nt| nt.net_type == NetTotalType::Portfolio)
    {
        Some(np) => np,
        None => NetTotal::new_portfolio(),
    };

    let mut year = YearDetail {
        id: year_data.id,
        year: year_data.year,
        refreshed_at: year_data.refreshed_at,
        net_assets,
        net_portfolio,
        saving_rates,
        resources: vec![],
        months,
    };

    if let Some(last_month) = year.get_last_month() {
        if year.needs_net_totals_update(&last_month.net_assets, &last_month.net_portfolio) {
            year.update_net_assets_with_last_month(&last_month.net_assets);
            year.update_net_portfolio_with_last_month(&last_month.net_portfolio);
        }
    }

    // Also update with previous year since we might just have updated the total balance of current year.
    if let Ok(prev_year) = year_repo.get_year_data_by_number(year.year - 1).await {
        if let Ok(prev_net_totals) = year_repo.get_net_totals(prev_year.id).await {
            if let Some(prev_net_assets) = prev_net_totals
                .iter()
                .find(|pnt| pnt.net_type == NetTotalType::Asset)
            {
                year.update_net_assets_with_previous(prev_net_assets);
            }
            if let Some(prev_net_portfolio) = prev_net_totals
                .iter()
                .find(|pnt| pnt.net_type == NetTotalType::Portfolio)
            {
                year.update_net_portfolio_with_previous(prev_net_portfolio);
            }
        }
    }

    // Should also try to update next year if it exists
    if let Ok(next_year) = year_repo.get_year_data_by_number(year.year + 1).await {
        update_year_net_totals(year_repo, next_year.year).await?;
    }

    year_repo
        .insert_net_totals(year.id, [&year.net_assets, &year.net_portfolio])
        .await?;

    Ok(())
}
