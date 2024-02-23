use std::fmt::Debug;

use axum_login::tower_sessions::{
    cookie::time::OffsetDateTime,
    session::{Id, Record},
    session_store, SessionStore,
};
use datamize_domain::async_trait;
use fred;
use fred::{prelude::KeysInterface, types::Expiration};

#[derive(Debug, thiserror::Error)]
pub enum RedisStoreError {
    #[error(transparent)]
    Redis(#[from] fred::error::RedisError),

    #[error(transparent)]
    Decode(#[from] rmp_serde::decode::Error),

    #[error(transparent)]
    Encode(#[from] rmp_serde::encode::Error),
}

impl From<RedisStoreError> for session_store::Error {
    fn from(err: RedisStoreError) -> Self {
        match err {
            RedisStoreError::Redis(inner) => session_store::Error::Backend(inner.to_string()),
            RedisStoreError::Decode(inner) => session_store::Error::Decode(inner.to_string()),
            RedisStoreError::Encode(inner) => session_store::Error::Encode(inner.to_string()),
        }
    }
}

/// A Redis session store.
#[derive(Debug, Clone, Default)]
pub struct RedisStore<C: KeysInterface + Send + Sync> {
    client: C,
}

impl<C: KeysInterface + Send + Sync> RedisStore<C> {
    /// Create a new Redis store with the provided client.
    pub fn new(client: C) -> Self {
        Self { client }
    }
}

#[async_trait]
impl<C> SessionStore for RedisStore<C>
where
    C: KeysInterface + Send + Sync + Debug + 'static,
{
    #[tracing::instrument(skip_all)]
    async fn save(&self, record: &Record) -> session_store::Result<()> {
        let expire = Some(Expiration::EXAT(OffsetDateTime::unix_timestamp(
            record.expiry_date,
        )));

        self.client
            .set(
                record.id.to_string(),
                rmp_serde::to_vec(&record)
                    .map_err(RedisStoreError::Encode)?
                    .as_slice(),
                expire,
                None,
                false,
            )
            .await
            .map_err(RedisStoreError::Redis)?;

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn load(&self, session_id: &Id) -> session_store::Result<Option<Record>> {
        let data = self
            .client
            .get::<Option<Vec<u8>>, _>(session_id.to_string())
            .await
            .map_err(RedisStoreError::Redis)?;

        if let Some(data) = data {
            Ok(Some(
                rmp_serde::from_slice(&data).map_err(RedisStoreError::Decode)?,
            ))
        } else {
            Ok(None)
        }
    }

    #[tracing::instrument(skip_all)]
    async fn delete(&self, session_id: &Id) -> session_store::Result<()> {
        self.client
            .del(session_id.to_string())
            .await
            .map_err(RedisStoreError::Redis)?;
        Ok(())
    }
}
