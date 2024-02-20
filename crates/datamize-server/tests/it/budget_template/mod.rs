mod budgeters;
mod details;
mod expenses_categorization;
mod summary;
mod transactions;

use std::fmt::Display;

use datamize_domain::Uuid;
use fake::Dummy;
use serde::Serialize;

use crate::helpers::TestApp;

impl TestApp {
    pub async fn get_template_details(&self) -> reqwest::Response {
        self.api_client
            .get(&format!("{}/api/template/details", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn get_template_summary(&self) -> reqwest::Response {
        self.api_client
            .get(&format!("{}/api/template/summary", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn get_template_transactions(&self) -> reqwest::Response {
        self.api_client
            .get(&format!("{}/api/template/transactions", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn get_all_budgeters(&self) -> reqwest::Response {
        self.api_client
            .get(&format!("{}/api/template/budgeters", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn create_budgeter<B>(&self, body: &B) -> reqwest::Response
    where
        B: Serialize,
    {
        self.api_client
            .post(&format!("{}/api/template/budgeter", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn get_budgeter<I>(&self, id: I) -> reqwest::Response
    where
        I: Display,
    {
        self.api_client
            .get(&format!("{}/api/template/budgeter/{}", &self.address, id))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn update_budgeter<I, B>(&self, id: I, body: &B) -> reqwest::Response
    where
        I: Display,
        B: Serialize,
    {
        self.api_client
            .put(&format!("{}/api/template/budgeter/{}", &self.address, id))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn delete_budgeter<I>(&self, id: I) -> reqwest::Response
    where
        I: Display,
    {
        self.api_client
            .delete(&format!("{}/api/template/budgeter/{}", &self.address, id))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn get_all_expenses_categorization(&self) -> reqwest::Response {
        self.api_client
            .get(&format!(
                "{}/api/template/expenses_categorization",
                &self.address
            ))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn update_all_expenses_categorization<B>(&self, body: &B) -> reqwest::Response
    where
        B: Serialize,
    {
        self.api_client
            .put(&format!(
                "{}/api/template/expenses_categorization",
                &self.address
            ))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn get_expense_categorization<I>(&self, id: I) -> reqwest::Response
    where
        I: Display,
    {
        self.api_client
            .get(&format!(
                "{}/api/template/expense_categorization/{}",
                &self.address, id
            ))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn update_expense_categorization<I, B>(&self, id: I, body: &B) -> reqwest::Response
    where
        I: Display,
        B: Serialize,
    {
        self.api_client
            .put(&format!(
                "{}/api/template/expense_categorization/{}",
                &self.address, id
            ))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct BodyResp<T> {
    pub data: T,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScheduledTransactionsResp {
    pub scheduled_transactions: Vec<DummyScheduledTransactionDetail>,
    pub server_knowledge: i64,
}

#[derive(Debug, Clone, Serialize, Dummy)]
pub struct DummyScheduledTransactionDetail {
    pub id: Uuid,
    #[dummy(default)]
    pub date_first: chrono::NaiveDate,
    #[dummy(default)]
    pub date_next: chrono::NaiveDate,
    pub frequency: DummyRecurFrequency,
    #[dummy(faker = "-100000..100000")]
    pub amount: i64,
    pub memo: Option<String>,
    pub flag_color: Option<String>,
    pub account_id: Uuid,
    pub payee_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub transfer_account_id: Option<Uuid>,
    pub deleted: bool,
    pub account_name: String,
    pub payee_name: Option<String>,
    pub category_name: Option<String>,
    #[dummy(default)] // Skip complex scheduled transactions for now.
    pub subtransactions: Vec<DummyScheduledSubTransaction>,
}

#[derive(Debug, Clone, Serialize, Dummy)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub enum DummyRecurFrequency {
    Never,
    Daily,
    Weekly,
    EveryOtherWeek,
    TwiceAMonth,
    Every4Weeks,
    Monthly,
    EveryOtherMonth,
    Every3Months,
    Every4Months,
    TwiceAYear,
    Yearly,
    EveryOtherYear,
}

#[derive(Debug, Clone, Serialize, Dummy)]
pub struct DummyScheduledSubTransaction {
    pub id: Uuid,
    pub scheduled_transaction_id: Uuid,
    #[dummy(faker = "-100000..100000")]
    pub amount: i64,
    pub memo: Option<String>,
    pub payee_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub transfer_account_id: Option<Uuid>,
    pub deleted: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct CategoriesResp {
    pub category_groups: Vec<DummyCategoryGroupWithCategories>,
    pub server_knowledge: i64,
}

#[derive(Debug, Clone, Serialize, Dummy)]
pub struct DummyCategoryGroupWithCategories {
    pub id: Uuid,
    pub name: String,
    pub hidden: bool,
    pub deleted: bool,
    pub categories: Vec<DummyCategory>,
}

#[derive(Debug, Clone, Serialize, Dummy)]
pub struct DummyCategory {
    pub id: Uuid,
    pub category_group_id: Uuid,
    pub category_group_name: String,
    pub name: String,
    pub hidden: bool,
    pub original_category_group_id: Option<Uuid>,
    pub note: Option<String>,
    #[dummy(faker = "0..100000")]
    pub budgeted: i64,
    #[dummy(faker = "-100000..100000")]
    pub activity: i64,
    #[dummy(faker = "0..100000")]
    pub balance: i64,
    pub goal_type: Option<DummyGoalType>,
    #[dummy(faker = "0..32")]
    pub goal_day: Option<i32>,
    #[dummy(faker = "0..15")]
    pub goal_cadence: Option<i32>,
    #[dummy(faker = "1..14")]
    pub goal_cadence_frequency: Option<i32>,
    #[dummy(default)]
    pub goal_creation_month: Option<chrono::NaiveDate>,
    #[dummy(faker = "0..100000")]
    pub goal_target: Option<i64>,
    #[dummy(default)]
    pub goal_target_month: Option<chrono::NaiveDate>,
    #[dummy(faker = "0..100")]
    pub goal_percentage_complete: Option<i32>,
    #[dummy(faker = "0..100")]
    pub goal_months_to_budget: Option<i32>,
    #[dummy(faker = "0..100000")]
    pub goal_under_funded: Option<i64>,
    #[dummy(faker = "0..100000")]
    pub goal_overall_funded: Option<i64>,
    #[dummy(faker = "0..100000")]
    pub goal_overall_left: Option<i64>,
    pub deleted: bool,
}

#[derive(Debug, Clone, Serialize, Dummy)]
pub enum DummyGoalType {
    #[serde(rename = "TB")]
    TargetBalance,
    #[serde(rename = "TBD")]
    TargetBalanceByDate,
    #[serde(rename = "MF")]
    MonthlyFunding,
    #[serde(rename = "NEED")]
    PlanYourSpending,
    #[serde(rename = "DEBT")]
    Debt,
}
