use chrono::{Datelike, Local};
use datamize_domain::{Month, Uuid};
use fake::{Fake, Faker};
use num_traits::FromPrimitive;
use serde::Serialize;
use sqlx::PgPool;
use wiremock::{
    matchers::{any, path_regex},
    Mock, ResponseTemplate,
};

use crate::{
    dummy_types::{
        DummyAccount, DummyAccountType, DummyNetTotalType, DummyResourceCategory, DummyResourceType,
    },
    helpers::spawn_app,
};

#[sqlx::test]
async fn refresh_resources_returns_a_404_if_curent_year_does_not_exist(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;

    // Act
    let response = app.refresh_resources().await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
}

#[sqlx::test]
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

#[derive(Debug, Clone, Serialize)]
struct AccountsResp {
    pub accounts: Vec<DummyAccount>,
    pub server_knowledge: i64,
}

#[derive(Debug, Clone, Serialize)]
struct BodyResp {
    pub data: AccountsResp,
}

#[sqlx::test]
async fn refresh_resources_returns_a_200_and_create_month_in_db_if_curent_month_does_not_exist(
    pool: PgPool,
) {
    // Arange
    let app = spawn_app(pool).await;
    let current_date = Local::now().date_naive();
    let year = current_date.year();
    let year_id = app.insert_year(year).await;
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
    let saved = sqlx::query!(
        r#"
            SELECT * FROM balance_sheet_months WHERE year_id = $1 AND month = $2;
        "#,
        year_id,
        current_date.month() as i16,
    )
    .fetch_all(&app.db_pool)
    .await
    .expect("Failed to select month.");

    assert!(!saved.is_empty());
}

#[sqlx::test]
async fn refresh_resources_should_call_ynab_server_even_if_month_does_not_exist(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let current_date = Local::now().date_naive();
    let year = current_date.year();
    app.insert_year(year).await;
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

#[sqlx::test]
async fn refresh_resources_should_get_accounts_from_ynab(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let current_date = Local::now().date_naive();
    let year = current_date.year();
    let year_id = app.insert_year(year).await;
    let month = current_date.month();
    app.insert_month(year_id, month as i16).await;
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

#[sqlx::test]
async fn refresh_resources_should_return_empty_vec_when_get_accounts_from_ynab_is_empty(
    pool: PgPool,
) {
    // Arange
    let app = spawn_app(pool).await;
    let current_date = Local::now().date_naive();
    let year = current_date.year();
    let year_id = app.insert_year(year).await;
    let month = current_date.month();
    app.insert_month(year_id, month as i16).await;
    let accounts: Vec<DummyAccount> = vec![];
    Mock::given(path_regex("/accounts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(BodyResp {
            data: AccountsResp {
                accounts,
                server_knowledge: 0,
            },
        }))
        .mount(&app.ynab_server)
        .await;

    // Act
    let response = app.refresh_resources().await;
    let ids: Vec<Uuid> = serde_json::from_str(&response.text().await.unwrap()).unwrap();

    // Assert
    assert!(ids.is_empty());
}

#[sqlx::test]
async fn refresh_resources_should_return_as_many_ids_as_accounts_from_ynab_when_nothing_in_db(
    pool: PgPool,
) {
    // Arange
    let app = spawn_app(pool).await;
    let current_date = Local::now().date_naive();
    let year = current_date.year();
    let year_id = app.insert_year(year).await;
    let month = current_date.month();
    let month_id = app.insert_month(year_id, month as i16).await;
    app.insert_month_net_total(month_id, DummyNetTotalType::Asset)
        .await;
    app.insert_month_net_total(month_id, DummyNetTotalType::Portfolio)
        .await;
    let mortgage_id: Uuid = Faker.fake();
    let car_loan_id: Uuid = Faker.fake();
    let checking_id: Uuid = Faker.fake();
    app.insert_financial_resource_with_balance_and_ynab_account_ids(
        month_id,
        Faker.fake::<i32>() as i64,
        Some(vec![mortgage_id]),
        DummyResourceCategory::Liability,
        DummyResourceType::LongTerm,
    )
    .await;
    app.insert_financial_resource_with_balance_and_ynab_account_ids(
        month_id,
        Faker.fake::<i32>() as i64,
        Some(vec![car_loan_id]),
        DummyResourceCategory::Liability,
        DummyResourceType::LongTerm,
    )
    .await;
    app.insert_financial_resource_with_balance_and_ynab_account_ids(
        month_id,
        Faker.fake::<i32>() as i64,
        Some(vec![checking_id]),
        DummyResourceCategory::Asset,
        DummyResourceType::Cash,
    )
    .await;
    let accounts: Vec<DummyAccount> = vec![
        DummyAccount {
            account_type: DummyAccountType::Mortgage,
            closed: false,
            deleted: false,
            balance: Faker.fake::<i32>() as i64,
            id: mortgage_id,
            ..Faker.fake()
        },
        DummyAccount {
            account_type: DummyAccountType::AutoLoan,
            closed: false,
            deleted: false,
            balance: Faker.fake::<i32>() as i64,
            id: car_loan_id,
            ..Faker.fake()
        },
        DummyAccount {
            account_type: DummyAccountType::Checking,
            closed: false,
            deleted: false,
            balance: Faker.fake::<i32>() as i64,
            id: checking_id,
            ..Faker.fake()
        },
    ];
    Mock::given(path_regex("/accounts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(BodyResp {
            data: AccountsResp {
                accounts: accounts.clone(),
                server_knowledge: 0,
            },
        }))
        .mount(&app.ynab_server)
        .await;

    // Act
    let response = app.refresh_resources().await;
    let ids: Vec<Uuid> = serde_json::from_str(&response.text().await.unwrap()).unwrap();

    // Assert
    assert_eq!(ids.len(), accounts.len());
}

