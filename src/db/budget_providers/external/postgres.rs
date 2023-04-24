use secrecy::{ExposeSecret, Secret};
use sqlx::PgPool;

use crate::web_scraper::account::{EncryptedPassword, WebScrapingAccount};

#[tracing::instrument(skip_all)]
pub async fn add_new_external_account(
    db_conn_pool: &PgPool,
    account: &WebScrapingAccount,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO external_accounts (id, name, type, balance, username, encrypted_password, deleted)
        VALUES ($1, $2, $3, $4, $5, $6, $7);
        "#,
        account.id,
        account.name,
        account.account_type.to_string(),
        account.balance,
        account.username,
        account.encrypted_password.expose_secret().as_ref(),
        account.deleted,
    )
    .execute(db_conn_pool)
    .await?;

    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn update_external_account(
    db_conn_pool: &PgPool,
    account: &WebScrapingAccount,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO external_accounts (id, name, type, balance, username, encrypted_password, deleted)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (id) DO UPDATE SET
        name = EXCLUDED.name,
        type = EXCLUDED.type,
        balance = EXCLUDED.balance,
        username = EXCLUDED.username,
        encrypted_password = EXCLUDED.encrypted_password,
        deleted = EXCLUDED.deleted;
        "#,
        account.id,
        account.name,
        account.account_type.to_string(),
        account.balance,
        account.username,
        account.encrypted_password.expose_secret().as_ref(),
        account.deleted,
    )
    .execute(db_conn_pool)
    .await?;

    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn get_all_external_accounts(
    db_conn_pool: &PgPool,
) -> Result<Vec<WebScrapingAccount>, sqlx::Error> {
    sqlx::query!(
        r#"
        SELECT * FROM external_accounts;
        "#
    )
    .map(|row| WebScrapingAccount {
        id: row.id,
        name: row.name,
        account_type: row.r#type.parse().unwrap(),
        balance: row.balance,
        username: row.username,
        encrypted_password: Secret::new(EncryptedPassword::new(row.encrypted_password)),
        deleted: row.deleted,
    })
    .fetch_all(db_conn_pool)
    .await
}

#[tracing::instrument(skip_all)]
pub async fn get_external_account_by_name(
    db_conn_pool: &PgPool,
    name: &str,
) -> Result<Option<WebScrapingAccount>, sqlx::Error> {
    sqlx::query!(
        r#"
        SELECT * FROM external_accounts
        WHERE name = $1;
        "#,
        name,
    )
    .map(|row| WebScrapingAccount {
        id: row.id,
        name: row.name,
        account_type: row.r#type.parse().unwrap(),
        balance: row.balance,
        username: row.username,
        encrypted_password: Secret::new(EncryptedPassword::new(row.encrypted_password)),
        deleted: row.deleted,
    })
    .fetch_optional(db_conn_pool)
    .await
}
