use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{client::Response, error::YnabResult, Client, PayeeLocation};

#[cfg_attr(any(feature = "testutils", test), mockall::automock)]
#[async_trait]
pub trait PayeeLocationRequests {
    async fn get_payee_locations(&self) -> YnabResult<Vec<PayeeLocation>>;

    async fn get_payee_locations_for(&self, payee_id: &str) -> YnabResult<Vec<PayeeLocation>>;

    async fn get_payee_location_by_id(&self, payee_location_id: &str) -> YnabResult<PayeeLocation>;
}

#[async_trait]
impl PayeeLocationRequests for Client {
    async fn get_payee_locations(&self) -> YnabResult<Vec<PayeeLocation>> {
        self.get_payee_locations_request(None).await
    }

    async fn get_payee_locations_for(&self, payee_id: &str) -> YnabResult<Vec<PayeeLocation>> {
        self.get_payee_locations_request(Some(payee_id)).await
    }

    async fn get_payee_location_by_id(&self, payee_location_id: &str) -> YnabResult<PayeeLocation> {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct Inner {
            payee_location: PayeeLocation,
        }

        let path = format!(
            "budgets/{}/payee_locations/{}",
            self.get_budget_id(),
            payee_location_id
        );

        let body = self.get(&path).send().await?.text().await?;

        let resp: Response<Inner> = Client::convert_resp(body)?;
        Ok(resp.data.payee_location)
    }
}

impl Client {
    async fn get_payee_locations_request(
        &self,
        payee_id: Option<&str>,
    ) -> YnabResult<Vec<PayeeLocation>> {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct Inner {
            payee_locations: Vec<PayeeLocation>,
        }

        let path = match payee_id {
            Some(p) => format!(
                "budgets/{}/payees/{}/payee_locations",
                self.get_budget_id(),
                p
            ),
            None => format!("budgets/{}/payee_locations", self.get_budget_id()),
        };

        let body = self.get(&path).send().await?.text().await?;

        let resp: Response<Inner> = Client::convert_resp(body)?;
        Ok(resp.data.payee_locations)
    }
}
