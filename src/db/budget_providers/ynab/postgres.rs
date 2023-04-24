use sqlx::PgPool;
use ynab::types::{Account, AccountType, Category, GoalType, ScheduledTransactionDetail};

pub async fn save_categories(
    db_conn_pool: &PgPool,
    categories: &[Category],
) -> Result<(), sqlx::Error> {
    for c in categories {
        sqlx::query!(
                r#"
                INSERT INTO categories (id, category_group_id, name, hidden, original_category_group_id, note, budgeted, activity, balance, goal_type, goal_creation_month, goal_target, goal_target_month, goal_percentage_complete, goal_months_to_budget, goal_under_funded, goal_overall_funded, goal_overall_left, deleted)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
                ON CONFLICT (id) DO UPDATE
                SET category_group_id = EXCLUDED.category_group_id,
                name = EXCLUDED.name,
                hidden = EXCLUDED.hidden,
                original_category_group_id = EXCLUDED.original_category_group_id,
                note = EXCLUDED.note,
                budgeted = EXCLUDED.budgeted,
                activity = EXCLUDED.activity,
                balance = EXCLUDED.balance,
                goal_type = EXCLUDED.goal_type,
                goal_creation_month = EXCLUDED.goal_creation_month,
                goal_target = EXCLUDED.goal_target,
                goal_target_month = EXCLUDED.goal_target_month,
                goal_percentage_complete = EXCLUDED.goal_percentage_complete,
                goal_months_to_budget = EXCLUDED.goal_months_to_budget,
                goal_under_funded = EXCLUDED.goal_under_funded,
                goal_overall_funded = EXCLUDED.goal_overall_funded,
                goal_overall_left = EXCLUDED.goal_overall_left,
                deleted = EXCLUDED.deleted;
                "#,
                c.id,
                c.category_group_id,
                c.name,
                c.hidden,
                c.original_category_group_id,
                c.note,
                c.budgeted,
                c.activity,
                c.balance,
                c.goal_type.as_ref().map(|g| g.to_string()),
                c.goal_creation_month,
                c.goal_target,
                c.goal_target_month,
                c.goal_percentage_complete,
                c.goal_months_to_budget,
                c.goal_under_funded,
                c.goal_overall_funded,
                c.goal_overall_left,
                c.deleted
            ).execute(db_conn_pool).await?;
    }
    Ok(())
}

pub async fn get_categories(db_conn_pool: &PgPool) -> Result<Vec<Category>, sqlx::Error> {
    sqlx::query_as!(
        Category,
        r#"
        SELECT 
            id,
            category_group_id,
            name,
            hidden,
            original_category_group_id,
            note,
            budgeted,
            activity,
            balance,
            goal_type AS "goal_type?: GoalType",
            goal_creation_month,
            goal_target,
            goal_target_month,
            goal_percentage_complete,
            goal_months_to_budget,
            goal_under_funded,
            goal_overall_funded,
            goal_overall_left,
            deleted
        FROM categories
        "#
    )
    .fetch_all(db_conn_pool)
    .await
}

pub async fn get_category_by_id(
    db_conn_pool: &PgPool,
    cat_id: &uuid::Uuid,
) -> Result<Option<Category>, sqlx::Error> {
    sqlx::query_as!(
        Category,
        r#"
        SELECT 
            id,
            category_group_id,
            name,
            hidden,
            original_category_group_id,
            note,
            budgeted,
            activity,
            balance,
            goal_type AS "goal_type?: GoalType",
            goal_creation_month,
            goal_target,
            goal_target_month,
            goal_percentage_complete,
            goal_months_to_budget,
            goal_under_funded,
            goal_overall_funded,
            goal_overall_left,
            deleted
        FROM categories
        WHERE id = $1
        "#,
        cat_id,
    )
    .fetch_optional(db_conn_pool)
    .await
}

