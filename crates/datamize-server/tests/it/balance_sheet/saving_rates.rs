use datamize_domain::{SavingRate, Uuid};
use fake::{Dummy, Fake, Faker};
use pretty_assertions::assert_eq;
use serde::Serialize;
use sqlx::PgPool;
use wiremock::{matchers::path_regex, Mock, ResponseTemplate};

use crate::helpers::spawn_app;

#[sqlx::test(migrations = "../db-postgres/migrations")]
async fn get_all_of_year_returns_empty_list_when_nothing_in_db(pool: PgPool) {
    let app = spawn_app(pool).await;
    Mock::given(path_regex("/transactions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(BodyResp {
            data: TransactionsResp {
                transactions: vec![],
                server_knowledge: 0,
            },
        }))
        .expect(0)
        .mount(&app.ynab_server)
        .await;

    // Act
    let response = app.get_saving_rates(2020).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let value: serde_json::Value = serde_json::from_str(&response.text().await.unwrap()).unwrap();
    assert!(value.is_array());
}

#[sqlx::test(
    migrations = "../db-postgres/migrations",
    fixtures("years", "saving_rates")
)]
async fn get_all_of_year_returns_all_that_is_in_db(pool: PgPool) {
    let app = spawn_app(pool).await;
    Mock::given(path_regex("/transactions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(BodyResp {
            data: TransactionsResp {
                transactions: vec![],
                server_knowledge: 0,
            },
        }))
        .expect(1)
        .mount(&app.ynab_server)
        .await;

    // Act
    let response = app.get_saving_rates(2023).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let value: Vec<SavingRate> = serde_json::from_str(&response.text().await.unwrap()).unwrap();
    assert_eq!(value.len(), 2);
}

#[derive(Debug, Clone, Serialize, Dummy)]
struct CreateBody {
    id: Uuid,
    name: String,
    year: i32,
    savings: SaveSavings,
    employer_contribution: i64,
    employee_contribution: i64,
    mortgage_capital: i64,
    incomes: SaveIncomes,
}

#[derive(Debug, Serialize, Clone, Dummy)]
struct SaveSavings {
    pub category_ids: Vec<Uuid>,
    pub extra_balance: i64,
}

#[derive(Debug, Serialize, Clone, Dummy)]
struct SaveIncomes {
    pub payee_ids: Vec<Uuid>,
    pub extra_balance: i64,
}

#[sqlx::test(migrations = "../db-postgres/migrations", fixtures("years"))]
async fn create_returns_201_for_valid_body(pool: PgPool) {
    let app = spawn_app(pool).await;
    Mock::given(path_regex("/transactions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(BodyResp {
            data: TransactionsResp {
                transactions: vec![],
                server_knowledge: 0,
            },
        }))
        .expect(1)
        .mount(&app.ynab_server)
        .await;

    let body = CreateBody {
        year: 2024,
        ..Faker.fake()
    };
    // Act
    let response = app.create_saving_rate(&body).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::CREATED);
}

#[sqlx::test(migrations = "../db-postgres/migrations")]
async fn create_returns_404_when_year_does_not_exist(pool: PgPool) {
    let app = spawn_app(pool).await;
    Mock::given(path_regex("/transactions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(BodyResp {
            data: TransactionsResp {
                transactions: vec![],
                server_knowledge: 0,
            },
        }))
        .expect(0)
        .mount(&app.ynab_server)
        .await;

    let body = CreateBody {
        year: 2020,
        ..Faker.fake()
    };
    // Act
    let response = app.create_saving_rate(&body).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
}

#[sqlx::test(
    migrations = "../db-postgres/migrations",
    fixtures("years", "saving_rates")
)]
async fn create_returns_409_for_saving_rate_already_present(pool: PgPool) {
    let app = spawn_app(pool).await;
    Mock::given(path_regex("/transactions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(BodyResp {
            data: TransactionsResp {
                transactions: vec![],
                server_knowledge: 0,
            },
        }))
        .expect(0)
        .mount(&app.ynab_server)
        .await;

    let body = CreateBody {
        year: 2023,
        name: "SavingRates_Test1".to_string(),
        ..Faker.fake()
    };
    // Act
    let response = app.create_saving_rate(&body).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::CONFLICT);
}

#[sqlx::test(
    migrations = "../db-postgres/migrations",
    fixtures("years", "saving_rates")
)]
async fn get_returns_404_for_a_non_existing_saving_rate(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    Mock::given(path_regex("/transactions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(BodyResp {
            data: TransactionsResp {
                transactions: vec![],
                server_knowledge: 0,
            },
        }))
        .expect(0)
        .mount(&app.ynab_server)
        .await;

    // Act
    let response = app
        .get_saving_rate("dde265f1-5d0e-4d7c-aceb-74fb388047dc")
        .await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
}

#[sqlx::test(
    migrations = "../db-postgres/migrations",
    fixtures("years", "saving_rates")
)]
async fn get_returns_existing_saving_rate(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    Mock::given(path_regex("/transactions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(BodyResp {
            data: TransactionsResp {
                transactions: vec![],
                server_knowledge: 0,
            },
        }))
        .expect(1)
        .mount(&app.ynab_server)
        .await;

    // Act
    let response = app
        .get_saving_rate("231693be-437e-4d6a-b70e-2d74b23d2f39")
        .await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let value: SavingRate = serde_json::from_str(&response.text().await.unwrap()).unwrap();
    assert_eq!(value.year, 2023);
    assert_eq!(value.name, "SavingRates_Test1");
}

#[derive(Debug, Clone, Serialize, Dummy)]
struct UpdateBody {
    id: Uuid,
    name: String,
    year: i32,
    savings: UpdateSavings,
    employer_contribution: i64,
    employee_contribution: i64,
    mortgage_capital: i64,
    incomes: UpdateIncomes,
}