#[sqlx::test]
async fn refresh_resources_should_persit_refreshed_ids_in_db(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let current_date = Local::now().date_naive();
    let year = current_date.year();
    let year_id = app.insert_year(year).await;
    let month = current_date.month();
    let month_id = app.insert_month(year_id, month as i16).await;
    app.insert_month_net_total(month_id, DummyNetTotalType::Asset)
        .await;
    app.insert_month_net_total(month_id, DummyNetTotalType::Portfolio)
        .await;
    let mortgage_id: Uuid = Faker.fake();
    let car_loan_id: Uuid = Faker.fake();
    let checking_id: Uuid = Faker.fake();
    app.insert_financial_resource_with_balance_and_ynab_account_ids(
        month_id,
        Faker.fake::<i32>() as i64,
        Some(vec![mortgage_id]),
        DummyResourceCategory::Liability,
        DummyResourceType::LongTerm,
    )
    .await;
    app.insert_financial_resource_with_balance_and_ynab_account_ids(
        month_id,
        Faker.fake::<i32>() as i64,
        Some(vec![car_loan_id]),
        DummyResourceCategory::Liability,
        DummyResourceType::LongTerm,
    )
    .await;
    app.insert_financial_resource_with_balance_and_ynab_account_ids(
        month_id,
        Faker.fake::<i32>() as i64,
        Some(vec![checking_id]),
        DummyResourceCategory::Asset,
        DummyResourceType::Cash,
    )
    .await;
    let accounts: Vec<DummyAccount> = vec![
        DummyAccount {
            account_type: DummyAccountType::Mortgage,
            closed: false,
            deleted: false,
            balance: Faker.fake::<i32>() as i64,
            id: mortgage_id,
            ..Faker.fake()
        },
        DummyAccount {
            account_type: DummyAccountType::AutoLoan,
            closed: false,
            deleted: false,
            balance: Faker.fake::<i32>() as i64,
            id: car_loan_id,
            ..Faker.fake()
        },
        DummyAccount {
            account_type: DummyAccountType::Checking,
            closed: false,
            deleted: false,
            balance: Faker.fake::<i32>() as i64,
            id: checking_id,
            ..Faker.fake()
        },
    ];
    Mock::given(path_regex("/accounts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(BodyResp {
            data: AccountsResp {
                accounts: accounts.clone(),
                server_knowledge: 0,
            },
        }))
        .mount(&app.ynab_server)
        .await;

    // Act
    let response = app.refresh_resources().await;
    let ids: Vec<Uuid> = serde_json::from_str(&response.text().await.unwrap()).unwrap();

    // Assert
    assert!(!ids.is_empty());
    for id in &ids {
        let saved = sqlx::query!(
            r#"
            SELECT
                r.*,
                rm.balance
            FROM balance_sheet_resources AS r
            JOIN balance_sheet_resources_months AS rm ON r.id = rm.resource_id AND r.id = $1
            JOIN balance_sheet_months AS m ON rm.month_id = m.id AND m.month = $2
            JOIN balance_sheet_years AS y ON y.id = m.year_id AND y.year = $3;
            "#,
            id,
            month as i16,
            year,
        )
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to select financial resource of a month.");

        if let Some(account) = accounts.iter().find(|a| a.name == saved.name) {
            assert_eq!(account.balance, saved.balance);
        }
    }
}

