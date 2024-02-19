use chrono::{Datelike, Local};
use datamize_domain::{Month, MonthNum, Uuid};
use fake::Dummy;
use pretty_assertions::assert_eq;
use serde::Serialize;
use sqlx::PgPool;
use wiremock::{
    matchers::{any, path_regex},
    Mock, ResponseTemplate,
};

use crate::helpers::spawn_app;

#[derive(Debug, Clone, Serialize)]
struct CreateYear {
    year: i32,
}

#[derive(Debug, Clone, Serialize)]
struct CreateMonth {
    month: MonthNum,
}

#[sqlx::test(migrations = "../db-postgres/migrations")]
async fn refresh_resources_returns_a_404_if_curent_year_does_not_exist(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;

    // Act
    let response = app.refresh_resources().await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "../db-postgres/migrations")]
async fn refresh_resources_should_not_call_ynab_server_if_year_does_not_exist(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&app.ynab_server)
        .await;

    // Act
    app.refresh_resources().await;

    // Assert
    // Mock verifies on Drop that we haven't sent the request to ynab
}

#[sqlx::test(migrations = "../db-postgres/migrations")]
async fn refresh_resources_should_call_ynab_server_even_if_month_does_not_exist(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let current_date = Local::now().date_naive();
    let year = current_date.year();
    let year_body = CreateYear { year };
    app.create_year(&year_body).await;
    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.ynab_server)
        .await;

    // Act
    app.refresh_resources().await;

    // Assert
    // Mock verifies on Drop that we haven't sent the request to ynab
}

#[sqlx::test(migrations = "../db-postgres/migrations")]
async fn refresh_resources_should_get_accounts_from_ynab(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let current_date = Local::now().date_naive();
    let year = current_date.year();
    let year_body = CreateYear { year };
    app.create_year(&year_body).await;
    let month = current_date.month().try_into().unwrap();
    let month_body = CreateMonth { month };
    app.create_month(year, &month_body).await;

    Mock::given(path_regex("/accounts"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.ynab_server)
        .await;

    // Act
    app.refresh_resources().await;

    // Assert
    // Mock verifies on Drop that we haven't sent the request to ynab
}

#[sqlx::test(migrations = "../db-postgres/migrations")]
async fn refresh_resources_returns_a_200_and_create_month_in_db_if_curent_month_does_not_exist(
    pool: PgPool,
) {
    // Arange
    let app = spawn_app(pool).await;
    let current_date = Local::now().date_naive();
    let year = current_date.year();
    let year_body = CreateYear { year };
    app.create_year(&year_body).await;
    Mock::given(path_regex("/accounts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(BodyResp {
            data: AccountsResp {
                accounts: vec![],
                server_knowledge: 0,
            },
        }))
        .mount(&app.ynab_server)
        .await;

    // Act
    let response = app.refresh_resources().await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let month = current_date.month();
    let saved = app.get_month(year, month).await;
    let value: Month = serde_json::from_str(&saved.text().await.unwrap()).unwrap();

    assert_eq!(value.month, month.try_into().unwrap());
}

#[derive(Debug, Clone, Serialize)]
struct AccountsResp {
    pub accounts: Vec<DummyAccount>,
    pub server_knowledge: i64,
}

#[derive(Debug, Clone, Serialize)]
struct BodyResp {
    pub data: AccountsResp,
}

#[derive(Debug, Clone, Serialize, Dummy)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
enum DummyAccountType {
    Checking,
    Savings,
    Cash,
    CreditCard,
    LineOfCredit,
    OtherAsset,
    OtherLiability,
    Mortgage,
    AutoLoan,
    StudentLoan,
}

#[derive(Debug, Clone, Serialize, Dummy)]
struct DummyAccount {
    pub id: Uuid,
    pub name: String,
    #[serde(rename = "type")]
    pub account_type: DummyAccountType,
    pub on_budget: bool,
    pub closed: bool,
    pub note: Option<String>,
    pub balance: i64,
    pub cleared_balance: i64,
    pub uncleared_balance: i64,
    pub transfer_payee_id: Uuid,
    pub direct_import_linked: Option<bool>,
    pub direct_import_in_error: Option<bool>,
    pub deleted: bool,
}
