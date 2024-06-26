use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use datamize_domain::{async_trait, BudgetSummary, MonthTarget};
use http_body_util::BodyExt;
use pretty_assertions::assert_eq;
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
        async fn get_template_summary(
            &self,
            month: MonthTarget,
            use_category_groups_as_sub_type: bool,
        ) -> DatamizeResult<BudgetSummary> {
            assert_eq!(month, MonthTarget::Current);
            assert_eq!(use_category_groups_as_sub_type, true);
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

    let body = response.into_body().collect().await.unwrap().to_bytes();
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
            &self,
            month: MonthTarget,
            use_category_groups_as_sub_type: bool,
        ) -> DatamizeResult<BudgetSummary> {
            assert_eq!(month, MonthTarget::Previous);
            assert_eq!(use_category_groups_as_sub_type, false);
            Ok(BudgetSummary::default())
        }
    }
    let template_summary_service = Arc::new(MockTemplateSummaryService {});

    let app = get_summary_routes(template_summary_service);
    let response = app
        .oneshot(
            Request::builder()
                .uri("/summary?month=previous&use_category_groups_as_sub_type=false")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
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
            &self,
            _month: MonthTarget,
            _use_category_groups_as_sub_type: bool,
        ) -> DatamizeResult<BudgetSummary> {
            Ok(BudgetSummary::default())
        }
    }
    let template_summary_service = Arc::new(MockTemplateSummaryService {});

    let app = get_summary_routes(template_summary_service);
    let response = app
        .oneshot(
            Request::builder()
                .uri("/summary?month=dfdfb&use_category_groups_as_sub_type=ddewewe")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
