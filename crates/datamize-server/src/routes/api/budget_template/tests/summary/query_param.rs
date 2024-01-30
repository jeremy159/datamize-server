use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use datamize_domain::{async_trait, BudgetSummary, MonthTarget};
use tower::ServiceExt;

use crate::{
    error::DatamizeResult, routes::api::budget_template::get_summary_routes,
    services::budget_template::TemplateSummaryServiceExt,
};

#[tokio::test]
async fn get_template_summary_success_with_no_query_params() {
    #[derive(Clone)]
    struct MockTemplateSummaryService {}
    #[async_trait]
    impl TemplateSummaryServiceExt for MockTemplateSummaryService {
        async fn get_template_summary(&self, month: MonthTarget) -> DatamizeResult<BudgetSummary> {
            assert_eq!(month, MonthTarget::Current);
            Ok(BudgetSummary::default())
        }
    }
    let template_summary_service = Arc::new(MockTemplateSummaryService {});

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
        async fn get_template_summary(&self, month: MonthTarget) -> DatamizeResult<BudgetSummary> {
            assert_eq!(month, MonthTarget::Previous);
            Ok(BudgetSummary::default())
        }
    }
    let template_summary_service = Arc::new(MockTemplateSummaryService {});

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
        async fn get_template_summary(&self, _month: MonthTarget) -> DatamizeResult<BudgetSummary> {
            Ok(BudgetSummary::default())
        }
    }
    let template_summary_service = Arc::new(MockTemplateSummaryService {});

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
