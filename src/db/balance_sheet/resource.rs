use std::collections::{BTreeMap, HashMap};

use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{
    BaseFinancialResource, FinancialResourceMonthly, FinancialResourceYearly, MonthNum,
};

#[tracing::instrument(skip(db_conn_pool))]
pub async fn get_all_financial_resources_of_all_years(
    db_conn_pool: &PgPool,
) -> Result<Vec<FinancialResourceYearly>, sqlx::Error> {
    let mut resources: HashMap<Uuid, FinancialResourceYearly> = HashMap::new();

    let db_rows = sqlx::query!(
        r#"
        SELECT
            r.*,
            rm.balance,
            m.month,
            y.year
        FROM balance_sheet_resources AS r
        JOIN balance_sheet_resources_months AS rm ON r.id = rm.resource_id
        JOIN balance_sheet_months AS m ON rm.month_id = m.id
        JOIN balance_sheet_years AS y ON y.id = m.year_id
        "#
    )
    .fetch_all(db_conn_pool)
    .await?;

    for r in db_rows {
        resources
            .entry(r.id)
            .and_modify(|res| {
                res.balance_per_month
                    .insert(r.month.try_into().unwrap(), r.balance);
            })
            .or_insert_with(|| {
                let mut balance_per_month: BTreeMap<MonthNum, i64> = BTreeMap::new();

                // TODO: Can month already be there? And what will happen? To test...
                balance_per_month.insert(r.month.try_into().unwrap(), r.balance);

                FinancialResourceYearly {
                    base: BaseFinancialResource {
                        id: r.id,
                        name: r.name,
                        category: r.category.parse().unwrap(),
                        r_type: r.r#type.parse().unwrap(),
                        editable: r.editable,
                    },
                    year: r.year,
                    balance_per_month,
                }
            });
    }

    let mut resources: Vec<FinancialResourceYearly> = resources.into_values().collect();

    resources.sort_by(|a, b| a.year.cmp(&b.year));

    Ok(resources)
}

#[tracing::instrument(skip(db_conn_pool))]
pub async fn get_financial_resources_of_year(
    db_conn_pool: &PgPool,
    year: i32,
) -> Result<Vec<FinancialResourceYearly>, sqlx::Error> {
    let mut resources: BTreeMap<Uuid, FinancialResourceYearly> = BTreeMap::new();

    let db_rows = sqlx::query!(
        r#"
        SELECT
            r.*,
            rm.balance,
            m.month
        FROM balance_sheet_resources AS r
        JOIN balance_sheet_resources_months AS rm ON r.id = rm.resource_id
        JOIN balance_sheet_months AS m ON rm.month_id = m.id
        JOIN balance_sheet_years AS y ON y.id = m.year_id AND y.year = $1;
        "#,
        year,
    )
    .fetch_all(db_conn_pool)
    .await?;

    for r in db_rows {
        resources
            .entry(r.id)
            .and_modify(|res| {
                res.balance_per_month
                    .insert(r.month.try_into().unwrap(), r.balance);
            })
            .or_insert_with(|| {
                let mut balance_per_month: BTreeMap<MonthNum, i64> = BTreeMap::new();

                // TODO: Can month already be there? And what will happen? To test...
                balance_per_month.insert(r.month.try_into().unwrap(), r.balance);

                FinancialResourceYearly {
                    base: BaseFinancialResource {
                        id: r.id,
                        name: r.name,
                        category: r.category.parse().unwrap(),
                        r_type: r.r#type.parse().unwrap(),
                        editable: r.editable,
                    },
                    year,
                    balance_per_month,
                }
            });
    }

    Ok(resources.into_values().collect())
}

#[tracing::instrument(skip(db_conn_pool))]
pub async fn get_financial_resources_of_month(
    db_conn_pool: &PgPool,
    month: MonthNum,
    year: i32,
) -> Result<Vec<FinancialResourceMonthly>, sqlx::Error> {
    let mut resources: Vec<FinancialResourceMonthly> = vec![];

    let db_rows = sqlx::query!(
        r#"
            SELECT
                r.*,
                rm.balance,
                m.month,
                y.year
            FROM balance_sheet_resources AS r
            JOIN balance_sheet_resources_months AS rm ON r.id = rm.resource_id
            JOIN balance_sheet_months AS m ON rm.month_id = m.id AND m.month = $1
            JOIN balance_sheet_years AS y ON y.id = m.year_id AND y.year = $2;
            "#,
        month as i16,
        year,
    )
    .fetch_all(db_conn_pool)
    .await?;

    for r in db_rows {
        resources.push(FinancialResourceMonthly {
            base: BaseFinancialResource {
                id: r.id,
                name: r.name,
                category: r.category.parse().unwrap(),
                r_type: r.r#type.parse().unwrap(),
                editable: r.editable,
            },
            month: r.month.try_into().unwrap(),
            year: r.year,
            balance: r.balance,
        });
    }

    Ok(resources)
}

