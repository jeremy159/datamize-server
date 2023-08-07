use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{client::Response, error::YnabResult, Account, AccountsDelta, Client, SaveAccount};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait AccountRequests {
    async fn get_accounts(&self) -> YnabResult<Vec<Account>>;

    async fn get_accounts_delta(
        &self,
        last_knowledge_of_server: Option<i64>,
    ) -> YnabResult<AccountsDelta>;

    async fn create_account(&self, data: SaveAccount) -> YnabResult<Account>;

    async fn get_account_by_id(&self, account_id: &str) -> YnabResult<Account>;
}

#[async_trait]
impl AccountRequests for Client {
    async fn get_accounts(&self) -> YnabResult<Vec<Account>> {
        Ok(self.get_accounts_request(None).await?.accounts)
    }

    async fn get_accounts_delta(
        &self,
        last_knowledge_of_server: Option<i64>,
    ) -> YnabResult<AccountsDelta> {
        self.get_accounts_request(last_knowledge_of_server).await
    }

    async fn create_account(&self, data: SaveAccount) -> YnabResult<Account> {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct Body {
            account: SaveAccount,
        }
        let body: Body = Body { account: data };

        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct Inner {
            account: Account,
        }

        let path = format!("budgets/{}/accounts", self.get_budget_id());

        let body_resp = self.post(&path, Some(&body)).send().await?.text().await?;

        let resp: Response<Inner> = Client::convert_resp(body_resp)?;
        Ok(resp.data.account)
    }

    async fn get_account_by_id(&self, account_id: &str) -> YnabResult<Account> {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct Inner {
            account: Account,
        }

        let path = format!("budgets/{}/accounts/{}", self.get_budget_id(), account_id);

        let body = self.get(&path).send().await?.text().await?;

        let resp: Response<Inner> = Client::convert_resp(body)?;
        Ok(resp.data.account)
    }
}

impl Client {
    async fn get_accounts_request(
        &self,
        last_knowledge_of_server: Option<i64>,
    ) -> YnabResult<AccountsDelta> {
        let path = format!("budgets/{}/accounts", self.get_budget_id());

        let body = match last_knowledge_of_server {
            Some(k) => self.get_with_query(&path, &[("last_knowledge_of_server", k)]),
            None => self.get(&path),
        }
        .send()
        .await?
        .text()
        .await?;

        let resp: Response<AccountsDelta> = Client::convert_resp(body)?;
        Ok(resp.data)
    }
}
