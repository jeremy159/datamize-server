use std::{
    cmp::Ordering,
    collections::{BTreeMap, HashMap, HashSet},
    sync::Arc,
};

use datamize_domain::{
    async_trait,
    db::{DbError, DbResult, FinResRepo},
    BaseFinancialResource, FinancialResourceMonthly, FinancialResourceYearly, MonthNum,
    ResourceCategory, ResourceType, Uuid,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct SqliteFinResRepo {
    pub db_conn_pool: SqlitePool,
}

impl SqliteFinResRepo {
    pub fn new_arced(db_conn_pool: SqlitePool) -> Arc<Self> {
        Arc::new(Self { db_conn_pool })
    }
}

#[async_trait]
impl FinResRepo for SqliteFinResRepo {
    #[tracing::instrument(skip(self))]
    async fn get_from_all_years(&self) -> DbResult<Vec<FinancialResourceYearly>> {
        let mut resources: HashMap<Uuid, FinancialResourceYearly> = HashMap::new();

        let db_rows = sqlx::query!(
            r#"
            SELECT
                r.id AS "id: Uuid",
                r.name,
                r.category AS "category: ResourceCategory",
                r.type AS "type: ResourceType",
                r.editable,
                r.ynab_account_ids,
                r.external_account_ids,
                rm.balance,
                m.month AS "month: MonthNum",
                y.year AS "year: i32"
            FROM balance_sheet_resources AS r
            JOIN balance_sheet_resources_months AS rm ON r.id = rm.resource_id
            JOIN balance_sheet_months AS m ON rm.month_id = m.id
            JOIN balance_sheet_years AS y ON y.id = m.year_id
            "#
        )
        .fetch_all(&self.db_conn_pool)
        .await?;

        for r in db_rows {
            resources
                .entry(r.id)
                .and_modify(|res| {
                    res.balance_per_month.insert(r.month, r.balance);
                })
                .or_insert_with(|| {
                    let mut balance_per_month: BTreeMap<MonthNum, i64> = BTreeMap::new();

                    // Relations in the DB enforces that only one month in a year exists for one resource
                    balance_per_month.insert(r.month, r.balance);

                    let ynab_account_ids_rec: IdsRecord = r
                        .ynab_account_ids
                        .as_ref()
                        .map(|r| serde_json::from_str(r).unwrap())
                        .unwrap_or(IdsRecord { ids: None });
                    let external_account_ids_rec: IdsRecord = r
                        .external_account_ids
                        .as_ref()
                        .map(|r| serde_json::from_str(r).unwrap())
                        .unwrap_or(IdsRecord { ids: None });

                    FinancialResourceYearly {
                        base: BaseFinancialResource {
                            id: r.id,
                            name: r.name,
                            category: r.category,
                            r_type: r.r#type,
                            editable: r.editable,
                            ynab_account_ids: ynab_account_ids_rec.ids,
                            external_account_ids: external_account_ids_rec.ids,
                        },
                        year: r.year,
                        balance_per_month,
                    }
                });
        }

        let mut resources: Vec<FinancialResourceYearly> = resources.into_values().collect();

        resources.sort_by(|a, b| match a.year.cmp(&b.year) {
            Ordering::Equal => a.base.name.cmp(&b.base.name),
            other => other,
        });

        Ok(resources)
    }

    #[tracing::instrument(skip(self))]
    async fn get_from_year(&self, year: i32) -> DbResult<Vec<FinancialResourceYearly>> {
        let mut resources: BTreeMap<Uuid, FinancialResourceYearly> = BTreeMap::new();

        let db_rows = sqlx::query!(
            r#"
            SELECT
                r.id AS "id: Uuid",
                r.name,
                r.category AS "category: ResourceCategory",
                r.type AS "type: ResourceType",
                r.editable,
                r.ynab_account_ids,
                r.external_account_ids,
                rm.balance,
                m.month AS "month: MonthNum",
                y.year AS "year: i32"
            FROM balance_sheet_resources AS r
            JOIN balance_sheet_resources_months AS rm ON r.id = rm.resource_id
            JOIN balance_sheet_months AS m ON rm.month_id = m.id
            JOIN balance_sheet_years AS y ON y.id = m.year_id AND y.year = $1;
            "#,
            year,
        )
        .fetch_all(&self.db_conn_pool)
        .await?;

        for r in db_rows {
            resources
                .entry(r.id)
                .and_modify(|res| {
                    res.balance_per_month.insert(r.month, r.balance);
                })
                .or_insert_with(|| {
                    let mut balance_per_month: BTreeMap<MonthNum, i64> = BTreeMap::new();

                    // Relations in the DB enforces that only one month in a year exists for one resource
                    balance_per_month.insert(r.month, r.balance);

                    let ynab_account_ids_rec: IdsRecord = r
                        .ynab_account_ids
                        .as_ref()
                        .map(|r| serde_json::from_str(r).unwrap())
                        .unwrap_or(IdsRecord { ids: None });
                    let external_account_ids_rec: IdsRecord = r
                        .external_account_ids
                        .as_ref()
                        .map(|r| serde_json::from_str(r).unwrap())
                        .unwrap_or(IdsRecord { ids: None });

                    FinancialResourceYearly {
                        base: BaseFinancialResource {
                            id: r.id,
                            name: r.name,
                            category: r.category,
                            r_type: r.r#type,
                            editable: r.editable,
                            ynab_account_ids: ynab_account_ids_rec.ids,
                            external_account_ids: external_account_ids_rec.ids,
                        },
                        year,
                        balance_per_month,
                    }
                });
        }

        let mut resources: Vec<FinancialResourceYearly> = resources.into_values().collect();

        resources.sort_by(|a, b| a.base.name.cmp(&b.base.name));

        Ok(resources)
    }

    #[tracing::instrument(skip(self))]
    async fn get_from_month(
        &self,
        month: MonthNum,
        year: i32,
    ) -> DbResult<Vec<FinancialResourceMonthly>> {
        let mut resources: HashSet<FinancialResourceMonthly> = HashSet::new();

        let db_rows = sqlx::query!(
            r#"
                SELECT
                    r.id AS "id: Uuid",
                    r.name,
                    r.category AS "category: ResourceCategory",
                    r.type AS "type: ResourceType",
                    r.editable,
                    r.ynab_account_ids,
                    r.external_account_ids,
                    rm.balance,
                    m.month AS "month: MonthNum",
                    y.year AS "year: i32"
                FROM balance_sheet_resources AS r
                JOIN balance_sheet_resources_months AS rm ON r.id = rm.resource_id
                JOIN balance_sheet_months AS m ON rm.month_id = m.id AND m.month = $1
                JOIN balance_sheet_years AS y ON y.id = m.year_id AND y.year = $2;
                "#,
            month,
            year,
        )
        .fetch_all(&self.db_conn_pool)
        .await?;

        for r in db_rows {
            let ynab_account_ids_rec: IdsRecord = r
                .ynab_account_ids
                .as_ref()
                .map(|r| serde_json::from_str(r).unwrap())
                .unwrap_or(IdsRecord { ids: None });
            let external_account_ids_rec: IdsRecord = r
                .external_account_ids
                .as_ref()
                .map(|r| serde_json::from_str(r).unwrap())
                .unwrap_or(IdsRecord { ids: None });
            resources.insert(FinancialResourceMonthly {
                base: BaseFinancialResource {
                    id: r.id,
                    name: r.name,
                    category: r.category,
                    r_type: r.r#type,
                    editable: r.editable,
                    ynab_account_ids: ynab_account_ids_rec.ids,
                    external_account_ids: external_account_ids_rec.ids,
                },
                month: r.month,
                year: r.year,
                balance: r.balance,
            });
        }

        let mut resources: Vec<FinancialResourceMonthly> = resources.into_iter().collect();
        resources.sort_by(|a, b| a.base.name.cmp(&b.base.name));

        Ok(resources)
    }

    #[tracing::instrument(skip(self))]
    async fn get(&self, resource_id: Uuid) -> DbResult<FinancialResourceYearly> {
        let db_rows = sqlx::query!(
            r#"
            SELECT
                r.id AS "id: Uuid",
                r.name,
                r.category AS "category: ResourceCategory",
                r.type AS "type: ResourceType",
                r.editable,
                r.ynab_account_ids,
                r.external_account_ids,
                rm.balance,
                m.month AS "month: MonthNum",
                y.year AS "year: i32"
            FROM balance_sheet_resources AS r
            JOIN balance_sheet_resources_months AS rm ON r.id = rm.resource_id AND r.id = $1
            JOIN balance_sheet_months AS m ON rm.month_id = m.id
            JOIN balance_sheet_years AS y ON y.id = m.year_id;
            "#,
            resource_id,
        )
        .fetch_all(&self.db_conn_pool)
        .await?;

        let mut resource: Option<FinancialResourceYearly> = None;

        for r in db_rows {
            match resource {
                Some(ref mut res) => {
                    res.balance_per_month.insert(r.month, r.balance);
                }
                None => {
                    let mut balance_per_month: BTreeMap<MonthNum, i64> = BTreeMap::new();

                    // Relations in the DB enforces that only one month in a year exists for one resource
                    balance_per_month.insert(r.month, r.balance);
                    let ynab_account_ids_rec: IdsRecord = r
                        .ynab_account_ids
                        .as_ref()
                        .map(|r| serde_json::from_str(r).unwrap())
                        .unwrap_or(IdsRecord { ids: None });
                    let external_account_ids_rec: IdsRecord = r
                        .external_account_ids
                        .as_ref()
                        .map(|r| serde_json::from_str(r).unwrap())
                        .unwrap_or(IdsRecord { ids: None });

                    resource = Some(FinancialResourceYearly {
                        base: BaseFinancialResource {
                            id: r.id,
                            name: r.name,
                            category: r.category,
                            r_type: r.r#type,
                            editable: r.editable,
                            ynab_account_ids: ynab_account_ids_rec.ids,
                            external_account_ids: external_account_ids_rec.ids,
                        },
                        year: r.year,
                        balance_per_month,
                    })
                }
            }
        }

        resource.ok_or(DbError::NotFound)
    }

    #[tracing::instrument(skip(self))]
    async fn get_by_name(&self, name: &str) -> DbResult<Vec<FinancialResourceYearly>> {
        let db_rows = sqlx::query!(
            r#"
            SELECT
                r.id AS "id: Uuid",
                r.name,
                r.category AS "category: ResourceCategory",
                r.type AS "type: ResourceType",
                r.editable,
                r.ynab_account_ids,
                r.external_account_ids,
                rm.balance,
                m.month AS "month: MonthNum",
                y.year AS "year: i32"
            FROM balance_sheet_resources AS r
            JOIN balance_sheet_resources_months AS rm ON r.id = rm.resource_id AND r.name = $1
            JOIN balance_sheet_months AS m ON rm.month_id = m.id
            JOIN balance_sheet_years AS y ON y.id = m.year_id;
            "#,
            name,
        )
        .fetch_all(&self.db_conn_pool)
        .await?;

        let mut resources: BTreeMap<i32, FinancialResourceYearly> = BTreeMap::new();

        for r in db_rows {
            resources
                .entry(r.year)
                .and_modify(|res| {
                    res.balance_per_month.insert(r.month, r.balance);
                })
                .or_insert_with(|| {
                    let mut balance_per_month: BTreeMap<MonthNum, i64> = BTreeMap::new();

                    // Relations in the DB enforces that only one month in a year exists for one resource
                    balance_per_month.insert(r.month, r.balance);

                    let ynab_account_ids_rec: IdsRecord = r
                        .ynab_account_ids
                        .as_ref()
                        .map(|r| serde_json::from_str(r).unwrap())
                        .unwrap_or(IdsRecord { ids: None });
                    let external_account_ids_rec: IdsRecord = r
                        .external_account_ids
                        .as_ref()
                        .map(|r| serde_json::from_str(r).unwrap())
                        .unwrap_or(IdsRecord { ids: None });

                    FinancialResourceYearly {
                        base: BaseFinancialResource {
                            id: r.id,
                            name: r.name,
                            category: r.category,
                            r_type: r.r#type,
                            editable: r.editable,
                            ynab_account_ids: ynab_account_ids_rec.ids,
                            external_account_ids: external_account_ids_rec.ids,
                        },
                        year: r.year,
                        balance_per_month,
                    }
                });
        }

        Ok(resources.into_values().collect())
    }

    #[tracing::instrument(skip_all)]
    async fn update(&self, resource: &FinancialResourceYearly) -> DbResult<()> {
        let ynab_account_ids = serde_json::to_string(&IdsRecord {
            ids: resource.base.ynab_account_ids.clone(),
        })
        .unwrap();
        let external_account_ids = serde_json::to_string(&IdsRecord {
            ids: resource.base.external_account_ids.clone(),
        })
        .unwrap();
        // First update the resource itself
        sqlx::query!(
            r#"
            INSERT INTO balance_sheet_resources (id, name, category, type, editable, ynab_account_ids, external_account_ids)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (id) DO UPDATE
            SET name = EXCLUDED.name,
            category = EXCLUDED.category,
            type = EXCLUDED.type,
            editable = EXCLUDED.editable,
            ynab_account_ids = EXCLUDED.ynab_account_ids,
            external_account_ids = EXCLUDED.external_account_ids;
            "#,
            resource.base.id,
            resource.base.name,
            resource.base.category,
            resource.base.r_type,
            resource.base.editable,
            ynab_account_ids,
            external_account_ids,
        )
        .execute(&self.db_conn_pool)
        .await?;

        // Then the balance per month
        for (month, balance) in &resource.balance_per_month {
            sqlx::query!(
                r#"
                INSERT INTO balance_sheet_resources_months (resource_id, month_id, balance)
                SELECT r.id, m.id, $1
                FROM balance_sheet_resources AS r
                JOIN balance_sheet_years AS y ON y.year = $3
                JOIN balance_sheet_months AS m ON m.month = $4 AND m.year_id = y.id
                WHERE r.id = $2
                ON CONFLICT (resource_id, month_id) DO UPDATE SET
                balance = EXCLUDED.balance;
                "#,
                balance,
                resource.base.id,
                resource.year,
                *month,
            )
            .execute(&self.db_conn_pool)
            .await?;
        }

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn update_monthly(&self, resource: &FinancialResourceMonthly) -> DbResult<()> {
        let ynab_account_ids = serde_json::to_string(&IdsRecord {
            ids: resource.base.ynab_account_ids.clone(),
        })
        .unwrap();
        let external_account_ids = serde_json::to_string(&IdsRecord {
            ids: resource.base.external_account_ids.clone(),
        })
        .unwrap();

        // First update the resource itself
        sqlx::query!(
            r#"
            INSERT INTO balance_sheet_resources (id, name, category, type, editable, ynab_account_ids, external_account_ids)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (id) DO UPDATE SET
            name = EXCLUDED.name,
            category = EXCLUDED.category,
            type = EXCLUDED.type,
            editable = EXCLUDED.editable,
            ynab_account_ids = EXCLUDED.ynab_account_ids,
            external_account_ids = EXCLUDED.external_account_ids;
            "#,
            resource.base.id,
            resource.base.name,
            resource.base.category,
            resource.base.r_type,
            resource.base.editable,
            ynab_account_ids,
            external_account_ids,
        )
        .execute(&self.db_conn_pool)
        .await?;

        #[derive(Debug)]
        struct MonthData {
            id: Uuid,
        }

        let month_data = sqlx::query_as!(
            MonthData,
            r#"
            SELECT
                m.id AS "id: Uuid"
            FROM balance_sheet_months AS m
            JOIN balance_sheet_years AS y ON y.id = m.year_id AND y.year = $1
            WHERE m.month = $2;
            "#,
            resource.year,
            resource.month,
        )
        .fetch_one(&self.db_conn_pool)
        .await?;

        // Then the balance of the month
        sqlx::query!(
            r#"
            INSERT INTO balance_sheet_resources_months (resource_id, month_id, balance)
            VALUES ($1, $2, $3)
            ON CONFLICT (resource_id, month_id) DO UPDATE SET
            balance = EXCLUDED.balance;
            "#,
            resource.base.id,
            month_data.id,
            resource.balance,
        )
        .execute(&self.db_conn_pool)
        .await?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn delete(&self, resource_id: Uuid) -> DbResult<()> {
        sqlx::query!(
            r#"
                DELETE FROM balance_sheet_resources
                WHERE id = $1
            "#,
            resource_id,
        )
        .execute(&self.db_conn_pool)
        .await?;

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct IdsRecord {
    ids: Option<Vec<Uuid>>,
}

pub async fn sabotage_resources_table(pool: &SqlitePool) -> DbResult<()> {
    sqlx::query!("ALTER TABLE balance_sheet_resources DROP COLUMN name;",)
        .execute(pool)
        .await?;

    Ok(())
}
