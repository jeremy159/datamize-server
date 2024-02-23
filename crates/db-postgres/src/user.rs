use std::sync::Arc;

use datamize_domain::{
    async_trait,
    db::{DbResult, UserRepo},
    secrecy::ExposeSecret,
    User, Uuid,
};
use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct PostgresUserRepo {
    pub db_conn_pool: PgPool,
}

impl PostgresUserRepo {
    pub fn new_arced(db_conn_pool: PgPool) -> Arc<Self> {
        Arc::new(Self { db_conn_pool })
    }
}

#[async_trait]
impl UserRepo for PostgresUserRepo {
    #[tracing::instrument(skip_all)]
    async fn add(&self, user: &User) -> DbResult<()> {
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
            user.access_token.expose_secret(),
            user.refresh_token.as_ref().map(|r| r.expose_secret()),
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
                *
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
                *
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
