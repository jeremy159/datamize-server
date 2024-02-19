mod months;
mod refresh_resources;
mod resources;
mod saving_rates;
mod years;

use std::fmt::Display;

use serde::Serialize;

use crate::helpers::TestApp;

impl TestApp {
    pub async fn get_years(&self) -> reqwest::Response {
        self.api_client
            .get(&format!("{}/api/balance_sheet/years", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn create_year<B>(&self, body: &B) -> reqwest::Response
    where
        B: Serialize,
    {
        self.api_client
            .post(&format!("{}/api/balance_sheet/years", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn get_year<Y>(&self, year: Y) -> reqwest::Response
    where
        Y: Display,
    {
        self.api_client
            .get(&format!(
                "{}/api/balance_sheet/years/{}",
                &self.address, year
            ))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn delete_year<Y>(&self, year: Y) -> reqwest::Response
    where
        Y: Display,
    {
        self.api_client
            .delete(&format!(
                "{}/api/balance_sheet/years/{}",
                &self.address, year
            ))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn get_all_months(&self) -> reqwest::Response {
        self.api_client
            .get(&format!("{}/api/balance_sheet/months", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn get_months<Y>(&self, year: Y) -> reqwest::Response
    where
        Y: Display,
    {
        self.api_client
            .get(&format!(
                "{}/api/balance_sheet/years/{}/months",
                &self.address, year
            ))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn create_month<Y, B>(&self, year: Y, body: &B) -> reqwest::Response
    where
        Y: Display,
        B: Serialize,
    {
        self.api_client
            .post(&format!(
                "{}/api/balance_sheet/years/{}/months",
                &self.address, year
            ))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn get_month<Y, M>(&self, year: Y, month: M) -> reqwest::Response
    where
        Y: Display,
        M: Display,
    {
        self.api_client
            .get(&format!(
                "{}/api/balance_sheet/years/{}/months/{}",
                &self.address, year, month
            ))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn delete_month<Y, M>(&self, year: Y, month: M) -> reqwest::Response
    where
        Y: Display,
        M: Display,
    {
        self.api_client
            .delete(&format!(
                "{}/api/balance_sheet/years/{}/months/{}",
                &self.address, year, month
            ))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn refresh_resources(&self) -> reqwest::Response {
        self.api_client
            .post(&format!(
                "{}/api/balance_sheet/resources/refresh",
                &self.address
            ))
            .header("Content-Type", "application/json")
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn get_all_resources(&self) -> reqwest::Response {
        self.api_client
            .get(&format!("{}/api/balance_sheet/resources", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn get_resources<Y>(&self, year: Y) -> reqwest::Response
    where
        Y: Display,
    {
        self.api_client
            .get(&format!(
                "{}/api/balance_sheet/years/{}/resources",
                &self.address, year
            ))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn create_resource<B>(&self, body: &B) -> reqwest::Response
    where
        B: Serialize,
    {
        self.api_client
            .post(&format!("{}/api/balance_sheet/resources", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn get_resource<R>(&self, res_id: R) -> reqwest::Response
    where
        R: Display,
    {
        self.api_client
            .get(&format!(
                "{}/api/balance_sheet/resources/{}",
                &self.address, res_id
            ))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn update_resource<R, B>(&self, res_id: R, body: &B) -> reqwest::Response
    where
        R: Display,
        B: Serialize,
    {
        self.api_client
            .put(&format!(
                "{}/api/balance_sheet/resources/{}",
                &self.address, res_id
            ))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn delete_resource<R>(&self, res_id: R) -> reqwest::Response
    where
        R: Display,
    {
        self.api_client
            .delete(&format!(
                "{}/api/balance_sheet/resources/{}",
                &self.address, res_id
            ))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn get_saving_rates<Y>(&self, year: Y) -> reqwest::Response
    where
        Y: Display,
    {
        self.api_client
            .get(&format!(
                "{}/api/balance_sheet/years/{}/saving_rates",
                &self.address, year
            ))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn create_saving_rate<B>(&self, body: &B) -> reqwest::Response
    where
        B: Serialize,
    {
        self.api_client
            .post(&format!("{}/api/balance_sheet/saving_rates", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn get_saving_rate<R>(&self, id: R) -> reqwest::Response
    where
        R: Display,
    {
        self.api_client
            .get(&format!(
                "{}/api/balance_sheet/saving_rates/{}",
                &self.address, id
            ))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn update_saving_rate<R, B>(&self, id: R, body: &B) -> reqwest::Response
    where
        R: Display,
        B: Serialize,
    {
        self.api_client
            .put(&format!(
                "{}/api/balance_sheet/saving_rates/{}",
                &self.address, id
            ))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn delete_saving_rate<R>(&self, id: R) -> reqwest::Response
    where
        R: Display,
    {
        self.api_client
            .delete(&format!(
                "{}/api/balance_sheet/saving_rates/{}",
                &self.address, id
            ))
            .send()
            .await
            .expect("Failed to execute request.")
    }
}