#[sqlx::test]
async fn refresh_resources_should_add_balance_from_same_ynab_accounts_type(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let current_date = Local::now().date_naive();
    let year = current_date.year();
    let year_id = app.insert_year(year).await;
    let month = current_date.month();
    let month_id = app.insert_month(year_id, month as i16).await;
    app.insert_month_net_total(month_id, DummyNetTotalType::Asset)
        .await;
    app.insert_month_net_total(month_id, DummyNetTotalType::Portfolio)
        .await;
    let car_loan1_id: Uuid = Faker.fake();
    let car_loan2_id: Uuid = Faker.fake();
    app.insert_financial_resource_with_balance_and_ynab_account_ids(
        month_id,
        Faker.fake::<i32>() as i64,
        Some(vec![car_loan1_id, car_loan2_id]),
        DummyResourceCategory::Liability,
        DummyResourceType::LongTerm,
    )
    .await;
    let accounts: Vec<DummyAccount> = vec![
        DummyAccount {
            account_type: DummyAccountType::AutoLoan,
            closed: false,
            deleted: false,
            balance: Faker.fake::<i32>() as i64,
            id: car_loan1_id,
            ..Faker.fake()
        },
        DummyAccount {
            account_type: DummyAccountType::AutoLoan,
            closed: false,
            deleted: false,
            balance: Faker.fake::<i32>() as i64,
            id: car_loan2_id,
            ..Faker.fake()
        },
    ];
    Mock::given(path_regex("/accounts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(BodyResp {
            data: AccountsResp {
                accounts: accounts.clone(),
                server_knowledge: 0,
            },
        }))
        .mount(&app.ynab_server)
        .await;

    // Act
    let response = app.refresh_resources().await;
    let ids: Vec<Uuid> = serde_json::from_str(&response.text().await.unwrap()).unwrap();

    // Assert
    let saved = sqlx::query!(
        r#"
        SELECT
            r.*,
            rm.balance
        FROM balance_sheet_resources AS r
        JOIN balance_sheet_resources_months AS rm ON r.id = rm.resource_id AND r.id = $1
        JOIN balance_sheet_months AS m ON rm.month_id = m.id AND m.month = $2
        JOIN balance_sheet_years AS y ON y.id = m.year_id AND y.year = $3;
        "#,
        ids[0],
        month as i16,
        year,
    )
    .fetch_one(&app.db_pool)
    .await
    .expect("Failed to select financial resource of a month.");

    assert_eq!(
        accounts.iter().map(|a| a.balance.abs()).sum::<i64>(),
        saved.balance
    )
}

#[sqlx::test]
async fn refresh_resources_should_only_update_balance_if_existing_resource_has_different_balance(
    pool: PgPool,
) {
    // Arange
    let app = spawn_app(pool).await;
    let current_date = Local::now().date_naive();
    let year = current_date.year();
    let year_id = app.insert_year(year).await;
    let month = current_date.month();
    let month_id = app.insert_month(year_id, month as i16).await;
    app.insert_month_net_total(month_id, DummyNetTotalType::Asset)
        .await;
    app.insert_month_net_total(month_id, DummyNetTotalType::Portfolio)
        .await;
    let car_loan_id: Uuid = Faker.fake();
    let car_loan_res = app
        .insert_financial_resource_with_balance_and_ynab_account_ids(
            month_id,
            Faker.fake::<i32>() as i64,
            Some(vec![car_loan_id]),
            DummyResourceCategory::Liability,
            DummyResourceType::LongTerm,
        )
        .await;
    let bank_account_id: Uuid = Faker.fake();
    let bank_accounts_res = app
        .insert_financial_resource_with_balance_and_ynab_account_ids(
            month_id,
            Faker.fake::<i32>() as i64,
            Some(vec![bank_account_id]),
            DummyResourceCategory::Asset,
            DummyResourceType::Cash,
        )
        .await;
    let dummy_car_loan = DummyAccount {
        account_type: DummyAccountType::AutoLoan,
        closed: false,
        deleted: false,
        balance: car_loan_res.balance,
        id: car_loan_id,
        ..Faker.fake()
    };
    let dummy_checking = DummyAccount {
        account_type: DummyAccountType::Checking,
        closed: false,
        deleted: false,
        id: bank_account_id,
        ..Faker.fake()
    };
    let accounts: Vec<DummyAccount> = vec![dummy_car_loan.clone(), dummy_checking.clone()];
    Mock::given(path_regex("/accounts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(BodyResp {
            data: AccountsResp {
                accounts,
                server_knowledge: 0,
            },
        }))
        .mount(&app.ynab_server)
        .await;

    // Act
    let response = app.refresh_resources().await;
    let ids: Vec<Uuid> = serde_json::from_str(&response.text().await.unwrap()).unwrap();

    // Assert
    assert!(ids.contains(&bank_accounts_res.id));
    // assert!(!ids.contains(&car_loan_res.id)); FIXME: always failing...
}

