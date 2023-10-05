use std::{collections::HashMap, sync::Arc};

use async_recursion::async_recursion;
use chrono::{DateTime, Utc};
use datamize_domain::db::{DbResult, MonthRepo, YearData, YearRepo};
use datamize_domain::{async_trait, NetTotal, NetTotalType, Uuid, Year};
use futures::try_join;
use futures::{stream::FuturesUnordered, StreamExt};
use sqlx::SqlitePool;

use super::{SqliteFinResRepo, SqliteMonthRepo};

#[derive(Debug, Clone)]
pub struct SqliteYearRepo {
    pub db_conn_pool: SqlitePool,
    pub month_repo: SqliteMonthRepo,
}

impl SqliteYearRepo {
    pub fn new_arced(db_conn_pool: SqlitePool) -> Arc<Self> {
        Arc::new(Self {
            db_conn_pool: db_conn_pool.clone(),
            month_repo: SqliteMonthRepo {
                db_conn_pool: db_conn_pool.clone(),
                fin_res_repo: SqliteFinResRepo { db_conn_pool },
            },
        })
    }

    #[tracing::instrument(skip(self, net_totals))]
    async fn insert_net_totals(&self, year_id: Uuid, net_totals: [&NetTotal; 2]) -> DbResult<()> {
        for nt in net_totals {
            let net_type = nt.net_type.to_string();
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
                net_type,
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
impl YearRepo for SqliteYearRepo {
    #[tracing::instrument(skip(self))]
    async fn get_years(&self) -> DbResult<Vec<Year>> {
        let mut years = HashMap::<Uuid, Year>::new();

        let db_rows = sqlx::query!(
            r#"
        SELECT
            y.id as "year_id: Uuid",
            y.year as "year: i32",
            y.refreshed_at as "refreshed_at: DateTime<Utc>",
            n.id as "net_total_id: Uuid",
            n.type as "type: NetTotalType",
            n.total,
            n.percent_var as "percent_var: f32",
            n.balance_var,
            n.year_id as "net_total_year_id: Uuid"
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
            let is_net_assets_total = r.r#type == NetTotalType::Asset;
            let net_assets = match is_net_assets_total {
                true => NetTotal {
                    id: r.net_total_id,
                    net_type: r.r#type.clone(),
                    total: r.total,
                    percent_var: r.percent_var,
                    balance_var: r.balance_var,
                },
                false => NetTotal::new_asset(),
            };

            let net_portfolio = match r.r#type == NetTotalType::Portfolio {
                true => NetTotal {
                    id: r.net_total_id,
                    net_type: r.r#type,
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
                .or_insert_with(|| Year {
                    id: r.year_id,
                    year: r.year,
                    refreshed_at: r.refreshed_at,
                    net_assets,
                    net_portfolio,
                    months: vec![],
                });
        }

        let mut years = years.into_values().collect::<Vec<_>>();

        years.sort_by(|a, b| a.year.cmp(&b.year));

        let months_stream = years
            .iter()
            .map(|y| self.month_repo.get_months_of_year(y.year))
            .collect::<FuturesUnordered<_>>()
            .collect::<Vec<_>>()
            .await;

        for months in months_stream {
            let months = months?;
            let idx = years.iter().position(|y| y.year == months[0].year);
            if let Some(idx) = idx {
                years[idx].months = months;
            }
        }

        Ok(years)
    }

    #[tracing::instrument(skip(self))]
    async fn get_year_data_by_number(&self, year: i32) -> DbResult<YearData> {
        sqlx::query_as!(
            YearData,
            r#"
            SELECT id as "id: Uuid", year as "year: i32", refreshed_at as "refreshed_at: DateTime<Utc>"
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
    async fn add(&self, year: &Year) -> DbResult<()> {
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

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn get(&self, year: i32) -> DbResult<Year> {
        let year_data = self.get_year_data_by_number(year).await?;

        let (net_totals, months) = try_join!(
            self.get_net_totals(year_data.id),
            self.month_repo.get_months_of_year(year),
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

        let year = Year {
            id: year_data.id,
            year: year_data.year,
            refreshed_at: year_data.refreshed_at,
            net_assets,
            net_portfolio,
            months,
        };

        Ok(year)
    }

    #[tracing::instrument(skip(self))]
    async fn get_net_totals(&self, year_id: Uuid) -> DbResult<Vec<NetTotal>> {
        sqlx::query_as!(
            NetTotal,
            r#"
            SELECT
                id as "id: Uuid",
                type AS "net_type: NetTotalType",
                total,
                percent_var as "percent_var: f32",
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
    async fn update_net_totals(&self, year: i32) -> DbResult<()> {
        update_year_net_totals(self, year).await
    }

    #[tracing::instrument(skip_all)]
    async fn update_refreshed_at(&self, year: &YearData) -> DbResult<()> {
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
    async fn delete(&self, year: i32) -> DbResult<()> {
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
async fn update_year_net_totals(year_repo: &SqliteYearRepo, year: i32) -> DbResult<()> {
    let year_data = year_repo.get_year_data_by_number(year).await?;

    let (net_totals, months) = try_join!(
        year_repo.get_net_totals(year_data.id),
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

    let mut year = Year {
        id: year_data.id,
        year: year_data.year,
        refreshed_at: year_data.refreshed_at,
        net_assets,
        net_portfolio,
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
