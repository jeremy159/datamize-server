use axum::{
    extract::{Path, State},
    Json,
};
use axum_extra::extract::WithRejection;
use datamize_domain::{ExpenseCategorization, Uuid};

use crate::{
    error::{HttpJsonDatamizeResult, JsonError},
    services::budget_template::DynExpenseCategorizationService,
};

/// Returns an expense categorization.
#[tracing::instrument(skip_all)]
pub async fn get_expense_categorization(
    Path(id): Path<Uuid>,
    State(expense_categorization_service): State<DynExpenseCategorizationService>,
) -> HttpJsonDatamizeResult<ExpenseCategorization> {
    Ok(Json(
        expense_categorization_service
            .get_expense_categorization(id)
            .await?,
    ))
}

/// Updates the expense categorization and returns the entity.
#[tracing::instrument(skip_all)]
pub async fn update_expense_categorization(
    Path(_id): Path<Uuid>,
    State(expense_categorization_service): State<DynExpenseCategorizationService>,
    WithRejection(Json(body), _): WithRejection<Json<ExpenseCategorization>, JsonError>,
) -> HttpJsonDatamizeResult<ExpenseCategorization> {
    Ok(Json(
        expense_categorization_service
            .update_expense_categorization(body)
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

    use crate::{
        error::AppError, routes::api::budget_template::get_expense_categorization_routes,
        services::budget_template::MockExpenseCategorizationServiceExt,
    };

    use super::*;

    #[tokio::test]
    async fn get_expense_categorization_success() {
        let expense_categorization: ExpenseCategorization = Faker.fake();

        let mut expense_categorization_service = MockExpenseCategorizationServiceExt::new();
        let expense_categorization_cloned = expense_categorization.clone();
        expense_categorization_service
            .expect_get_expense_categorization()
            .returning(move |_| Ok(expense_categorization_cloned.clone()));

        let app = get_expense_categorization_routes(Arc::new(expense_categorization_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/expense_categorization/{:?}",
                        expense_categorization.id
                    ))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: ExpenseCategorization = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, expense_categorization);
    }

    #[tokio::test]
    async fn get_expense_categorization_error_500() {
        let mut expense_categorization_service = MockExpenseCategorizationServiceExt::new();
        expense_categorization_service
            .expect_get_expense_categorization()
            .returning(move |_| {
                Err(AppError::InternalServerError(Into::into(
                    sqlx::Error::RowNotFound,
                )))
            });

        let app = get_expense_categorization_routes(Arc::new(expense_categorization_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/expense_categorization/{:?}",
                        Faker.fake::<Uuid>()
                    ))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn get_expense_categorization_error_404_non_existing_resource() {
        let mut expense_categorization_service = MockExpenseCategorizationServiceExt::new();
        expense_categorization_service
            .expect_get_expense_categorization()
            .returning(move |_| Err(AppError::ResourceNotFound));

        let app = get_expense_categorization_routes(Arc::new(expense_categorization_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/expense_categorization/{:?}",
                        Faker.fake::<Uuid>()
                    ))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn get_expense_categorization_error_400_invalid_uuid_format_in_path() {
        let expense_categorization_service = MockExpenseCategorizationServiceExt::new();

        let app = get_expense_categorization_routes(Arc::new(expense_categorization_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/expense_categorization/{:?}", Faker.fake::<i64>()))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn update_expense_categorization_success() {
        let expense_categorization: ExpenseCategorization = Faker.fake();

        let mut expense_categorization_service = MockExpenseCategorizationServiceExt::new();
        let expense_categorization_cloned = expense_categorization.clone();
        expense_categorization_service
            .expect_update_expense_categorization()
            .returning(move |_| Ok(expense_categorization_cloned.clone()));

        let app = get_expense_categorization_routes(Arc::new(expense_categorization_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/expense_categorization/{:?}",
                        expense_categorization.id
                    ))
                    .method("PUT")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_vec(&expense_categorization).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: ExpenseCategorization = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, expense_categorization);
    }

    #[tokio::test]
    async fn update_expense_categorization_error_500() {
        let expense_categorization: ExpenseCategorization = Faker.fake();

        let mut expense_categorization_service = MockExpenseCategorizationServiceExt::new();
        expense_categorization_service
            .expect_update_expense_categorization()
            .returning(move |_| {
                Err(AppError::InternalServerError(Into::into(
                    sqlx::Error::RowNotFound,
                )))
            });

        let app = get_expense_categorization_routes(Arc::new(expense_categorization_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/expense_categorization/{:?}",
                        expense_categorization.id
                    ))
                    .method("PUT")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_vec(&expense_categorization).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn update_expense_categorization_error_400_invalid_uuid_format_in_path() {
        let expense_categorization: ExpenseCategorization = Faker.fake();

        let expense_categorization_service = MockExpenseCategorizationServiceExt::new();

        let app = get_expense_categorization_routes(Arc::new(expense_categorization_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/expense_categorization/{:?}", Faker.fake::<i64>()))
                    .method("PUT")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_vec(&expense_categorization).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn update_expense_categorization_error_422_wrong_body_attribute() {
        #[derive(Debug, Clone, Serialize, Dummy)]
        struct Body {
            id: Uuid,
            nameeeeeeeeeeeeee: String,
            payee_ids: Vec<Uuid>,
        }
        let body: Body = Faker.fake();

        let expense_categorization_service = MockExpenseCategorizationServiceExt::new();

        let app = get_expense_categorization_routes(Arc::new(expense_categorization_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/expense_categorization/{:?}", body.id))
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
    async fn update_expense_categorization_error_422_missing_body_attribute() {
        #[derive(Debug, Clone, Serialize, Dummy)]
        struct Body {
            id: Uuid,
            payee_ids: Vec<Uuid>,
        }
        let body: Body = Faker.fake();

        let expense_categorization_service = MockExpenseCategorizationServiceExt::new();

        let app = get_expense_categorization_routes(Arc::new(expense_categorization_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/expense_categorization/{:?}", body.id))
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
    async fn update_expense_categorization_error_422_wrong_body_attribute_type() {
        #[derive(Debug, Clone, Serialize, Dummy)]
        struct Body {
            id: Uuid,
            name: i64,
            payee_ids: Vec<Uuid>,
        }
        let body: Body = Faker.fake();

        let expense_categorization_service = MockExpenseCategorizationServiceExt::new();

        let app = get_expense_categorization_routes(Arc::new(expense_categorization_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/expense_categorization/{:?}", body.id))
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
    async fn update_expense_categorization_error_400_empty_body() {
        let expense_categorization_service = MockExpenseCategorizationServiceExt::new();

        let app = get_expense_categorization_routes(Arc::new(expense_categorization_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/expense_categorization/{:?}",
                        Faker.fake::<Uuid>()
                    ))
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
    async fn update_expense_categorization_error_415_missing_json_content_type() {
        let expense_categorization: ExpenseCategorization = Faker.fake();

        let expense_categorization_service = MockExpenseCategorizationServiceExt::new();

        let app = get_expense_categorization_routes(Arc::new(expense_categorization_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/expense_categorization/{:?}",
                        expense_categorization.id
                    ))
                    .method("PUT")
                    .body(serde_json::to_vec(&expense_categorization).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }

    #[tokio::test]
    async fn update_expense_categorization_error_405_wrong_http_method() {
        let expense_categorization: ExpenseCategorization = Faker.fake();

        let expense_categorization_service = MockExpenseCategorizationServiceExt::new();

        let app = get_expense_categorization_routes(Arc::new(expense_categorization_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/expense_categorization/{:?}",
                        expense_categorization.id
                    ))
                    .method("POST")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_vec(&expense_categorization).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn update_expense_categorization_error_404_when_not_found() {
        let expense_categorization: ExpenseCategorization = Faker.fake();

        let mut expense_categorization_service = MockExpenseCategorizationServiceExt::new();

        expense_categorization_service
            .expect_update_expense_categorization()
            .returning(|_| Err(AppError::ResourceNotFound));

        let app = get_expense_categorization_routes(Arc::new(expense_categorization_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/expense_categorization/{:?}",
                        expense_categorization.id
                    ))
                    .method("PUT")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_vec(&expense_categorization).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
