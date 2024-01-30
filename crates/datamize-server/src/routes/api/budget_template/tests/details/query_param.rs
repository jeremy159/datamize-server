use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use datamize_domain::{async_trait, BudgetDetails, MonthTarget};
use http_body_util::BodyExt;
use tower::ServiceExt;

use crate::{
    error::DatamizeResult, routes::api::budget_template::get_detail_routes,
    services::budget_template::TemplateDetailServiceExt,
};

#[tokio::test]
async fn get_template_details_success_with_no_query_params() {
    #[derive(Clone)]
    struct MockTemplateDetailService {}
    #[async_trait]
    impl TemplateDetailServiceExt for MockTemplateDetailService {
        async fn get_template_details(&self, month: MonthTarget) -> DatamizeResult<BudgetDetails> {
            assert_eq!(month, MonthTarget::Current);
            Ok(BudgetDetails::default())
        }
    }
    let template_detail_service = Arc::new(MockTemplateDetailService {});

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

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: BudgetDetails = serde_json::from_slice(&body).unwrap();
    assert_eq!(body, BudgetDetails::default());
}

#[tokio::test]
async fn get_template_details_success_with_query_params() {
    #[derive(Clone)]
    struct MockTemplateDetailService {}
    #[async_trait]
    impl TemplateDetailServiceExt for MockTemplateDetailService {
        async fn get_template_details(&self, month: MonthTarget) -> DatamizeResult<BudgetDetails> {
            assert_eq!(month, MonthTarget::Previous);
            Ok(BudgetDetails::default())
        }
    }
    let template_detail_service = Arc::new(MockTemplateDetailService {});

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

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: BudgetDetails = serde_json::from_slice(&body).unwrap();
    assert_eq!(body, BudgetDetails::default());
}

#[tokio::test]
async fn get_template_details_error_400_with_unsupported_query_param_value() {
    #[derive(Clone)]
    struct MockTemplateDetailService {}
    #[async_trait]
    impl TemplateDetailServiceExt for MockTemplateDetailService {
        async fn get_template_details(&self, _month: MonthTarget) -> DatamizeResult<BudgetDetails> {
            Ok(BudgetDetails::default())
        }
    }
    let template_detail_service = Arc::new(MockTemplateDetailService {});

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
