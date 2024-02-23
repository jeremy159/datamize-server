use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::{db::error::DbResult, User};

#[async_trait]
pub trait UserRepo: Send + Sync {
    async fn add(&self, user: &User) -> DbResult<()>;
    async fn get(&self, id: Uuid) -> DbResult<User>;
    async fn get_opt(&self, id: Uuid) -> DbResult<Option<User>>;
}

pub type DynUserRepo = Arc<dyn UserRepo>;
