use async_trait::async_trait;

use crate::{client::Response, error::YnabResult, Client, User, UserResp};

#[async_trait]
pub trait UserRequests {
    async fn get_user(&self) -> YnabResult<User>;
}

#[async_trait]
impl UserRequests for Client {
    async fn get_user(&self) -> YnabResult<User> {
        let body = self.get("user").send().await?.text().await?;

        let resp: Response<UserResp> = Client::convert_resp(body)?;
        Ok(resp.data.user)
    }
}

#[cfg(any(feature = "testutils", test))]
mockall::mock! {
    pub UserRequestsImpl {}

    impl Clone for UserRequestsImpl {
        fn clone(&self) -> Self;
    }

    #[async_trait]
    impl UserRequests for UserRequestsImpl {
        async fn get_user(&self) -> YnabResult<User>;
    }
}
