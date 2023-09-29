use crate::{
    models::budget_template::{BudgetDetails, TemplateParams},
    services::budget_template::DynTemplateDetailService,
};
use axum::{
    extract::{Query, State},
    Json,
};

use crate::error::HttpJsonDatamizeResult;

/// Returns a budget template details
/// Can specify the month to get details from.
/// /template/details?month=previous
/// Possible values to pass in query params are `previous` and `next`. If nothing is specified,
/// the current month will be used.
pub async fn template_details(
    State(mut template_detail_service): State<DynTemplateDetailService>,
    template_params: Query<TemplateParams>,
) -> HttpJsonDatamizeResult<BudgetDetails> {
    let month = template_params.month.unwrap_or_default();

    Ok(Json(
        template_detail_service.get_template_details(month).await?,
    ))
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use fake::{Fake, Faker};
    use tower::ServiceExt;

    use crate::{
        error::{AppError, DatamizeResult},
        models::budget_template::MonthTarget,
        routes::api::budget_template::get_detail_routes,
        services::budget_template::TemplateDetailServiceExt,
    };

    use super::*;

    #[tokio::test]
    async fn get_template_details_success_with_no_query_params() {
        #[derive(Clone)]
        struct MockTemplateDetailService {}
        #[async_trait]
        impl TemplateDetailServiceExt for MockTemplateDetailService {
            async fn get_template_details(
                &mut self,
                month: MonthTarget,
            ) -> DatamizeResult<BudgetDetails> {
                assert_eq!(month, MonthTarget::Current);
                Ok(BudgetDetails::default())
            }
        }
        let template_detail_service = Box::new(MockTemplateDetailService {});

        let app = get_detail_routes(template_detail_service);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/details")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: BudgetDetails = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, BudgetDetails::default());
    }

    #[tokio::test]
    async fn get_template_details_success_with_query_params() {
        #[derive(Clone)]
        struct MockTemplateDetailService {}
        #[async_trait]
        impl TemplateDetailServiceExt for MockTemplateDetailService {
            async fn get_template_details(
                &mut self,
                month: MonthTarget,
            ) -> DatamizeResult<BudgetDetails> {
                assert_eq!(month, MonthTarget::Previous);
                Ok(BudgetDetails::default())
            }
        }
        let template_detail_service = Box::new(MockTemplateDetailService {});

        let app = get_detail_routes(template_detail_service);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/details?month=previous")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: BudgetDetails = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, BudgetDetails::default());
    }

    #[tokio::test]
    async fn get_template_details_error_400_with_unsupported_query_param_value() {
        #[derive(Clone)]
        struct MockTemplateDetailService {}
        #[async_trait]
        impl TemplateDetailServiceExt for MockTemplateDetailService {
            async fn get_template_details(
                &mut self,
                _month: MonthTarget,
            ) -> DatamizeResult<BudgetDetails> {
                Ok(BudgetDetails::default())
            }
        }
        let template_detail_service = Box::new(MockTemplateDetailService {});

        let app = get_detail_routes(template_detail_service);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/details?month=dfdfb")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn get_template_details_error_500() {
        #[derive(Clone)]
        struct MockTemplateDetailService {}
        #[async_trait]
        impl TemplateDetailServiceExt for MockTemplateDetailService {
            async fn get_template_details(
                &mut self,
                _month: MonthTarget,
            ) -> DatamizeResult<BudgetDetails> {
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
        let template_detail_service = Box::new(MockTemplateDetailService {});

        let app = get_detail_routes(template_detail_service);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/details")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
