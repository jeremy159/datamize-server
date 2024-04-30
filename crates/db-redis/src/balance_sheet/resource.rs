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
        let res: Vec<String> = self
            .redis_conn_pool
            .get(&format!("{}_{}_order", year, category))
            .await?;

        res.into_iter()
            .map(|id| Uuid::try_parse(&id).map_err(Into::into))
            .collect()
    }

    #[tracing::instrument(skip(self))]
    async fn set_order(
        &self,
        year: i32,
        category: &ResourceCategory,
        order: &[Uuid],
    ) -> DbResult<()> {
        let order = order.iter().map(|id| id.to_string()).collect::<Vec<_>>();

        self.redis_conn_pool
            .set(
                &format!("{}_{}_order", year, category),
                order,
                None,
                None,
                false,
            )
            .await?;
        Ok(())
    }
}