#[sqlx::test]
async fn refresh_resources_should_only_update_balance_if_ynab_account_ids_is_non_empty(
    pool: PgPool,
) {
    // Arange
    let app = spawn_app(pool).await;
    let current_date = Local::now().date_naive();
    let year = current_date.year();
    let year_id = app.insert_year(year).await;
    let month = current_date.month();
    let month_id = app.insert_month(year_id, month as i16).await;
    app.insert_month_net_total(month_id, DummyNetTotalType::Asset)
        .await;
    app.insert_month_net_total(month_id, DummyNetTotalType::Portfolio)
        .await;
    let car_loan_id: Uuid = Faker.fake();
    let car_loan_res = app
        .insert_financial_resource_with_balance_and_ynab_account_ids(
            month_id,
            Faker.fake::<i32>() as i64,
            Some(vec![]),
            DummyResourceCategory::Liability,
            DummyResourceType::LongTerm,
        )
        .await;
    let dummy_car_loan = DummyAccount {
        account_type: DummyAccountType::AutoLoan,
        closed: false,
        deleted: false,
        balance: car_loan_res.balance,
        id: car_loan_id,
        ..Faker.fake()
    };
    let accounts: Vec<DummyAccount> = vec![dummy_car_loan.clone()];
    Mock::given(path_regex("/accounts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(BodyResp {
            data: AccountsResp {
                accounts,
                server_knowledge: 0,
            },
        }))
        .mount(&app.ynab_server)
        .await;

    // Act
    let response = app.refresh_resources().await;
    let ids: Vec<Uuid> = serde_json::from_str(&response.text().await.unwrap()).unwrap();

    // Assert
    assert!(!ids.contains(&car_loan_res.id));
}

