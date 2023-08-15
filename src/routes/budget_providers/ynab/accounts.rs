use axum::{extract::State, Json};
use ynab::types::Account;

use crate::{error::HttpJsonDatamizeResult, services::budget_providers::DynYnabAccountService};

/// Returns all accounts from YNAB's API.
#[tracing::instrument(name = "Get all accounts from YNAB's API", skip_all)]
pub async fn get_ynab_accounts(
    State(mut ynab_account_service): State<DynYnabAccountService>,
) -> HttpJsonDatamizeResult<Vec<Account>> {
    Ok(Json(ynab_account_service.get_all_ynab_accounts().await?))
}

#[cfg(test)]
mod tests {
    use crate::{
        error::{AppError, DatamizeResult},
        routes::budget_providers::ynab::get_ynab_account_routes,
        services::budget_providers::YnabAccountServiceExt,
    };

    use super::*;
    use async_trait::async_trait;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use fake::{Fake, Faker};
    use tower::ServiceExt; // for `oneshot` and `ready`
    use ynab::AccountType;

    #[tokio::test]
    async fn get_ynab_accounts_success() {
        let accounts = vec![
            Account {
                id: Faker.fake(),
                name: Faker.fake(),
                account_type: AccountType::Cash,
                on_budget: Faker.fake(),
                closed: Faker.fake(),
                note: Faker.fake(),
                balance: Faker.fake(),
                cleared_balance: Faker.fake(),
                uncleared_balance: Faker.fake(),
                transfer_payee_id: Faker.fake(),
                direct_import_linked: Faker.fake(),
                direct_import_in_error: Faker.fake(),
                deleted: false,
            },
            Account {
                id: Faker.fake(),
                name: Faker.fake(),
                account_type: AccountType::Cash,
                on_budget: Faker.fake(),
                closed: Faker.fake(),
                note: Faker.fake(),
                balance: Faker.fake(),
                cleared_balance: Faker.fake(),
                uncleared_balance: Faker.fake(),
                transfer_payee_id: Faker.fake(),
                direct_import_linked: Faker.fake(),
                direct_import_in_error: Faker.fake(),
                deleted: false,
            },
        ];

        #[derive(Clone)]
        struct MockYnabAccountService {
            accounts: Vec<Account>,
        }
        #[async_trait]
        impl YnabAccountServiceExt for MockYnabAccountService {
            async fn get_all_ynab_accounts(&mut self) -> DatamizeResult<Vec<Account>> {
                Ok(self.accounts.clone())
            }
        }
        let ynab_account_service = Box::new(MockYnabAccountService {
            accounts: accounts.clone(),
        });

        let app = get_ynab_account_routes(ynab_account_service);

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/accounts")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: Vec<Account> = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, accounts);
    }

    #[tokio::test]
    async fn get_ynab_accounts_error_500() {
        #[derive(Clone)]
        struct MockYnabAccountService {}
        #[async_trait]
        impl YnabAccountServiceExt for MockYnabAccountService {
            async fn get_all_ynab_accounts(&mut self) -> DatamizeResult<Vec<Account>> {
                Err(AppError::InternalServerError(
                    ynab::Error::Api(ynab::ApiError {
                        id: Faker.fake(),
                        name: Faker.fake(),
                        detail: Faker.fake(),
                    })
                    .into(),
                ))
            }
        }
        let ynab_account_service = Box::new(MockYnabAccountService {});

        let app = get_ynab_account_routes(ynab_account_service);

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/accounts")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
