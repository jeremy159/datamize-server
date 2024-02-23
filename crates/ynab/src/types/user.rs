use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
/// See https://api.youneedabudget.com/v1#/User/getUser
pub struct UserResp {
    pub user: User,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// See https://api.youneedabudget.com/v1#/User/getUser
pub struct User {
    pub id: Uuid,
}
