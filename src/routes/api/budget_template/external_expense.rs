use axum::{
    extract::{Path, State},
    Json,
};
use axum_extra::extract::WithRejection;
use uuid::Uuid;

use crate::{
    error::{HttpJsonDatamizeResult, JsonError},
    models::budget_template::ExternalExpense,
    services::budget_template::DynExternalExpenseService,
};

/// Returns an external expense.
#[tracing::instrument(skip_all)]
pub async fn get_external_expense(
    Path(id): Path<Uuid>,
    State(external_expense_service): State<DynExternalExpenseService>,
) -> HttpJsonDatamizeResult<ExternalExpense> {
    Ok(Json(
        external_expense_service.get_external_expense(id).await?,
    ))
}

/// Updates the external expense and returns the entity.
#[tracing::instrument(skip_all)]
pub async fn update_external_expense(
    Path(_id): Path<Uuid>,
    State(external_expense_service): State<DynExternalExpenseService>,
    WithRejection(Json(body), _): WithRejection<Json<ExternalExpense>, JsonError>,
) -> HttpJsonDatamizeResult<ExternalExpense> {
    Ok(Json(
        external_expense_service
            .update_external_expense(body)
            .await?,
    ))
}

/// Deletes the external expense and returns the entity.
#[tracing::instrument(skip_all)]
pub async fn delete_external_expense(
    Path(id): Path<Uuid>,
    State(external_expense_service): State<DynExternalExpenseService>,
) -> HttpJsonDatamizeResult<ExternalExpense> {
    Ok(Json(
        external_expense_service.delete_external_expense(id).await?,
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
        error::AppError, routes::api::budget_template::get_external_expense_routes,
        services::budget_template::MockExternalExpenseServiceExt,
    };

    use super::*;

    #[tokio::test]
    async fn get_external_expense_success() {
        let external_expense: ExternalExpense = Faker.fake();

        let mut external_expense_service = MockExternalExpenseServiceExt::new();
        let external_expense_cloned = external_expense.clone();
        external_expense_service
            .expect_get_external_expense()
            .returning(move |_| Ok(external_expense_cloned.clone()));

        let app = get_external_expense_routes(Arc::new(external_expense_service));

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/external_expense/{:?}", external_expense.id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: ExternalExpense = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, external_expense);
    }

    #[tokio::test]
    async fn get_external_expense_error_500() {
        let mut external_expense_service = MockExternalExpenseServiceExt::new();
        external_expense_service
            .expect_get_external_expense()
            .returning(move |_| {
                Err(AppError::InternalServerError(Into::into(
                    sqlx::Error::RowNotFound,
                )))
            });

        let app = get_external_expense_routes(Arc::new(external_expense_service));

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/external_expense/{:?}", Faker.fake::<Uuid>()))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn get_external_expense_error_404_non_existing_resource() {
        let mut external_expense_service = MockExternalExpenseServiceExt::new();
        external_expense_service
            .expect_get_external_expense()
            .returning(move |_| Err(AppError::ResourceNotFound));

        let app = get_external_expense_routes(Arc::new(external_expense_service));

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/external_expense/{:?}", Faker.fake::<Uuid>()))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn get_external_expense_error_400_invalid_uuid_format_in_path() {
        let external_expense_service = MockExternalExpenseServiceExt::new();

        let app = get_external_expense_routes(Arc::new(external_expense_service));

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/external_expense/{:?}", Faker.fake::<i64>()))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn update_external_expense_success() {
        let external_expense: ExternalExpense = Faker.fake();

        let mut external_expense_service = MockExternalExpenseServiceExt::new();
        let external_expense_cloned = external_expense.clone();
        external_expense_service
            .expect_update_external_expense()
            .returning(move |_| Ok(external_expense_cloned.clone()));

        let app = get_external_expense_routes(Arc::new(external_expense_service));

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/external_expense/{:?}", external_expense.id))
                    .method("PUT")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_vec(&external_expense).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: ExternalExpense = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, external_expense);
    }

    #[tokio::test]
    async fn update_external_expense_error_500() {
        let external_expense: ExternalExpense = Faker.fake();

        let mut external_expense_service = MockExternalExpenseServiceExt::new();
        external_expense_service
            .expect_update_external_expense()
            .returning(move |_| {
                Err(AppError::InternalServerError(Into::into(
                    sqlx::Error::RowNotFound,
                )))
            });

        let app = get_external_expense_routes(Arc::new(external_expense_service));

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/external_expense/{:?}", external_expense.id))
                    .method("PUT")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_vec(&external_expense).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn update_external_expense_error_400_invalid_uuid_format_in_path() {
        let external_expense: ExternalExpense = Faker.fake();

        let external_expense_service = MockExternalExpenseServiceExt::new();

        let app = get_external_expense_routes(Arc::new(external_expense_service));

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/external_expense/{:?}", Faker.fake::<i64>()))
                    .method("PUT")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_vec(&external_expense).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn update_external_expense_error_422_wrong_body_attribute() {
        #[derive(Debug, Clone, Serialize, Dummy)]
        struct Body {
            id: Uuid,
            nameeeeeeeeeeeeee: String,
            payee_ids: Vec<Uuid>,
        }
        let body: Body = Faker.fake();

        let external_expense_service = MockExternalExpenseServiceExt::new();

        let app = get_external_expense_routes(Arc::new(external_expense_service));

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/external_expense/{:?}", body.id))
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
    async fn update_external_expense_error_422_missing_body_attribute() {
        #[derive(Debug, Clone, Serialize, Dummy)]
        struct Body {
            id: Uuid,
            payee_ids: Vec<Uuid>,
        }
        let body: Body = Faker.fake();

        let external_expense_service = MockExternalExpenseServiceExt::new();

        let app = get_external_expense_routes(Arc::new(external_expense_service));

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/external_expense/{:?}", body.id))
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
    async fn update_external_expense_error_422_wrong_body_attribute_type() {
        #[derive(Debug, Clone, Serialize, Dummy)]
        struct Body {
            id: Uuid,
            name: i64,
            payee_ids: Vec<Uuid>,
        }
        let body: Body = Faker.fake();

        let external_expense_service = MockExternalExpenseServiceExt::new();

        let app = get_external_expense_routes(Arc::new(external_expense_service));

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/external_expense/{:?}", body.id))
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
    async fn update_external_expense_error_400_empty_body() {
        let external_expense_service = MockExternalExpenseServiceExt::new();

        let app = get_external_expense_routes(Arc::new(external_expense_service));

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/external_expense/{:?}", Faker.fake::<Uuid>()))
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
    async fn update_external_expense_error_415_missing_json_content_type() {
        let external_expense: ExternalExpense = Faker.fake();

        let external_expense_service = MockExternalExpenseServiceExt::new();

        let app = get_external_expense_routes(Arc::new(external_expense_service));

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/external_expense/{:?}", external_expense.id))
                    .method("PUT")
                    .body(serde_json::to_vec(&external_expense).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }

    #[tokio::test]
    async fn update_external_expense_error_405_wrong_http_method() {
        let external_expense: ExternalExpense = Faker.fake();

        let external_expense_service = MockExternalExpenseServiceExt::new();

        let app = get_external_expense_routes(Arc::new(external_expense_service));

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/external_expense/{:?}", external_expense.id))
                    .method("POST")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_vec(&external_expense).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn update_external_expense_error_404_when_not_found() {
        let external_expense: ExternalExpense = Faker.fake();

        let mut external_expense_service = MockExternalExpenseServiceExt::new();

        external_expense_service
            .expect_update_external_expense()
            .returning(|_| Err(AppError::ResourceNotFound));

        let app = get_external_expense_routes(Arc::new(external_expense_service));

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/external_expense/{:?}", external_expense.id))
                    .method("PUT")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_vec(&external_expense).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn delete_external_expense_success() {
        let external_expense: ExternalExpense = Faker.fake();

        let mut external_expense_service = MockExternalExpenseServiceExt::new();
        let external_expense_cloned = external_expense.clone();
        external_expense_service
            .expect_delete_external_expense()
            .returning(move |_| Ok(external_expense_cloned.clone()));

        let app = get_external_expense_routes(Arc::new(external_expense_service));

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/external_expense/{:?}", external_expense.id))
                    .method("DELETE")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: ExternalExpense = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, external_expense);
    }

    #[tokio::test]
    async fn delete_external_expense_error_500() {
        let mut external_expense_service = MockExternalExpenseServiceExt::new();
        external_expense_service
            .expect_delete_external_expense()
            .returning(move |_| {
                Err(AppError::InternalServerError(Into::into(
                    sqlx::Error::RowNotFound,
                )))
            });

        let app = get_external_expense_routes(Arc::new(external_expense_service));

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/external_expense/{:?}", Faker.fake::<Uuid>()))
                    .method("DELETE")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn delete_external_expense_error_404_non_existing_resource() {
        let mut external_expense_service = MockExternalExpenseServiceExt::new();
        external_expense_service
            .expect_delete_external_expense()
            .returning(move |_| Err(AppError::ResourceNotFound));

        let app = get_external_expense_routes(Arc::new(external_expense_service));

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/external_expense/{:?}", Faker.fake::<Uuid>()))
                    .method("DELETE")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn delete_external_expense_error_400_invalid_uuid_format_in_path() {
        let external_expense_service = MockExternalExpenseServiceExt::new();

        let app = get_external_expense_routes(Arc::new(external_expense_service));

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/external_expense/{:?}", Faker.fake::<i64>()))
                    .method("DELETE")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