#[tracing::instrument(skip(db_conn_pool))]
pub async fn get_financial_resource(
    db_conn_pool: &PgPool,
    year: i32,
    resource_id: Uuid,
) -> Result<FinancialResourceYearly, sqlx::Error> {
    let db_rows = sqlx::query!(
        r#"
        SELECT
            r.*,
            rm.balance,
            m.month
        FROM balance_sheet_resources AS r
        JOIN balance_sheet_resources_months AS rm ON r.id = rm.resource_id AND r.id = $1
        JOIN balance_sheet_months AS m ON rm.month_id = m.id
        JOIN balance_sheet_years AS y ON y.id = m.year_id AND y.year = $2;
        "#,
        resource_id,
        year,
    )
    .fetch_all(db_conn_pool)
    .await?;

    let mut resource: Option<FinancialResourceYearly> = None;

    for r in db_rows {
        match resource {
            Some(ref mut res) => {
                res.balance_per_month
                    .insert(r.month.try_into().unwrap(), r.balance);
            }
            None => {
                let mut balance_per_month: BTreeMap<MonthNum, i64> = BTreeMap::new();

                // TODO: Can month already be there? And what will happen? To test...
                balance_per_month.insert(r.month.try_into().unwrap(), r.balance);

                resource = Some(FinancialResourceYearly {
                    base: BaseFinancialResource {
                        id: r.id,
                        name: r.name,
                        category: r.category.parse().unwrap(),
                        r_type: r.r#type.parse().unwrap(),
                        editable: r.editable,
                    },
                    year,
                    balance_per_month,
                })
            }
        }
    }

    resource.ok_or(sqlx::Error::RowNotFound)
}

#[tracing::instrument(skip_all)]
pub async fn update_financial_resource(
    db_conn_pool: &PgPool,
    resource: &FinancialResourceYearly,
) -> Result<(), sqlx::Error> {
    // First update the resource itself
    sqlx::query!(
        r#"
        INSERT INTO balance_sheet_resources (id, name, category, type, editable)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (id) DO UPDATE
        SET name = EXCLUDED.name,
        category = EXCLUDED.category,
        type = EXCLUDED.type,
        editable = EXCLUDED.editable;
        "#,
        resource.base.id,
        resource.base.name,
        resource.base.category.to_string(),
        resource.base.r_type.to_string(),
        resource.base.editable,
    )
    .execute(db_conn_pool)
    .await?;

    // Then the balance per month
    for (month, balance) in &resource.balance_per_month {
        // TODO: To test, what happens when month already there but not resource? Technically it should update...
        // Or if resource already there but not month...
        sqlx::query!(
            r#"
            INSERT INTO balance_sheet_resources_months (resource_id, month_id, balance)
            SELECT r.id, m.id, balance
            FROM (
            VALUES
                ($1::bigint)
            ) x (balance)
			JOIN balance_sheet_resources AS r ON r.id = $2
            JOIN balance_sheet_years AS y ON y.year = $3
			JOIN balance_sheet_months AS m ON m.month = $4 AND m.year_id = y.id
			ON CONFLICT (resource_id, month_id) DO UPDATE SET
            balance = EXCLUDED.balance;
            "#,
            balance,
            resource.base.id,
            resource.year,
            *month as i16,
        )
        .execute(db_conn_pool)
        .await?;
    }

    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn update_monthly_financial_resource(
    db_conn_pool: &PgPool,
    resource: &FinancialResourceMonthly,
) -> Result<(), sqlx::Error> {
    // First update the resource itself
    sqlx::query!(
        r#"
        INSERT INTO balance_sheet_resources (id, name, category, type, editable)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (id) DO UPDATE SET
        name = EXCLUDED.name,
        category = EXCLUDED.category,
        type = EXCLUDED.type,
        editable = EXCLUDED.editable;
        "#,
        resource.base.id,
        resource.base.name,
        resource.base.category.to_string(),
        resource.base.r_type.to_string(),
        resource.base.editable,
    )
    .execute(db_conn_pool)
    .await?;

    // Then the balance of the month
    // TODO: To test, what happens when month already there but not resource? Technically it should update...
    // Or if resource already there but not month...
    sqlx::query!(
        r#"
        INSERT INTO balance_sheet_resources_months (resource_id, month_id, balance)
        SELECT r.id, m.id, balance
        FROM (
        VALUES
            ($1::bigint)
        ) x (balance)
        JOIN balance_sheet_resources AS r ON r.id = $2
        JOIN balance_sheet_years AS y ON y.year = $3
        JOIN balance_sheet_months AS m ON m.month = $4 AND m.year_id = y.id
        ON CONFLICT (resource_id, month_id) DO UPDATE SET
        balance = EXCLUDED.balance;
        "#,
        resource.balance,
        resource.base.id,
        resource.year,
        resource.month as i16,
    )
    .execute(db_conn_pool)
    .await?;

    Ok(())
}

#[tracing::instrument(skip(db_conn_pool))]
pub async fn delete_financial_resource(
    db_conn_pool: &PgPool,
    resource_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
            DELETE FROM balance_sheet_resources
            WHERE id = $1
        "#,
        resource_id,
    )
    .execute(db_conn_pool)
    .await?;

    Ok(())
}
