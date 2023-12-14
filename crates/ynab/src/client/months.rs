use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    client::Response, error::YnabResult, Client, MonthDetail, MonthSummary, MonthSummaryDelta,
};

#[async_trait]
pub trait MonthRequests {
    async fn get_months(&self) -> YnabResult<Vec<MonthSummary>>;

    async fn get_months_delta(
        &self,
        last_knowledge_of_server: Option<i64>,
    ) -> YnabResult<MonthSummaryDelta>;

    async fn get_month_by_date(&self, date: &str) -> YnabResult<MonthDetail>;
}

#[async_trait]
impl MonthRequests for Client {
    async fn get_months(&self) -> YnabResult<Vec<MonthSummary>> {
        Ok(self.get_months_request(None).await?.months)
    }

    async fn get_months_delta(
        &self,
        last_knowledge_of_server: Option<i64>,
    ) -> YnabResult<MonthSummaryDelta> {
        self.get_months_request(last_knowledge_of_server).await
    }

    async fn get_month_by_date(&self, date: &str) -> YnabResult<MonthDetail> {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct Inner {
            month: MonthDetail,
        }

        let path = format!("budgets/{}/months/{}", self.get_budget_id(), date);

        let body = self.get(&path).send().await?.text().await?;

        let resp: Response<Inner> = Client::convert_resp(body)?;
        Ok(resp.data.month)
    }
}

impl Client {
    async fn get_months_request(
        &self,
        last_knowledge_of_server: Option<i64>,
    ) -> YnabResult<MonthSummaryDelta> {
        let path = format!("budgets/{}/months", self.get_budget_id());

        let body = match last_knowledge_of_server {
            Some(k) => self.get_with_query(&path, &[("last_knowledge_of_server", k)]),
            None => self.get(&path),
        }
        .send()
        .await?
        .text()
        .await?;

        let resp: Response<MonthSummaryDelta> = Client::convert_resp(body)?;
        Ok(resp.data)
    }
}

#[cfg(any(feature = "testutils", test))]
mockall::mock! {
    pub MonthRequestsImpl {}

    impl Clone for MonthRequestsImpl {
        fn clone(&self) -> Self;
    }

    #[async_trait]
    impl MonthRequests for MonthRequestsImpl {
        async fn get_months(&self) -> YnabResult<Vec<MonthSummary>>;
        async fn get_months_delta(
            &self,
            last_knowledge_of_server: Option<i64>,
        ) -> YnabResult<MonthSummaryDelta>;
        async fn get_month_by_date(&self, date: &str) -> YnabResult<MonthDetail>;
    }
}
