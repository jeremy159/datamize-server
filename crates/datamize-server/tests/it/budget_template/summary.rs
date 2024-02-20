use pretty_assertions::assert_eq;
use sqlx::PgPool;
use wiremock::{matchers::path_regex, Mock, ResponseTemplate};

use crate::{
    budget_template::{
        BodyResp, CategoriesResp, DummyCategoryGroupWithCategories,
        DummyScheduledTransactionDetail, ScheduledTransactionsResp,
    },
    helpers::spawn_app,
};

#[sqlx::test(migrations = "../db-postgres/migrations")]
async fn get_returns_200_when_nothing_in_db(pool: PgPool) {
    let app = spawn_app(pool).await;
    let scheduled_trans_mock = Mock::given(path_regex("/scheduled_transactions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(BodyResp {
            data: ScheduledTransactionsResp {
                scheduled_transactions: fake::vec![DummyScheduledTransactionDetail; 0..10],
                server_knowledge: 0,
            },
        }))
        .expect(1);
    app.ynab_server.register(scheduled_trans_mock).await;

    let categories_mock = Mock::given(path_regex("/categories"))
        .respond_with(ResponseTemplate::new(200).set_body_json(BodyResp {
            data: CategoriesResp {
                category_groups: fake::vec![DummyCategoryGroupWithCategories; 0..10],
                server_knowledge: 0,
            },
        }))
        .expect(1);
    app.ynab_server.register(categories_mock).await;

    // Act
    let response = app.get_template_summary().await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::OK);
}

#[sqlx::test(
    migrations = "../db-postgres/migrations",
    fixtures(
        "budgeters",
        "expenses_categorization",
        "categories",
        "scheduled_transactions"
    )
)]
async fn get_returns_200_with_what_is_in_db(pool: PgPool) {
    let app = spawn_app(pool).await;
    let scheduled_trans_mock = Mock::given(path_regex("/scheduled_transactions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(BodyResp {
            data: ScheduledTransactionsResp {
                scheduled_transactions: fake::vec![DummyScheduledTransactionDetail; 0..10],
                server_knowledge: 0,
            },
        }))
        .expect(1);
    app.ynab_server.register(scheduled_trans_mock).await;

    let categories_mock = Mock::given(path_regex("/categories"))
        .respond_with(ResponseTemplate::new(200).set_body_json(BodyResp {
            data: CategoriesResp {
                category_groups: fake::vec![DummyCategoryGroupWithCategories; 0..10],
                server_knowledge: 0,
            },
        }))
        .expect(1);
    app.ynab_server.register(categories_mock).await;

    // Act
    let response = app.get_template_summary().await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::OK);
}
