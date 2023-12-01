use std::sync::Arc;

use chrono::{DateTime, Utc};
use datamize_domain::{
    async_trait,
    db::{DbResult, SavingRateRepo, YearData},
    Incomes, SavingRate, Savings, Uuid,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct SqliteSavingRateRepo {
    pub db_conn_pool: SqlitePool,
}

impl SqliteSavingRateRepo {
    pub fn new_arced(db_conn_pool: SqlitePool) -> Arc<Self> {
        Arc::new(Self { db_conn_pool })
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
}

#[async_trait]
impl SavingRateRepo for SqliteSavingRateRepo {
    #[tracing::instrument(skip(self))]
    async fn get_from_year(&self, year: i32) -> DbResult<Vec<SavingRate>> {
        let db_rows = sqlx::query!(
            r#"
            SELECT
                sr.id as "saving_rate_id: Uuid",
                sr.name,
                sr.savings,
                sr.employer_contribution,
                sr.employee_contribution,
                sr.mortgage_capital,
                sr.incomes
            FROM balance_sheet_saving_rates AS sr
            JOIN balance_sheet_years AS y ON y.id = sr.year_id AND y.year = $1;
            "#,
            year,
        )
        .fetch_all(&self.db_conn_pool)
        .await?;

        Ok(db_rows
            .into_iter()
            .map(|r| {
                let savings: IdsAndBalanceRecord = serde_json::from_str(&r.savings).unwrap();
                let incomes: IdsAndBalanceRecord = serde_json::from_str(&r.incomes).unwrap();
                SavingRate {
                    id: r.saving_rate_id,
                    name: r.name,
                    savings: Savings {
                        category_ids: savings.ids,
                        extra_balance: savings.extra_balance,
                        total: 0,
                    },
                    employer_contribution: r.employer_contribution,
                    employee_contribution: r.employee_contribution,
                    mortgage_capital: r.mortgage_capital,
                    incomes: Incomes {
                        payee_ids: incomes.ids,
                        extra_balance: incomes.extra_balance,
                        total: 0,
                    },
                    year,
                }
            })
            .collect())
    }

    #[tracing::instrument(skip(self))]
    async fn get(&self, saving_rate_id: Uuid) -> DbResult<SavingRate> {
        let db_row = sqlx::query!(
            r#"
            SELECT
                sr.id as "id: Uuid",
                sr.name,
                sr.savings,
                sr.employer_contribution,
                sr.employee_contribution,
                sr.mortgage_capital,
                sr.incomes,
                y.year as "year: i32"
            FROM balance_sheet_saving_rates AS sr
            JOIN balance_sheet_years AS y ON y.id = sr.year_id
            WHERE sr.id = $1;
            "#,
            saving_rate_id,
        )
        .fetch_one(&self.db_conn_pool)
        .await?;

        let savings: IdsAndBalanceRecord = serde_json::from_str(&db_row.savings).unwrap();
        let incomes: IdsAndBalanceRecord = serde_json::from_str(&db_row.incomes).unwrap();

        Ok(SavingRate {
            id: db_row.id,
            name: db_row.name,
            savings: Savings {
                category_ids: savings.ids,
                extra_balance: savings.extra_balance,
                total: 0,
            },
            employer_contribution: db_row.employer_contribution,
            employee_contribution: db_row.employee_contribution,
            mortgage_capital: db_row.mortgage_capital,
            incomes: Incomes {
                payee_ids: incomes.ids,
                extra_balance: incomes.extra_balance,
                total: 0,
            },
            year: db_row.year,
        })
    }

    #[tracing::instrument(skip(self))]
    async fn get_by_name(&self, name: &str) -> DbResult<SavingRate> {
        let db_row = sqlx::query!(
            r#"
            SELECT
                sr.id as "id: Uuid",
                sr.name,
                sr.savings,
                sr.employer_contribution,
                sr.employee_contribution,
                sr.mortgage_capital,
                sr.incomes,
                y.year as "year: i32"
            FROM balance_sheet_saving_rates AS sr
            JOIN balance_sheet_years AS y ON y.id = sr.year_id
            WHERE sr.name = $1;
            "#,
            name,
        )
        .fetch_one(&self.db_conn_pool)
        .await?;

        let savings: IdsAndBalanceRecord = serde_json::from_str(&db_row.savings).unwrap();
        let incomes: IdsAndBalanceRecord = serde_json::from_str(&db_row.incomes).unwrap();

        Ok(SavingRate {
            id: db_row.id,
            name: db_row.name,
            savings: Savings {
                category_ids: savings.ids,
                extra_balance: savings.extra_balance,
                total: 0,
            },
            employer_contribution: db_row.employer_contribution,
            employee_contribution: db_row.employee_contribution,
            mortgage_capital: db_row.mortgage_capital,
            incomes: Incomes {
                payee_ids: incomes.ids,
                extra_balance: incomes.extra_balance,
                total: 0,
            },
            year: db_row.year,
        })
    }

    #[tracing::instrument(skip_all)]
    async fn update(&self, saving_rate: &SavingRate) -> DbResult<()> {
        let year_data = self.get_year_data_by_number(saving_rate.year).await?;
        let savings = serde_json::to_string(&IdsAndBalanceRecord {
            ids: saving_rate.savings.category_ids.clone(),
            extra_balance: saving_rate.savings.extra_balance,
        })
        .unwrap();
        let incomes = serde_json::to_string(&IdsAndBalanceRecord {
            ids: saving_rate.incomes.payee_ids.clone(),
            extra_balance: saving_rate.incomes.extra_balance,
        })
        .unwrap();

        sqlx::query!(
            r#"
            INSERT INTO balance_sheet_saving_rates (id, name, savings, employer_contribution, employee_contribution, mortgage_capital, incomes, year_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (id) DO UPDATE SET
            name = EXCLUDED.name,
            savings = EXCLUDED.savings,
            employer_contribution = EXCLUDED.employer_contribution,
            employee_contribution = EXCLUDED.employee_contribution,
            mortgage_capital = EXCLUDED.mortgage_capital,
            incomes = EXCLUDED.incomes,
            year_id = EXCLUDED.year_id;
            "#,
            saving_rate.id,
            saving_rate.name,
            savings,
            saving_rate.employer_contribution,
            saving_rate.employee_contribution,
            saving_rate.mortgage_capital,
            incomes,
            year_data.id
        )
        .execute(&self.db_conn_pool)
        .await?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn delete(&self, saving_rate_id: Uuid) -> DbResult<()> {
        sqlx::query!(
            r#"
                DELETE FROM balance_sheet_saving_rates
                WHERE id = $1
            "#,
            saving_rate_id,
        )
        .execute(&self.db_conn_pool)
        .await?;

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
struct IdsAndBalanceRecord {
    ids: Vec<Uuid>,
    extra_balance: i64,
}

pub async fn sabotage_saving_rates_table(pool: &SqlitePool) -> DbResult<()> {
    sqlx::query!("ALTER TABLE balance_sheet_saving_rates DROP COLUMN name;",)
        .execute(pool)
        .await?;

    Ok(())
}
