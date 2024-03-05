use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use datamize_domain::{async_trait, BudgetDetails, MonthTarget};
use http_body_util::BodyExt;
use pretty_assertions::assert_eq;
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
        async fn get_template_details(
            &self,
            month: MonthTarget,
            use_category_groups_as_sub_type: bool,
        ) -> DatamizeResult<BudgetDetails> {
            assert_eq!(month, MonthTarget::Current);
            assert_eq!(use_category_groups_as_sub_type, true);
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
        async fn get_template_details(
            &self,
            month: MonthTarget,
            use_category_groups_as_sub_type: bool,
        ) -> DatamizeResult<BudgetDetails> {
            assert_eq!(month, MonthTarget::Previous);
            assert_eq!(use_category_groups_as_sub_type, false);
            Ok(BudgetDetails::default())
        }
    }
    let template_detail_service = Arc::new(MockTemplateDetailService {});

    let app = get_detail_routes(template_detail_service);
    let response = app
        .oneshot(
            Request::builder()
                .uri("/details?month=previous&use_category_groups_as_sub_type=false")
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
        async fn get_template_details(
            &self,
            _month: MonthTarget,
            _use_category_groups_as_sub_type: bool,
        ) -> DatamizeResult<BudgetDetails> {
            Ok(BudgetDetails::default())
        }
    }
    let template_detail_service = Arc::new(MockTemplateDetailService {});

    let app = get_detail_routes(template_detail_service);
    let response = app
        .oneshot(
            Request::builder()
                .uri("/details?month=dfdfb&use_category_groups_as_sub_type=ddewewe")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
