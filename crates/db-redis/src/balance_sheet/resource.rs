use std::sync::Arc;

use datamize_domain::{
    async_trait,
    db::{DbResult, FinResOrderRepo},
    ResourceCategory, Uuid,
};
use fred::{clients::RedisPool, interfaces::KeysInterface};

#[derive(Clone)]
pub struct RedisFinResOrderRepo {
    pub redis_conn_pool: RedisPool,
}

impl RedisFinResOrderRepo {
    pub fn new_arced(redis_conn_pool: RedisPool) -> Arc<Self> {
        Arc::new(Self { redis_conn_pool })
    }
}

#[async_trait]
impl FinResOrderRepo for RedisFinResOrderRepo {
    #[tracing::instrument(skip(self))]
    async fn get_order(&self, year: i32, category: &ResourceCategory) -> DbResult<Vec<Uuid>> {
        let res: Option<String> = self
            .redis_conn_pool
            .get(&format!("{}_{}_order", year, category))
            .await?;

        match res {
            Some(res) => Ok(serde_json::from_str(&res)?),
            None => Ok(vec![]),
        }
    }

    #[tracing::instrument(skip(self))]
    async fn set_order(
        &self,
        year: i32,
        category: &ResourceCategory,
        order: &[Uuid],
    ) -> DbResult<()> {
        let serialized = serde_json::to_string(order)?;

        self.redis_conn_pool
            .set(
                &format!("{}_{}_order", year, category),
                serialized,
                None,
                None,
                false,
            )
            .await?;
        Ok(())
    }
}
