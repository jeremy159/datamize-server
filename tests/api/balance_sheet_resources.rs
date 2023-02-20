use chrono::{Datelike, Local};
use datamize::domain::{Month, NetTotalType};
use fake::{Fake, Faker};
use num_traits::FromPrimitive;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;
use wiremock::{
    matchers::{any, path_regex},
    Mock, ResponseTemplate,
};

use crate::{
    dummy_types::{DummyAccount, DummyAccountType, DummyNetTotalType},
    helpers::spawn_app,
};

#[sqlx::test]
async fn post_resources_returns_a_404_if_curent_year_does_not_exist(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;

    // Act
    let response = app.refresh_resources().await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn post_resources_should_not_call_ynab_server_if_year_does_not_exist(pool: PgPool) {
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
async fn post_resources_returns_a_200_and_create_month_in_db_if_curent_month_does_not_exist(
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
async fn post_resources_should_call_ynab_server_even_if_month_does_not_exist(pool: PgPool) {
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
async fn post_resources_should_get_accounts_from_ynab(pool: PgPool) {
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
async fn post_resources_should_return_empty_vec_when_get_accounts_from_ynab_is_empty(pool: PgPool) {
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
async fn post_resources_should_return_as_many_ids_as_accounts_from_ynab_when_nothing_in_db(
    pool: PgPool,
) {
    // Arange
    let app = spawn_app(pool).await;
    let current_date = Local::now().date_naive();
    let year = current_date.year();
    let year_id = app.insert_year(year).await;
    let month = current_date.month();
    app.insert_month(year_id, month as i16).await;
    let accounts: Vec<DummyAccount> = vec![
        DummyAccount {
            account_type: DummyAccountType::Mortgage,
            closed: false,
            deleted: false,
            ..Faker.fake()
        },
        DummyAccount {
            account_type: DummyAccountType::AutoLoan,
            closed: false,
            deleted: false,
            ..Faker.fake()
        },
        DummyAccount {
            account_type: DummyAccountType::Checking,
            closed: false,
            deleted: false,
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
async fn post_resources_should_persit_refreshed_ids_in_db(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let current_date = Local::now().date_naive();
    let year = current_date.year();
    let year_id = app.insert_year(year).await;
    let month = current_date.month();
    app.insert_month(year_id, month as i16).await;
    let accounts: Vec<DummyAccount> = vec![
        DummyAccount {
            account_type: DummyAccountType::Mortgage,
            closed: false,
            deleted: false,
            ..Faker.fake()
        },
        DummyAccount {
            account_type: DummyAccountType::AutoLoan,
            closed: false,
            deleted: false,
            ..Faker.fake()
        },
        DummyAccount {
            account_type: DummyAccountType::Checking,
            closed: false,
            deleted: false,
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
                SELECT * FROM balance_sheet_resources WHERE id = $1;
                "#,
            id,
        )
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to select financial resource of a month.");

        if let Some(account) = accounts.iter().find(|a| a.name == saved.name) {
            assert_eq!(account.balance as i64, saved.balance);
        }
    }
}

#[sqlx::test]
async fn post_resources_should_add_balance_from_same_ynab_accounts(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let current_date = Local::now().date_naive();
    let year = current_date.year();
    let year_id = app.insert_year(year).await;
    let month = current_date.month();
    app.insert_month(year_id, month as i16).await;
    let accounts: Vec<DummyAccount> = vec![
        DummyAccount {
            account_type: DummyAccountType::AutoLoan,
            closed: false,
            deleted: false,
            ..Faker.fake()
        },
        DummyAccount {
            account_type: DummyAccountType::AutoLoan,
            closed: false,
            deleted: false,
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
            SELECT * FROM balance_sheet_resources WHERE id = $1;
            "#,
        ids[0],
    )
    .fetch_one(&app.db_pool)
    .await
    .expect("Failed to select financial resource of a month.");

    assert_eq!(
        accounts.iter().map(|a| a.balance as i64).sum::<i64>(),
        saved.balance
    )
}

#[sqlx::test]
async fn post_resources_should_update_month_net_totals(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let current_date = Local::now().date_naive();
    let year = current_date.year();
    let year_id = app.insert_year(year).await;
    let month = current_date.month();
    app.insert_month(year_id, month as i16).await;
    let accounts: Vec<DummyAccount> = vec![
        DummyAccount {
            account_type: DummyAccountType::Mortgage,
            closed: false,
            deleted: false,
            ..Faker.fake()
        },
        DummyAccount {
            account_type: DummyAccountType::AutoLoan,
            closed: false,
            deleted: false,
            ..Faker.fake()
        },
        DummyAccount {
            account_type: DummyAccountType::Checking,
            closed: false,
            deleted: false,
            ..Faker.fake()
        },
        DummyAccount {
            account_type: DummyAccountType::CreditCard,
            closed: false,
            deleted: false,
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
    for nt in &month.net_totals {
        assert_ne!(nt.total, 0);
    }
}

#[sqlx::test]
async fn post_resources_should_update_month_net_totals_with_prev_month(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let current_date = Local::now().date_naive();
    let year = current_date.year();
    let year_id = app.insert_year(year).await;
    let month = current_date.month();
    app.insert_month(year_id, month as i16).await;

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
    for nt in &month.net_totals {
        if nt.net_type == NetTotalType::Asset {
            assert_ne!(
                nt.balance_var,
                (accounts[0].balance - prev_net_total_assets.total) as i64
            );
        } else if nt.net_type == NetTotalType::Portfolio {
            assert_ne!(
                nt.balance_var,
                (accounts[0].balance - prev_net_total_portfolio.total) as i64
            );
        }
    }
}