pub async fn save_scheduled_transactions(
    db_conn_pool: &PgPool,
    scheduled_transactions: &[ScheduledTransactionDetail],
) -> Result<(), sqlx::Error> {
    for st in scheduled_transactions {
        if st.deleted {
            sqlx::query!(
                r#"
                DELETE FROM scheduled_transactions
                WHERE id = $1
                "#,
                st.id
            )
            .execute(db_conn_pool)
            .await?;
        } else {
            sqlx::query!(
                r#"
                INSERT INTO scheduled_transactions (id, date_first, date_next, frequency, amount, memo, flag_color, account_id, payee_id, category_id, transfer_account_id, deleted, account_name, payee_name, category_name, subtransactions)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
                ON CONFLICT (id) DO UPDATE
                SET date_first = EXCLUDED.date_first,
                date_next = EXCLUDED.date_next,
                frequency = EXCLUDED.frequency,
                amount = EXCLUDED.amount,
                memo = EXCLUDED.memo,
                flag_color = EXCLUDED.flag_color,
                account_id = EXCLUDED.account_id,
                payee_id = EXCLUDED.payee_id,
                category_id = EXCLUDED.category_id,
                transfer_account_id = EXCLUDED.transfer_account_id,
                deleted = EXCLUDED.deleted,
                account_name = EXCLUDED.account_name,
                payee_name = EXCLUDED.payee_name,
                category_name = EXCLUDED.category_name,
                subtransactions = EXCLUDED.subtransactions;
                "#,
                st.id,
                st.date_first,
                st.date_next,
                st.frequency.as_ref().map(|f| f.to_string()),
                st.amount,
                st.memo,
                st.flag_color,
                st.account_id,
                st.payee_id,
                st.category_id,
                st.transfer_account_id,
                st.deleted,
                st.account_name,
                st.payee_name,
                st.category_name,
                serde_json::to_value(&st.subtransactions).unwrap()
            ).execute(db_conn_pool).await?;
        }
    }
    Ok(())
}

pub async fn get_scheduled_transactions(
    db_conn_pool: &PgPool,
) -> Result<Vec<ScheduledTransactionDetail>, sqlx::Error> {
    let scheduled_transactions_rows = sqlx::query!(
        r#"
        SELECT 
            id,
            date_first,
            date_next,
            frequency,
            amount,
            memo,
            flag_color,
            account_id,
            payee_id,
            category_id,
            transfer_account_id,
            deleted,
            account_name,
            payee_name,
            category_name,
            subtransactions
        FROM scheduled_transactions
        "#
    )
    .fetch_all(db_conn_pool)
    .await?;

    let mut saved_scheduled_transactions: Vec<ScheduledTransactionDetail> = vec![];

    for str in scheduled_transactions_rows {
        saved_scheduled_transactions.push(ScheduledTransactionDetail {
            id: str.id,
            date_first: str.date_first,
            date_next: str.date_next,
            frequency: str.frequency.map(|v| v.parse().unwrap()),
            amount: str.amount,
            memo: str.memo,
            flag_color: str.flag_color,
            account_id: str.account_id,
            payee_id: str.payee_id,
            category_id: str.category_id,
            transfer_account_id: str.transfer_account_id,
            deleted: str.deleted,
            account_name: str.account_name,
            payee_name: str.payee_name,
            category_name: str.category_name,
            subtransactions: serde_json::from_value(str.subtransactions).unwrap(),
        })
    }
    let saved_scheduled_transactions = saved_scheduled_transactions;
    Ok(saved_scheduled_transactions)
}

pub async fn save_accounts(db_conn_pool: &PgPool, accounts: &[Account]) -> Result<(), sqlx::Error> {
    for a in accounts {
        sqlx::query!(
                r#"
                INSERT INTO accounts (id, name, type, on_budget, closed, note, balance, cleared_balance, uncleared_balance, transfer_payee_id, direct_import_linked, direct_import_in_error, deleted)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
                ON CONFLICT (id) DO UPDATE SET
                name = EXCLUDED.name,
                type = EXCLUDED.type,
                on_budget = EXCLUDED.on_budget,
                closed = EXCLUDED.closed,
                note = EXCLUDED.note,
                balance = EXCLUDED.balance,
                cleared_balance = EXCLUDED.cleared_balance,
                uncleared_balance = EXCLUDED.uncleared_balance,
                transfer_payee_id = EXCLUDED.transfer_payee_id,
                direct_import_linked = EXCLUDED.direct_import_linked,
                direct_import_in_error = EXCLUDED.direct_import_in_error,
                deleted = EXCLUDED.deleted;
                "#,
                a.id,
                a.name,
                a.account_type.to_string(),
                a.on_budget,
                a.closed,
                a.note,
                a.balance,
                a.cleared_balance,
                a.uncleared_balance,
                a.transfer_payee_id,
                a.direct_import_linked,
                a.direct_import_in_error,
                a.deleted
            ).execute(db_conn_pool).await?;
    }
    Ok(())
}

pub async fn get_accounts(db_conn_pool: &PgPool) -> Result<Vec<Account>, sqlx::Error> {
    sqlx::query_as!(
        Account,
        r#"
        SELECT 
            id,
            name,
            type AS "account_type: AccountType",
            on_budget,
            closed,
            note,
            balance,
            cleared_balance,
            uncleared_balance,
            transfer_payee_id,
            direct_import_linked,
            direct_import_in_error,
            deleted
        FROM accounts
        "#
    )
    .fetch_all(db_conn_pool)
    .await
}