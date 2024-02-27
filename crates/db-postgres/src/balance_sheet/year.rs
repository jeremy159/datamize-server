use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use datamize_domain::db::{DbResult, MonthRepo, NetTotalType, YearData, YearRepo};
use datamize_domain::{async_trait, NetTotal, NetTotals, Uuid, Year};
use futures::try_join;
use itertools::Itertools;
use sqlx::PgPool;

use super::{PostgresFinResRepo, PostgresMonthRepo};

#[derive(Debug, Clone)]
pub struct PostgresYearRepo {
    pub db_conn_pool: PgPool,
    pub month_repo: PostgresMonthRepo,
}

impl PostgresYearRepo {
    pub fn new_arced(db_conn_pool: PgPool) -> Arc<Self> {
        Arc::new(Self {
            db_conn_pool: db_conn_pool.clone(),
            month_repo: PostgresMonthRepo {
                db_conn_pool: db_conn_pool.clone(),
                fin_res_repo: PostgresFinResRepo { db_conn_pool },
            },
        })
    }
}

// TODO: Create index for month and year access to increase performance.
// And update all schema to improve manipulation (e.g. UNIQUE on cols appropriate,
// no col with id as name but for example resource_id instead).
#[async_trait]
impl YearRepo for PostgresYearRepo {
    #[tracing::instrument(skip(self))]
    async fn get_years(&self) -> DbResult<Vec<Year>> {
        let year_datas = sqlx::query_as!(
            YearData,
            r#"
            SELECT
                id as "id: Uuid",
                year as "year: i32",
                refreshed_at as "refreshed_at: DateTime<Utc>"
            FROM balance_sheet_years
            ORDER BY year;
            "#
        )
        .fetch_all(&self.db_conn_pool)
        .await?;

        let mut years: Vec<Year> = vec![];

        for yd in year_datas {
            let net_totals = self.get_net_totals(yd.id).await?;
            let months = self.month_repo.get_months_of_year(yd.year).await?;

            years.push(Year {
                id: yd.id,
                year: yd.year,
                refreshed_at: yd.refreshed_at,
                net_totals,
                months,
            });
        }

        Ok(years)
    }