#[derive(Debug, Serialize, Clone, Dummy)]
struct UpdateSavings {
    pub category_ids: Vec<Uuid>,
    pub extra_balance: i64,
}

#[derive(Debug, Serialize, Clone, Dummy)]
struct UpdateIncomes {
    pub payee_ids: Vec<Uuid>,
    pub extra_balance: i64,
}

#[sqlx::test(
    migrations = "../db-postgres/migrations",
    fixtures("years", "saving_rates")
)]
async fn update_returns_404_for_a_non_existing_saving_rate(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    Mock::given(path_regex("/transactions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(BodyResp {
            data: TransactionsResp {
                transactions: vec![],
                server_knowledge: 0,
            },
        }))
        .expect(0)
        .mount(&app.ynab_server)
        .await;

    let body = UpdateBody {
        id: "5e387420-e64d-4716-bebd-db690361e73d".parse().unwrap(),
        ..Faker.fake()
    };

    // Act
    let response = app
        .update_saving_rate("5e387420-e64d-4716-bebd-db690361e73d", &body)
        .await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
}

#[sqlx::test(
    migrations = "../db-postgres/migrations",
    fixtures("years", "saving_rates")
)]
async fn update_changes_existing_saving_rate(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    Mock::given(path_regex("/transactions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(BodyResp {
            data: TransactionsResp {
                transactions: vec![],
                server_knowledge: 0,
            },
        }))
        .expect(1)
        .mount(&app.ynab_server)
        .await;

    let body = UpdateBody {
        id: "231693be-437e-4d6a-b70e-2d74b23d2f39".parse().unwrap(),
        name: "Updated_name".to_string(),
        year: 2023,
        ..Faker.fake()
    };

    // Act
    let response = app
        .update_saving_rate("231693be-437e-4d6a-b70e-2d74b23d2f39", &body)
        .await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let value: SavingRate = serde_json::from_str(&response.text().await.unwrap()).unwrap();
    assert_eq!(value.name, "Updated_name");
    assert_eq!(value.year, 2023);
}

#[sqlx::test(
    migrations = "../db-postgres/migrations",
    fixtures("years", "saving_rates")
)]
async fn delete_returns_404_for_a_non_existing_saving_rate(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    Mock::given(path_regex("/transactions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(BodyResp {
            data: TransactionsResp {
                transactions: vec![],
                server_knowledge: 0,
            },
        }))
        .expect(0)
        .mount(&app.ynab_server)
        .await;

    // Act
    let response = app
        .delete_saving_rate("011c2732-8564-41fb-a3d0-f3063707558e")
        .await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
}

#[sqlx::test(
    migrations = "../db-postgres/migrations",
    fixtures("years", "saving_rates")
)]
async fn delete_returns_existing_saving_rate(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    Mock::given(path_regex("/transactions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(BodyResp {
            data: TransactionsResp {
                transactions: vec![],
                server_knowledge: 0,
            },
        }))
        .expect(1)
        .mount(&app.ynab_server)
        .await;

    // Act
    let response = app
        .delete_saving_rate("231693be-437e-4d6a-b70e-2d74b23d2f39")
        .await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let value: SavingRate = serde_json::from_str(&response.text().await.unwrap()).unwrap();
    assert_eq!(value.name, "SavingRates_Test1");
    assert_eq!(value.year, 2023);
}

#[derive(Debug, Clone, Serialize)]
struct BodyResp {
    pub data: TransactionsResp,
}

#[derive(Debug, Clone, Serialize)]
struct TransactionsResp {
    pub transactions: Vec<DummyTransaction>,
    pub server_knowledge: i64,
}

#[derive(Debug, Clone, Serialize, Dummy)]
struct DummyTransaction {
    pub id: Uuid,
    pub date: chrono::NaiveDate,
    #[dummy(faker = "-100000..100000")]
    pub amount: i64,
    pub memo: Option<String>,
    pub cleared: DummyClearedType,
    pub approved: bool,
    pub flag_color: Option<String>,
    pub account_id: Uuid,
    pub payee_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub transfer_account_id: Option<Uuid>,
    pub transfer_transaction_id: Option<Uuid>,
    pub matched_transaction_id: Option<Uuid>,
    pub import_id: Option<Uuid>,
    pub import_payee_name: Option<String>,
    pub import_payee_name_original: Option<String>,
    pub debt_transaction_type: Option<DummyDebtTransactionType>,
    pub deleted: bool,
    pub account_name: String,
    pub payee_name: Option<String>,
    pub category_name: Option<String>,
    pub subtransactions: Vec<DummySubTransaction>,
}

#[derive(Debug, Clone, Serialize, Dummy)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
enum DummyClearedType {
    Cleared,
    Uncleared,
    Reconciled,
}

#[derive(Debug, Clone, Serialize, Dummy)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
enum DummyDebtTransactionType {
    Payment,
    Refund,
    Fee,
    Interest,
    Escrow,
    BalanceAdjustment,
    Credit,
    Charge,
}

#[derive(Debug, Clone, Serialize, Dummy)]
struct DummySubTransaction {
    pub id: Uuid,
    pub transaction_id: Uuid,
    #[dummy(faker = "-100000..100000")]
    pub amount: i64,
    pub memo: Option<String>,
    pub payee_id: Option<Uuid>,
    pub payee_name: Option<String>,
    pub category_id: Option<Uuid>,
    pub category_name: Option<String>,
    pub transfer_account_id: Option<Uuid>,
    pub transfer_transaction_id: Option<Uuid>,
    pub deleted: bool,
}
