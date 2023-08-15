use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_extra::extract::WithRejection;

use crate::{
    error::{DatamizeResult, HttpJsonDatamizeResult, JsonError},
    models::budget_template::{BudgeterConfig, SaveBudgeterConfig},
    services::budget_template::DynBudgeterService,
};

/// Returns all the budgeters.
#[tracing::instrument(skip_all)]
pub async fn get_all_budgeters(
    State(budgeter_service): State<DynBudgeterService>,
) -> HttpJsonDatamizeResult<Vec<BudgeterConfig>> {
    Ok(Json(budgeter_service.get_all_budgeters().await?))
}

/// Creates a new budgeter if it doesn't already exist and returns the newly created entity.
#[tracing::instrument(skip_all)]
pub async fn create_budgeter(
    State(budgeter_service): State<DynBudgeterService>,
    WithRejection(Json(body), _): WithRejection<Json<SaveBudgeterConfig>, JsonError>,
) -> DatamizeResult<impl IntoResponse> {
    Ok((
        StatusCode::CREATED,
        Json(budgeter_service.create_budgeter(body).await?),
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
        error::AppError, routes::api::budget_template::get_budgeter_routes,
        services::budget_template::MockBudgeterServiceExt,
    };

    use super::*;

    #[tokio::test]
    async fn get_all_budgeters_success() {
        let budgeters: Vec<BudgeterConfig> = Faker.fake();

        let mut budgeter_service = MockBudgeterServiceExt::new();
        let budgeters_cloned = budgeters.clone();
        budgeter_service
            .expect_get_all_budgeters()
            .returning(move || Ok(budgeters_cloned.clone()));

        let app = get_budgeter_routes(Arc::new(budgeter_service));

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/budgeters")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: Vec<BudgeterConfig> = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, budgeters);
    }

    #[tokio::test]
    async fn get_all_budgeters_error_500() {
        let mut budgeter_service = MockBudgeterServiceExt::new();
        budgeter_service.expect_get_all_budgeters().returning(|| {
            Err(AppError::InternalServerError(Into::into(
                sqlx::Error::RowNotFound,
            )))
        });

        let app = get_budgeter_routes(Arc::new(budgeter_service));

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/budgeters")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn create_budgeter_success() {
        let new_budgeter = Faker.fake::<SaveBudgeterConfig>();

        let mut budgeter_service = MockBudgeterServiceExt::new();
        let budgeter_cloned: BudgeterConfig = new_budgeter.clone().into();
        budgeter_service
            .expect_create_budgeter()
            .returning(move |_| Ok(budgeter_cloned.clone()));

        let app = get_budgeter_routes(Arc::new(budgeter_service));

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/budgeter")
                    .method("POST")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_vec(&new_budgeter).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: BudgeterConfig = serde_json::from_slice(&body).unwrap();
        assert_eq!(body.name, new_budgeter.name);
        assert_eq!(body.payee_ids, new_budgeter.payee_ids);
    }

    #[tokio::test]
    async fn create_budgeter_error_500() {
        let new_budgeter = Faker.fake::<SaveBudgeterConfig>();

        let mut budgeter_service = MockBudgeterServiceExt::new();
        budgeter_service
            .expect_create_budgeter()
            .returning(move |_| {
                Err(AppError::InternalServerError(Into::into(
                    sqlx::Error::RowNotFound,
                )))
            });

        let app = get_budgeter_routes(Arc::new(budgeter_service));

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/budgeter")
                    .method("POST")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_vec(&new_budgeter).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn create_budgeter_error_422_wrong_body_attribute() {
        #[derive(Debug, Clone, Serialize, Dummy)]
        struct Body {
            nameeeeeeeeeeeeee: String,
            payee_ids: Vec<Uuid>,
        }
        let body: Body = Faker.fake();

        let budgeter_service = MockBudgeterServiceExt::new();

        let app = get_budgeter_routes(Arc::new(budgeter_service));

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/budgeter")
                    .method("POST")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_vec(&body).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn create_budgeter_error_422_missing_body_attribute() {
        #[derive(Debug, Clone, Serialize, Dummy)]
        struct Body {
            payee_ids: Vec<Uuid>,
        }
        let body: Body = Faker.fake();

        let budgeter_service = MockBudgeterServiceExt::new();

        let app = get_budgeter_routes(Arc::new(budgeter_service));

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/budgeter")
                    .method("POST")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_vec(&body).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn create_budgeter_error_422_wrong_body_attribute_type() {
        #[derive(Debug, Clone, Serialize, Dummy)]
        struct Body {
            name: i64,
            payee_ids: Vec<Uuid>,
        }
        let body: Body = Faker.fake();

        let budgeter_service = MockBudgeterServiceExt::new();

        let app = get_budgeter_routes(Arc::new(budgeter_service));

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/budgeter")
                    .method("POST")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_vec(&body).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn create_budgeter_error_400_empty_body() {
        let budgeter_service = MockBudgeterServiceExt::new();

        let app = get_budgeter_routes(Arc::new(budgeter_service));

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/budgeter")
                    .method("POST")
                    .header("Content-Type", "application/json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn create_budgeter_error_415_missing_json_content_type() {
        let new_budgeter: SaveBudgeterConfig = Faker.fake();

        let budgeter_service = MockBudgeterServiceExt::new();

        let app = get_budgeter_routes(Arc::new(budgeter_service));

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/budgeter")
                    .method("POST")
                    .body(serde_json::to_vec(&new_budgeter).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }

    #[tokio::test]
    async fn create_budgeter_error_405_wrong_http_method() {
        let new_budgeter: SaveBudgeterConfig = Faker.fake();

        let budgeter_service = MockBudgeterServiceExt::new();

        let app = get_budgeter_routes(Arc::new(budgeter_service));

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/budgeter")
                    .method("PUT")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_vec(&new_budgeter).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn create_budgeter_error_409_when_already_exists() {
        let new_budgeter: SaveBudgeterConfig = Faker.fake();

        let mut budgeter_service = MockBudgeterServiceExt::new();

        budgeter_service
            .expect_create_budgeter()
            .returning(|_| Err(AppError::ResourceAlreadyExist));

        let app = get_budgeter_routes(Arc::new(budgeter_service));

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/budgeter")
                    .method("POST")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_vec(&new_budgeter).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CONFLICT);
    }
}
