use axum::{
    extract::{Query, State},
    Json,
};
use datamize_domain::{BudgetSummary, TemplateParams};

use crate::{error::HttpJsonDatamizeResult, services::budget_template::DynTemplateSummaryService};

/// Returns a budget template summary.
/// Can specify the month to get summary from.
/// /template/summary?month=previous
/// Possible values to pass in query params are `previous` and `next`. If nothing is specified,
/// the current month will be used.
pub async fn template_summary(
    State(mut template_summary_service): State<DynTemplateSummaryService>,
    template_params: Query<TemplateParams>,
) -> HttpJsonDatamizeResult<BudgetSummary> {
    let month = template_params.month.unwrap_or_default();

    Ok(Json(
        template_summary_service.get_template_summary(month).await?,
    ))
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use datamize_domain::{async_trait, MonthTarget};
    use fake::{Fake, Faker};
    use tower::ServiceExt;

    use crate::{
        error::{AppError, DatamizeResult},
        routes::api::budget_template::get_summary_routes,
        services::budget_template::TemplateSummaryServiceExt,
    };

    use super::*;

    #[tokio::test]
    async fn get_template_summary_success_with_no_query_params() {
        #[derive(Clone)]
        struct MockTemplateSummaryService {}
        #[async_trait]
        impl TemplateSummaryServiceExt for MockTemplateSummaryService {
            async fn get_template_summary(
                &mut self,
                month: MonthTarget,
            ) -> DatamizeResult<BudgetSummary> {
                assert_eq!(month, MonthTarget::Current);
                Ok(BudgetSummary::default())
            }
        }
        let template_summary_service = Box::new(MockTemplateSummaryService {});

        let app = get_summary_routes(template_summary_service);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/summary")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: BudgetSummary = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, BudgetSummary::default());
    }

    #[tokio::test]
    async fn get_template_summary_success_with_query_params() {
        #[derive(Clone)]
        struct MockTemplateSummaryService {}
        #[async_trait]
        impl TemplateSummaryServiceExt for MockTemplateSummaryService {
            async fn get_template_summary(
                &mut self,
                month: MonthTarget,
            ) -> DatamizeResult<BudgetSummary> {
                assert_eq!(month, MonthTarget::Previous);
                Ok(BudgetSummary::default())
            }
        }
        let template_summary_service = Box::new(MockTemplateSummaryService {});

        let app = get_summary_routes(template_summary_service);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/summary?month=previous")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: BudgetSummary = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, BudgetSummary::default());
    }

    #[tokio::test]
    async fn get_template_summary_error_400_with_unsupported_query_param_value() {
        #[derive(Clone)]
        struct MockTemplateSummaryService {}
        #[async_trait]
        impl TemplateSummaryServiceExt for MockTemplateSummaryService {
            async fn get_template_summary(
                &mut self,
                _month: MonthTarget,
            ) -> DatamizeResult<BudgetSummary> {
                Ok(BudgetSummary::default())
            }
        }
        let template_summary_service = Box::new(MockTemplateSummaryService {});

        let app = get_summary_routes(template_summary_service);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/summary?month=dfdfb")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn get_template_summary_error_500() {
        #[derive(Clone)]
        struct MockTemplateSummaryService {}
        #[async_trait]
        impl TemplateSummaryServiceExt for MockTemplateSummaryService {
            async fn get_template_summary(
                &mut self,
                _month: MonthTarget,
            ) -> DatamizeResult<BudgetSummary> {
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
        let template_summary_service = Box::new(MockTemplateSummaryService {});

        let app = get_summary_routes(template_summary_service);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/summary")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
