use axum::{extract::State, Json};
use datamize_domain::ScheduledTransactionsDistribution;

use crate::{
    error::HttpJsonDatamizeResult, services::budget_template::DynTemplateTransactionService,
};

/// Returns a budget template transactions, i.e. all the scheduled transactions in the upcoming 30 days.
pub async fn template_transactions(
    State(mut template_transaction_service): State<DynTemplateTransactionService>,
) -> HttpJsonDatamizeResult<ScheduledTransactionsDistribution> {
    Ok(Json(
        template_transaction_service
            .get_template_transactions()
            .await?,
    ))
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use datamize_domain::async_trait;
    use fake::{Fake, Faker};
    use tower::ServiceExt;

    use crate::{
        error::{AppError, DatamizeResult},
        routes::api::budget_template::get_transaction_routes,
        services::budget_template::TemplateTransactionServiceExt,
    };

    use super::*;

    #[tokio::test]
    async fn get_template_transactions_success() {
        #[derive(Clone)]
        struct MockTemplateTransactionService {}
        #[async_trait]
        impl TemplateTransactionServiceExt for MockTemplateTransactionService {
            async fn get_template_transactions(
                &mut self,
            ) -> DatamizeResult<ScheduledTransactionsDistribution> {
                Ok(ScheduledTransactionsDistribution::default())
            }
        }
        let template_transaction_service = Box::new(MockTemplateTransactionService {});

        let app = get_transaction_routes(template_transaction_service);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/transactions")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: ScheduledTransactionsDistribution = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, ScheduledTransactionsDistribution::default());
    }

    #[tokio::test]
    async fn get_template_transactions_error_500() {
        #[derive(Clone)]
        struct MockTemplateTransactionService {}
        #[async_trait]
        impl TemplateTransactionServiceExt for MockTemplateTransactionService {
            async fn get_template_transactions(
                &mut self,
            ) -> DatamizeResult<ScheduledTransactionsDistribution> {
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
        let template_transaction_service = Box::new(MockTemplateTransactionService {});

        let app = get_transaction_routes(template_transaction_service);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/transactions")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
