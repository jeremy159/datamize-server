use axum::{extract::State, Json};
use axum_extra::extract::WithRejection;

use crate::{
    error::{HttpJsonDatamizeResult, JsonError},
    models::budget_template::ExpenseCategorization,
    services::budget_template::DynExpenseCategorizationService,
};

/// Returns all expenses categorization.
#[tracing::instrument(skip_all)]
pub async fn get_all_expenses_categorization(
    State(expense_categorization_service): State<DynExpenseCategorizationService>,
) -> HttpJsonDatamizeResult<Vec<ExpenseCategorization>> {
    Ok(Json(
        expense_categorization_service
            .get_all_expenses_categorization()
            .await?,
    ))
}

/// Updates all expenses categorization and returns the collection.
#[tracing::instrument(skip_all)]
pub async fn update_all_expenses_categorization(
    State(expense_categorization_service): State<DynExpenseCategorizationService>,
    WithRejection(Json(body), _): WithRejection<Json<Vec<ExpenseCategorization>>, JsonError>,
) -> HttpJsonDatamizeResult<Vec<ExpenseCategorization>> {
    Ok(Json(
        expense_categorization_service
            .update_all_expenses_categorization(body)
            .await?,
    ))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use fake::{Dummy, Fake, Faker};
    use serde::Serialize;
    use tower::ServiceExt;
    use uuid::Uuid;

    use crate::{
        error::AppError,
        models::budget_template::{ExpenseType, SubExpenseType},
        routes::api::budget_template::get_expense_categorization_routes,
        services::budget_template::MockExpenseCategorizationServiceExt,
    };

    use super::*;

    #[tokio::test]
    async fn get_all_expenses_categorization_success() {
        let expenses_categorization: Vec<ExpenseCategorization> = Faker.fake();

        let mut expense_categorization_service = MockExpenseCategorizationServiceExt::new();
        let expenses_categorization_cloned = expenses_categorization.clone();
        expense_categorization_service
            .expect_get_all_expenses_categorization()
            .returning(move || Ok(expenses_categorization_cloned.clone()));

        let app = get_expense_categorization_routes(Arc::new(expense_categorization_service));

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/expenses_categorization")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: Vec<ExpenseCategorization> = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, expenses_categorization);
    }

    #[tokio::test]
    async fn get_all_expenses_categorization_error_500() {
        let mut expense_categorization_service = MockExpenseCategorizationServiceExt::new();
        expense_categorization_service
            .expect_get_all_expenses_categorization()
            .returning(|| {
                Err(AppError::InternalServerError(Into::into(
                    sqlx::Error::RowNotFound,
                )))
            });

        let app = get_expense_categorization_routes(Arc::new(expense_categorization_service));

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/expenses_categorization")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn update_all_expenses_categorization_success() {
        let expenses_categorization: Vec<ExpenseCategorization> = Faker.fake();

        let mut expense_categorization_service = MockExpenseCategorizationServiceExt::new();
        let expenses_categorization_cloned = expenses_categorization.clone();
        expense_categorization_service
            .expect_update_all_expenses_categorization()
            .returning(move |_| Ok(expenses_categorization_cloned.clone()));

        let app = get_expense_categorization_routes(Arc::new(expense_categorization_service));

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/expenses_categorization")
                    .method("PUT")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_vec(&expenses_categorization).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: Vec<ExpenseCategorization> = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, expenses_categorization);
    }

    #[tokio::test]
    async fn update_all_expenses_categorization_error_500() {
        let expenses_categorization: Vec<ExpenseCategorization> = Faker.fake();

        let mut expense_categorization_service = MockExpenseCategorizationServiceExt::new();
        expense_categorization_service
            .expect_update_all_expenses_categorization()
            .returning(move |_| {
                Err(AppError::InternalServerError(Into::into(
                    sqlx::Error::RowNotFound,
                )))
            });

        let app = get_expense_categorization_routes(Arc::new(expense_categorization_service));

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/expenses_categorization")
                    .method("PUT")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_vec(&expenses_categorization).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn update_all_expenses_categorization_error_422_wrong_body_attribute() {
        #[derive(Debug, Clone, Serialize, Dummy)]
        struct Body {
            id: Uuid,
            nameeeeeeeeeeeee: String,
            #[serde(rename = "type")]
            expense_type: ExpenseType,
            #[serde(rename = "sub_type")]
            sub_expense_type: SubExpenseType,
        }
        let body: Body = Faker.fake();

        let expense_categorization_service = MockExpenseCategorizationServiceExt::new();

        let app = get_expense_categorization_routes(Arc::new(expense_categorization_service));

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/expenses_categorization")
                    .method("PUT")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_vec(&body).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn update_all_expenses_categorization_error_422_missing_body_attribute() {
        #[derive(Debug, Clone, Serialize, Dummy)]
        struct Body {
            id: Uuid,
            #[serde(rename = "type")]
            expense_type: ExpenseType,
            #[serde(rename = "sub_type")]
            sub_expense_type: SubExpenseType,
        }
        let body: Body = Faker.fake();

        let expense_categorization_service = MockExpenseCategorizationServiceExt::new();

        let app = get_expense_categorization_routes(Arc::new(expense_categorization_service));

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/expenses_categorization")
                    .method("PUT")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_vec(&body).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn update_all_expenses_categorization_error_422_wrong_body_attribute_type() {
        #[derive(Debug, Clone, Serialize, Dummy)]
        struct Body {
            id: Uuid,
            name: i64,
            #[serde(rename = "type")]
            expense_type: ExpenseType,
            #[serde(rename = "sub_type")]
            sub_expense_type: SubExpenseType,
        }
        let body: Body = Faker.fake();

        let expense_categorization_service = MockExpenseCategorizationServiceExt::new();

        let app = get_expense_categorization_routes(Arc::new(expense_categorization_service));

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/expenses_categorization")
                    .method("PUT")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_vec(&body).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn update_all_expenses_categorization_error_400_empty_body() {
        let expense_categorization_service = MockExpenseCategorizationServiceExt::new();

        let app = get_expense_categorization_routes(Arc::new(expense_categorization_service));

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/expenses_categorization")
                    .method("PUT")
                    .header("Content-Type", "application/json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn update_all_expenses_categorization_error_415_missing_json_content_type() {
        let expenses_categorization: Vec<ExpenseCategorization> = Faker.fake();

        let expense_categorization_service = MockExpenseCategorizationServiceExt::new();

        let app = get_expense_categorization_routes(Arc::new(expense_categorization_service));

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/expenses_categorization")
                    .method("PUT")
                    .body(serde_json::to_vec(&expenses_categorization).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }

    #[tokio::test]
    async fn update_all_expenses_categorization_error_405_wrong_http_method() {
        let expenses_categorization: Vec<ExpenseCategorization> = Faker.fake();

        let expense_categorization_service = MockExpenseCategorizationServiceExt::new();

        let app = get_expense_categorization_routes(Arc::new(expense_categorization_service));

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/expenses_categorization")
                    .method("POST")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_vec(&expenses_categorization).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }
}
