use std::sync::Arc;

use datamize_domain::{
    async_trait,
    db::{DbResult, SavingRateRepo, YearData},
    Incomes, SavingRate, Savings, Uuid,
};
use sqlx::{postgres::PgHasArrayType, PgPool};

#[derive(Debug, Clone)]
pub struct PostgresSavingRateRepo {
    pub db_conn_pool: PgPool,
}

impl PostgresSavingRateRepo {
    pub fn new_arced(db_conn_pool: PgPool) -> Arc<Self> {
        Arc::new(Self { db_conn_pool })
    }

    #[tracing::instrument(skip(self))]
    async fn get_year_data_by_number(&self, year: i32) -> DbResult<YearData> {
        sqlx::query_as!(
            YearData,
            r#"
            SELECT year_id AS "id: Uuid", year, refreshed_at
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
impl SavingRateRepo for PostgresSavingRateRepo {
    #[tracing::instrument(skip(self))]
    async fn get_from_year(&self, year: i32) -> DbResult<Vec<SavingRate>> {
        let db_rows = sqlx::query!(
            r#"
            SELECT
                sr.saving_rate_id as saving_rate_id,
                sr.name,
                sr.savings AS "savings!: IdsAndBalanceRecord",
                sr.employer_contribution,
                sr.employee_contribution,
                sr.mortgage_capital,
                sr.incomes AS "incomes!: IdsAndBalanceRecord"
            FROM balance_sheet_saving_rates AS sr
            JOIN balance_sheet_years AS y ON y.year_id = sr.year_id AND y.year = $1;
            "#,
            year,
        )
        .fetch_all(&self.db_conn_pool)
        .await?;

        Ok(db_rows
            .into_iter()
            .map(|r| SavingRate {
                id: r.saving_rate_id,
                name: r.name,
                savings: Savings {
                    category_ids: r.savings.ids,
                    extra_balance: r.savings.extra_balance,
                    total: 0,
                },
                employer_contribution: r.employer_contribution,
                employee_contribution: r.employee_contribution,
                mortgage_capital: r.mortgage_capital,
                incomes: Incomes {
                    payee_ids: r.incomes.ids,
                    extra_balance: r.incomes.extra_balance,
                    total: 0,
                },
                year,
            })
            .collect())
    }

    #[tracing::instrument(skip(self))]
    async fn get(&self, saving_rate_id: Uuid) -> DbResult<SavingRate> {
        let db_row = sqlx::query!(
            r#"
            SELECT
                sr.saving_rate_id AS "id",
                sr.name,
                sr.savings AS "savings!: IdsAndBalanceRecord",
                sr.employer_contribution,
                sr.employee_contribution,
                sr.mortgage_capital,
                sr.incomes AS "incomes!: IdsAndBalanceRecord",
                y.year
            FROM balance_sheet_saving_rates AS sr
            JOIN balance_sheet_years AS y ON y.year_id = sr.year_id
            WHERE sr.saving_rate_id = $1;
            "#,
            saving_rate_id,
        )
        .fetch_one(&self.db_conn_pool)
        .await?;

        Ok(SavingRate {
            id: db_row.id,
            name: db_row.name,
            savings: Savings {
                category_ids: db_row.savings.ids,
                extra_balance: db_row.savings.extra_balance,
                total: 0,
            },
            employer_contribution: db_row.employer_contribution,
            employee_contribution: db_row.employee_contribution,
            mortgage_capital: db_row.mortgage_capital,
            incomes: Incomes {
                payee_ids: db_row.incomes.ids,
                extra_balance: db_row.incomes.extra_balance,
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
                sr.saving_rate_id AS "id",
                sr.name,
                sr.savings AS "savings!: IdsAndBalanceRecord",
                sr.employer_contribution,
                sr.employee_contribution,
                sr.mortgage_capital,
                sr.incomes AS "incomes!: IdsAndBalanceRecord",
                y.year
            FROM balance_sheet_saving_rates AS sr
            JOIN balance_sheet_years AS y ON y.year_id = sr.year_id
            WHERE sr.name = $1;
            "#,
            name,
        )
        .fetch_one(&self.db_conn_pool)
        .await?;

        Ok(SavingRate {
            id: db_row.id,
            name: db_row.name,
            savings: Savings {
                category_ids: db_row.savings.ids,
                extra_balance: db_row.savings.extra_balance,
                total: 0,
            },
            employer_contribution: db_row.employer_contribution,
            employee_contribution: db_row.employee_contribution,
            mortgage_capital: db_row.mortgage_capital,
            incomes: Incomes {
                payee_ids: db_row.incomes.ids,
                extra_balance: db_row.incomes.extra_balance,
                total: 0,
            },
            year: db_row.year,
        })
    }

    #[tracing::instrument(skip_all)]
    async fn update(&self, saving_rate: &SavingRate) -> DbResult<()> {
        let year_data = self.get_year_data_by_number(saving_rate.year).await?;

        sqlx::query_unchecked!(
            r#"
            INSERT INTO balance_sheet_saving_rates (saving_rate_id, name, savings, employer_contribution, employee_contribution, mortgage_capital, incomes, year_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (saving_rate_id) DO UPDATE SET
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
            IdsAndBalanceRecord { ids: saving_rate.savings.category_ids.clone(), extra_balance: saving_rate.savings.extra_balance },
            saving_rate.employer_contribution,
            saving_rate.employee_contribution,
            saving_rate.mortgage_capital,
            IdsAndBalanceRecord { ids: saving_rate.incomes.payee_ids.clone(), extra_balance: saving_rate.incomes.extra_balance },
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
                WHERE saving_rate_id = $1
            "#,
            saving_rate_id,
        )
        .execute(&self.db_conn_pool)
        .await?;

        Ok(())
    }
}

#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "ids_and_balance")]
struct IdsAndBalanceRecord {
    ids: Vec<Uuid>,
    extra_balance: i64,
}

impl PgHasArrayType for IdsAndBalanceRecord {
    fn array_type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("_ids_and_balance")
    }
}
