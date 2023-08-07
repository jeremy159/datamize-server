use async_trait::async_trait;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{
    client::Response, error::YnabResult, BaseBudgetSumary, BudgetDetail, BudgetDetailDelta,
    BudgetSettings, BudgetSummary, BudgetSummaryWithAccounts, Client,
};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait BudgetRequests {
    async fn get_budgets(&self) -> YnabResult<Vec<BudgetSummary>>;

    async fn get_budgets_with_accounts(&self) -> YnabResult<Vec<BudgetSummaryWithAccounts>>;

    async fn get_budget(&self) -> YnabResult<BudgetDetail>;

    async fn get_budget_delta(
        &self,
        last_knowledge_of_server: Option<i64>,
    ) -> YnabResult<BudgetDetailDelta>;

    async fn get_budget_settings(&self) -> YnabResult<BudgetSettings>;
}

#[async_trait]
impl BudgetRequests for Client {
    async fn get_budgets(&self) -> YnabResult<Vec<BudgetSummary>> {
        self.get_budgets_request(false).await
    }

    async fn get_budgets_with_accounts(&self) -> YnabResult<Vec<BudgetSummaryWithAccounts>> {
        self.get_budgets_request(true).await
    }

    async fn get_budget(&self) -> YnabResult<BudgetDetail> {
        Ok(self.get_budget_request(None).await?.budget)
    }

    async fn get_budget_delta(
        &self,
        last_knowledge_of_server: Option<i64>,
    ) -> YnabResult<BudgetDetailDelta> {
        self.get_budget_request(last_knowledge_of_server).await
    }

    async fn get_budget_settings(&self) -> YnabResult<BudgetSettings> {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct Inner {
            settings: BudgetSettings,
        }

        let path = format!("budgets/{}/settings", self.get_budget_id());

        let body = self.get(&path).send().await?.text().await?;

        let resp: Response<Inner> = Client::convert_resp(body)?;
        Ok(resp.data.settings)
    }
}

impl Client {
    async fn get_budgets_request<T>(&self, with_accounts: bool) -> YnabResult<Vec<T>>
    where
        T: AsRef<BaseBudgetSumary> + DeserializeOwned,
    {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct Inner<I> {
            budgets: Vec<I>,
            default_budget: Option<I>,
        }

        let body = match with_accounts {
            true => self.get_with_query("budgets", &[("include_accounts", "true")]),
            false => self.get("budgets"),
        }
        .send()
        .await?
        .text()
        .await?;

        let resp: Response<Inner<T>> = Client::convert_resp(body)?;
        Ok(resp.data.budgets)
    }

    async fn get_budget_request(
        &self,
        last_knowledge_of_server: Option<i64>,
    ) -> YnabResult<BudgetDetailDelta> {
        let path = format!("budgets/{}", self.get_budget_id());

        let body = match last_knowledge_of_server {
            Some(k) => self.get_with_query(&path, &[("last_knowledge_of_server", k)]),
            None => self.get(&path),
        }
        .send()
        .await?
        .text()
        .await?;

        let resp: Response<BudgetDetailDelta> = Client::convert_resp(body)?;
        Ok(resp.data)
    }
}
