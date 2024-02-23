use std::sync::Arc;

use datamize_domain::{
    async_trait,
    db::{DbResult, UserRepo},
    secrecy::ExposeSecret,
    User, Uuid,
};
use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct SqliteUserRepo {
    pub db_conn_pool: SqlitePool,
}

impl SqliteUserRepo {
    pub fn new_arced(db_conn_pool: SqlitePool) -> Arc<Self> {
        Arc::new(Self { db_conn_pool })
    }
}

#[async_trait]
impl UserRepo for SqliteUserRepo {
    #[tracing::instrument(skip_all)]
    async fn add(&self, user: &User) -> DbResult<()> {
        let access_token = user.access_token.expose_secret();
        let refresh_token = user.refresh_token.as_ref().map(|r| r.expose_secret());

        sqlx::query!(
            r#"
            INSERT INTO users (id, access_token, refresh_token, expires_at)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (id) DO UPDATE SET
            access_token = EXCLUDED.access_token,
            refresh_token = EXCLUDED.refresh_token,
            expires_at = EXCLUDED.expires_at;
            "#,
            user.ynab_user.id,
            access_token,
            refresh_token,
            user.expires_at,
        )
        .execute(&self.db_conn_pool)
        .await?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn get(&self, id: Uuid) -> DbResult<User> {
        sqlx::query!(
            r#"
            SELECT
                id as "id: Uuid",
                access_token as "access_token: String",
                refresh_token as "refresh_token: String",
                expires_at as "expires_at: chrono::DateTime<chrono::Utc>"
            FROM users
            WHERE id = $1;
            "#,
            id,
        )
        .fetch_one(&self.db_conn_pool)
        .await
        .map_err(Into::into)
        .map(|row| User {
            ynab_user: ynab::User { id: row.id },
            access_token: row.access_token.into(),
            refresh_token: row.refresh_token.map(|r| r.into()),
            expires_at: row.expires_at,
        })
    }

    #[tracing::instrument(skip(self))]
    async fn get_opt(&self, id: Uuid) -> DbResult<Option<User>> {
        sqlx::query!(
            r#"
            SELECT
                id as "id: Uuid",
                access_token as "access_token: String",
                refresh_token as "refresh_token: String",
                expires_at as "expires_at: chrono::DateTime<chrono::Utc>"
            FROM users
            WHERE id = $1;
            "#,
            id,
        )
        .fetch_optional(&self.db_conn_pool)
        .await
        .map_err(Into::into)
        .map(|row| {
            row.map(|row| User {
                ynab_user: ynab::User { id: row.id },
                access_token: row.access_token.into(),
                refresh_token: row.refresh_token.map(|r| r.into()),
                expires_at: row.expires_at,
            })
        })
    }
}
