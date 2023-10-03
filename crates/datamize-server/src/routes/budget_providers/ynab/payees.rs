use axum::{extract::State, Json};
use ynab::Payee;

use crate::{error::HttpJsonDatamizeResult, services::budget_providers::DynYnabPayeeService};

/// Returns all accounts from YNAB's API.
#[tracing::instrument(name = "Get all payees from YNAB's API", skip_all)]
pub async fn get_ynab_payees(
    State(mut ynab_payee_service): State<DynYnabPayeeService>,
) -> HttpJsonDatamizeResult<Vec<Payee>> {
    Ok(Json(ynab_payee_service.get_all_ynab_payees().await?))
}

#[cfg(test)]
mod tests {
    use crate::{
        error::{AppError, DatamizeResult},
        routes::budget_providers::ynab::get_ynab_payee_routes,
        services::budget_providers::YnabPayeeServiceExt,
    };

    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use datamize_domain::async_trait;
    use fake::{Fake, Faker};
    use tower::ServiceExt; // for `oneshot` and `ready`

    #[tokio::test]
    async fn get_ynab_payees_success() {
        let payees = vec![
            Payee {
                id: Faker.fake(),
                name: Faker.fake(),
                transfer_account_id: Faker.fake(),
                deleted: false,
            },
            Payee {
                id: Faker.fake(),
                name: Faker.fake(),
                transfer_account_id: Faker.fake(),
                deleted: false,
            },
        ];

        #[derive(Clone)]
        struct MockYnabPayeeService {
            payees: Vec<Payee>,
        }
        #[async_trait]
        impl YnabPayeeServiceExt for MockYnabPayeeService {
            async fn get_all_ynab_payees(&mut self) -> DatamizeResult<Vec<Payee>> {
                Ok(self.payees.clone())
            }
        }
        let ynab_payee_service = Box::new(MockYnabPayeeService {
            payees: payees.clone(),
        });

        let app = get_ynab_payee_routes(ynab_payee_service);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/payees")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: Vec<Payee> = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, payees);
    }

    #[tokio::test]
    async fn get_ynab_payees_error_500() {
        #[derive(Clone)]
        struct MockYnabPayeeService {}
        #[async_trait]
        impl YnabPayeeServiceExt for MockYnabPayeeService {
            async fn get_all_ynab_payees(&mut self) -> DatamizeResult<Vec<Payee>> {
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
        let ynab_payee_service = Box::new(MockYnabPayeeService {});

        let app = get_ynab_payee_routes(ynab_payee_service);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/payees")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