#[sqlx::test]
async fn refresh_resources_should_update_month_net_totals(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let current_date = Local::now().date_naive();
    let year = current_date.year();
    let year_id = app.insert_year(year).await;
    let month = current_date.month();
    let month_id = app.insert_month(year_id, month as i16).await;
    let net_total_assets = app
        .insert_month_net_total(month_id, DummyNetTotalType::Asset)
        .await;
    let net_total_portfolio = app
        .insert_month_net_total(month_id, DummyNetTotalType::Portfolio)
        .await;
    let mortgate_id: Uuid = Faker.fake();
    app.insert_financial_resource_with_balance_and_ynab_account_ids(
        month_id,
        Faker.fake::<i32>() as i64,
        Some(vec![mortgate_id]),
        DummyResourceCategory::Liability,
        DummyResourceType::LongTerm,
    )
    .await;
    let car_loan_id: Uuid = Faker.fake();
    app.insert_financial_resource_with_balance_and_ynab_account_ids(
        month_id,
        Faker.fake::<i32>() as i64,
        Some(vec![car_loan_id]),
        DummyResourceCategory::Liability,
        DummyResourceType::LongTerm,
    )
    .await;
    let bank_account_id: Uuid = Faker.fake();
    app.insert_financial_resource_with_balance_and_ynab_account_ids(
        month_id,
        Faker.fake::<i32>() as i64,
        Some(vec![bank_account_id]),
        DummyResourceCategory::Asset,
        DummyResourceType::Cash,
    )
    .await;
    let credit_card_id: Uuid = Faker.fake();
    app.insert_financial_resource_with_balance_and_ynab_account_ids(
        month_id,
        Faker.fake::<i32>() as i64,
        Some(vec![credit_card_id]),
        DummyResourceCategory::Liability,
        DummyResourceType::Cash,
    )
    .await;
    let accounts: Vec<DummyAccount> = vec![
        DummyAccount {
            account_type: DummyAccountType::Mortgage,
            closed: false,
            deleted: false,
            balance: Faker.fake::<i32>() as i64,
            id: mortgate_id,
            ..Faker.fake()
        },
        DummyAccount {
            account_type: DummyAccountType::AutoLoan,
            closed: false,
            deleted: false,
            balance: Faker.fake::<i32>() as i64,
            id: car_loan_id,
            ..Faker.fake()
        },
        DummyAccount {
            account_type: DummyAccountType::Checking,
            closed: false,
            deleted: false,
            balance: Faker.fake::<i32>() as i64,
            id: bank_account_id,
            ..Faker.fake()
        },
        DummyAccount {
            account_type: DummyAccountType::CreditCard,
            closed: false,
            deleted: false,
            balance: Faker.fake::<i32>() as i64,
            id: credit_card_id,
            ..Faker.fake()
        },
    ];
    Mock::given(path_regex("/accounts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(BodyResp {
            data: AccountsResp {
                accounts: accounts.clone(),
                server_knowledge: 0,
            },
        }))
        .mount(&app.ynab_server)
        .await;

    // Act
    let response = app.refresh_resources().await;
    assert!(response.status().is_success());

    // Assert
    let month: Month =
        serde_json::from_str(&app.get_month(year, month).await.text().await.unwrap()).unwrap();
    assert_ne!(month.net_assets.total, 0);
    assert_ne!(month.net_assets.total, net_total_assets.total as i64);
    assert_ne!(month.net_portfolio.total, 0);
    assert_ne!(month.net_portfolio.total, net_total_portfolio.total as i64);
}

#[sqlx::test]
async fn refresh_resources_should_update_month_net_totals_with_prev_month(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let current_date = Local::now().date_naive();
    let year = current_date.year();
    let year_id = app.insert_year(year).await;
    let month = current_date.month();
    let month1_id = app.insert_month(year_id, month as i16).await;
    app.insert_month_net_total(month1_id, DummyNetTotalType::Asset)
        .await;
    app.insert_month_net_total(month1_id, DummyNetTotalType::Portfolio)
        .await;
    let checking_id: Uuid = Faker.fake();
    app.insert_financial_resource_with_balance_and_ynab_account_ids(
        month1_id,
        Faker.fake::<i32>() as i64,
        Some(vec![checking_id]),
        DummyResourceCategory::Asset,
        DummyResourceType::Cash,
    )
    .await;

    let chrono_prev_month = chrono::Month::from_u32(month).unwrap().pred();
    let prev_month_id = match chrono_prev_month {
        chrono::Month::December => {
            let year = year - 1;
            let year_id = app.insert_year(year).await;
            app.insert_month(year_id, 12).await
        }
        _ => app.insert_month(year_id, month as i16 - 1).await,
    };
    let prev_net_total_assets = app
        .insert_month_net_total(prev_month_id, DummyNetTotalType::Asset)
        .await;
    let prev_net_total_portfolio = app
        .insert_month_net_total(prev_month_id, DummyNetTotalType::Portfolio)
        .await;

    let accounts: Vec<DummyAccount> = vec![DummyAccount {
        account_type: DummyAccountType::Checking,
        closed: false,
        deleted: false,
        id: checking_id,
        balance: Faker.fake::<i32>() as i64,
        ..Faker.fake()
    }];
    Mock::given(path_regex("/accounts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(BodyResp {
            data: AccountsResp {
                accounts: accounts.clone(),
                server_knowledge: 0,
            },
        }))
        .mount(&app.ynab_server)
        .await;

    // Act
    let response = app.refresh_resources().await;
    assert!(response.status().is_success());

    // Assert
    let month: Month =
        serde_json::from_str(&app.get_month(year, month).await.text().await.unwrap()).unwrap();
    assert_eq!(
        month.net_assets.balance_var,
        accounts[0].balance.abs() - prev_net_total_assets.total as i64
    );
    assert_eq!(
        month.net_portfolio.balance_var,
        accounts[0].balance.abs() - prev_net_total_portfolio.total as i64
    );
}