    #[tracing::instrument(skip(self))]
    async fn get_years_starting_from(&self, year: i32) -> DbResult<Vec<Year>> {
        let year_datas = sqlx::query_as!(
            YearData,
            r#"
            SELECT
                id as "id: Uuid",
                year as "year: i32",
                refreshed_at as "refreshed_at: DateTime<Utc>"
            FROM balance_sheet_years
            WHERE year >= $1
            ORDER BY year;
            "#,
            year,
        )
        .fetch_all(&self.db_conn_pool)
        .await?;

        let mut years: Vec<Year> = vec![];

        for yd in year_datas {
            let net_totals = self.get_net_totals(yd.id).await?;
            let months = self
                .month_repo
                .get_months_of_year_without_resources(year)
                .await?;

            years.push(Year {
                id: yd.id,
                year: yd.year,
                refreshed_at: yd.refreshed_at,
                net_totals,
                months,
            });
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

        self.insert_net_totals(year.id, &year.net_totals).await?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn get_without_resources(&self, year: i32) -> DbResult<Year> {
        let year_data = self.get_year_data_by_number(year).await?;

        let (net_totals, months) = try_join!(
            self.get_net_totals(year_data.id),
            self.month_repo.get_months_of_year_without_resources(year),
        )?;

        let year = Year {
            id: year_data.id,
            year: year_data.year,
            refreshed_at: year_data.refreshed_at,
            net_totals,
            months,
        };

        Ok(year)
    }

    #[tracing::instrument(skip(self))]
    async fn get(&self, year: i32) -> DbResult<Year> {
        let year_data = self.get_year_data_by_number(year).await?;

        let (net_totals, months) = try_join!(
            self.get_net_totals(year_data.id),
            self.month_repo.get_months_of_year(year),
        )?;

        let year = Year {
            id: year_data.id,
            year: year_data.year,
            refreshed_at: year_data.refreshed_at,
            net_totals,
            months,
        };

        Ok(year)
    }

    #[tracing::instrument(skip(self))]
    async fn get_net_totals(&self, year_id: Uuid) -> DbResult<NetTotals> {
        let rows = sqlx::query!(
            r#"
            SELECT
                id AS "id: Uuid",
                type AS "net_type: NetTotalType",
                total,
                percent_var as "percent_var: f32",
                balance_var,
                last_updated as "last_updated?: DateTime<Utc>"
            FROM balance_sheet_net_totals_years
            WHERE year_id = $1;
            "#,
            year_id,
        )
        .fetch_all(&self.db_conn_pool)
        .await?;

        let mut net_totals = NetTotals::default();

        for r in rows {
            match r.net_type {
                NetTotalType::Asset => {
                    net_totals.assets = NetTotal {
                        id: r.id,
                        total: r.total,
                        percent_var: r.percent_var,
                        balance_var: r.balance_var,
                        last_updated: r.last_updated,
                    };
                }
                NetTotalType::Portfolio => {
                    net_totals.portfolio = NetTotal {
                        id: r.id,
                        total: r.total,
                        percent_var: r.percent_var,
                        balance_var: r.balance_var,
                        last_updated: r.last_updated,
                    };
                }
            };
        }

        Ok(net_totals)
    }

    #[tracing::instrument(skip(self))]
    async fn update_net_totals(&self, year: i32) -> DbResult<()> {
        let mut years = self.get_years_starting_from(year).await?;
        if let Some(first_year) = years.first_mut() {
            first_year.update_net_totals();

            if let Ok(prev_year) = self.get_without_resources(year - 1).await {
                first_year.compute_variation(&prev_year);
            }
        }

        for (prev_year, curr_year) in years
            .iter_mut()
            .map(RefCell::new)
            .map(Rc::new)
            .tuple_windows()
        {
            let mut curr_year = curr_year.borrow_mut();
            curr_year.update_net_totals();
            curr_year.compute_variation(&prev_year.borrow());
        }

        for year in years {
            self.insert_net_totals(year.id, &year.net_totals).await?;
        }

        Ok(())
    }

    #[tracing::instrument(skip(self, net_totals))]
    async fn insert_net_totals(&self, year_id: Uuid, net_totals: &NetTotals) -> DbResult<()> {
        let mut transaction = self.db_conn_pool.begin().await?;

        let net_type = NetTotalType::Asset.to_string();
        sqlx::query!(
            r#"
            INSERT INTO balance_sheet_net_totals_years (id, type, total, percent_var, balance_var, last_updated, year_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (id) DO UPDATE
            SET type = EXCLUDED.type,
            total = EXCLUDED.total,
            percent_var = EXCLUDED.percent_var,
            balance_var = EXCLUDED.balance_var,
            last_updated = EXCLUDED.last_updated;
            "#,
            net_totals.assets.id,
            net_type,
            net_totals.assets.total,
            net_totals.assets.percent_var,
            net_totals.assets.balance_var,
            net_totals.assets.last_updated,
            year_id,
        )
        .execute(&mut *transaction)
        .await?;

        let net_type = NetTotalType::Portfolio.to_string();
        sqlx::query!(
            r#"
            INSERT INTO balance_sheet_net_totals_years (id, type, total, percent_var, balance_var, last_updated, year_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (id) DO UPDATE
            SET type = EXCLUDED.type,
            total = EXCLUDED.total,
            percent_var = EXCLUDED.percent_var,
            balance_var = EXCLUDED.balance_var,
            last_updated = EXCLUDED.last_updated;
            "#,
            net_totals.portfolio.id,
            net_type,
            net_totals.portfolio.total,
            net_totals.portfolio.percent_var,
            net_totals.portfolio.balance_var,
            net_totals.portfolio.last_updated,
            year_id,
        )
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;

        Ok(())
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
