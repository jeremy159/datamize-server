use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use datamize_domain::{
    BudgetDetails, BudgeterConfig, DatamizeScheduledTransaction, ExpenseCategorization,
    ExternalExpense,
};
use fake::{Fake, Faker};
use pretty_assertions::assert_eq;
use sqlx::SqlitePool;
use tower::ServiceExt;
use ynab::{
    Category, CategoryGroupWithCategories, CategoryGroupWithCategoriesDelta,
    ScheduledTransactionDetail, ScheduledTransactionsDetailDelta,
};

use crate::routes::api::budget_template::tests::details::testutils::TestContext;

struct YnabData(
    CategoryGroupWithCategoriesDelta,
    ScheduledTransactionsDetailDelta,
);

struct DbData(
    Vec<ExternalExpense>,
    Vec<BudgeterConfig>,
    Vec<ExpenseCategorization>,
);

async fn check_get(
    pool: SqlitePool,
    month_query: Option<&str>,
    ynab_data: YnabData,
    db_data: Option<DbData>,
    expected_status: StatusCode,
    expected_resp: Option<BudgetDetails>,
) {
    let context = TestContext::setup(pool, ynab_data.0, ynab_data.1);

    if let Some(DbData(external_expenses, budgeters_config, expenses_categorization)) = db_data {
        context.set_external_expenses(&external_expenses).await;
        context.set_budgeters(&budgeters_config).await;
        context
            .set_expenses_categorization(&expenses_categorization)
            .await;
    }

    let uri = match month_query {
        Some(month) => format!("/details?month={:?}", month),
        None => String::from("/details"),
    };

    let response = context
        .into_app()
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), expected_status);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    println!("{body:?}");
    // Asserts that the body is returning something valid and parseable.
    let body: BudgetDetails = serde_json::from_slice(&body).unwrap();

    if let Some(expected) = expected_resp {
        assert_eq!(body, expected);
    }
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_200_and_default_struct_when_nothing_in_db(pool: SqlitePool) {
    check_get(
        pool,
        None,
        YnabData(Faker.fake(), Faker.fake()),
        None,
        StatusCode::OK,
        Some(BudgetDetails::default()),
    )
    .await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_success_with_what_is_in_db(pool: SqlitePool) {
    let ynab_categories = CategoryGroupWithCategoriesDelta {
        category_groups: vec![
            CategoryGroupWithCategories {
                categories: fake::vec![Category; 3..9],
                ..Faker.fake()
            },
            CategoryGroupWithCategories {
                categories: fake::vec![Category; 0..9],
                ..Faker.fake()
            },
            CategoryGroupWithCategories {
                categories: fake::vec![Category; 3..9],
                ..Faker.fake()
            },
            CategoryGroupWithCategories {
                categories: fake::vec![Category; 0..9],
                ..Faker.fake()
            },
            CategoryGroupWithCategories {
                categories: fake::vec![Category; 3..9],
                ..Faker.fake()
            },
        ],
        ..Faker.fake()
    };
    let ynab_scheduled_transactions = ScheduledTransactionsDetailDelta {
        scheduled_transactions: fake::vec![ScheduledTransactionDetail; 1..10],
        ..Faker.fake()
    };
    let mut scheduled_transactions: Vec<DatamizeScheduledTransaction> = ynab_scheduled_transactions
        .clone()
        .scheduled_transactions
        .into_iter()
        .map(|st| st.into())
        .collect();
    let external_expenses = fake::vec![ExternalExpense; 1..3];
    let expenses_categorization = fake::vec![ExpenseCategorization; 4];
    let expenses_categorization = ynab_categories
        .clone()
        .category_groups
        .into_iter()
        .zip(expenses_categorization)
        .map(|(cg, ec)| ExpenseCategorization { id: cg.id, ..ec })
        .collect();

    if scheduled_transactions[0].payee_id.is_none() {
        scheduled_transactions[0].payee_id = Some(Faker.fake());
    }
    let budgeter_with_salary = BudgeterConfig {
        payee_ids: vec![scheduled_transactions[0].payee_id.unwrap()],
        ..Faker.fake()
    };
    let mut budgeters_config = fake::vec![BudgeterConfig; 1..3];
    budgeters_config.push(budgeter_with_salary);
    check_get(
        pool,
        None,
        YnabData(ynab_categories, ynab_scheduled_transactions),
        Some(DbData(
            external_expenses,
            budgeters_config,
            expenses_categorization,
        )),
        StatusCode::OK,
        None, // We don't really care what's the answer, as long as it is able to parse it
    )
    .await;
}
