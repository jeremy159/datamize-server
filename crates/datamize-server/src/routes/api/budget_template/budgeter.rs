use axum::{
    extract::{Path, State},
    Json,
};
use axum_extra::extract::WithRejection;
use datamize_domain::{BudgeterConfig, Uuid};

use crate::{
    error::{HttpJsonDatamizeResult, JsonError},
    services::budget_template::DynBudgeterService,
};

/// Returns a budgeter's config.
#[tracing::instrument(skip_all)]
pub async fn get_budgeter(
    Path(id): Path<Uuid>,
    State(budgeter_service): State<DynBudgeterService>,
) -> HttpJsonDatamizeResult<BudgeterConfig> {
    Ok(Json(budgeter_service.get_budgeter(id).await?))
}

/// Updates the budgeter's name and payee_ids.
#[tracing::instrument(skip_all)]
pub async fn update_budgeter(
    Path(_id): Path<Uuid>,
    State(budgeter_service): State<DynBudgeterService>,
    WithRejection(Json(body), _): WithRejection<Json<BudgeterConfig>, JsonError>,
) -> HttpJsonDatamizeResult<BudgeterConfig> {
    Ok(Json(budgeter_service.update_budgeter(body).await?))
}

/// Deletes the budgeter and returns the entity.
#[tracing::instrument(skip_all)]
pub async fn delete_budgeter(
    Path(id): Path<Uuid>,
    State(budgeter_service): State<DynBudgeterService>,
) -> HttpJsonDatamizeResult<BudgeterConfig> {
    Ok(Json(budgeter_service.delete_budgeter(id).await?))
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
        error::AppError, routes::api::budget_template::get_budgeter_routes,
        services::budget_template::MockBudgeterServiceExt,
    };

    use super::*;

    #[tokio::test]
    async fn get_budgeter_success() {
        let budgeter: BudgeterConfig = Faker.fake();

        let mut budgeter_service = MockBudgeterServiceExt::new();
        let budgeter_cloned = budgeter.clone();
        budgeter_service
            .expect_get_budgeter()
            .returning(move |_| Ok(budgeter_cloned.clone()));

        let app = get_budgeter_routes(Arc::new(budgeter_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/budgeter/{:?}", budgeter.id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: BudgeterConfig = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, budgeter);
    }

    #[tokio::test]
    async fn get_budgeter_error_500() {
        let mut budgeter_service = MockBudgeterServiceExt::new();
        budgeter_service.expect_get_budgeter().returning(move |_| {
            Err(AppError::InternalServerError(Into::into(
                sqlx::Error::RowNotFound,
            )))
        });

        let app = get_budgeter_routes(Arc::new(budgeter_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/budgeter/{:?}", Faker.fake::<Uuid>()))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn get_budgeter_error_404_non_existing_resource() {
        let mut budgeter_service = MockBudgeterServiceExt::new();
        budgeter_service
            .expect_get_budgeter()
            .returning(move |_| Err(AppError::ResourceNotFound));

        let app = get_budgeter_routes(Arc::new(budgeter_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/budgeter/{:?}", Faker.fake::<Uuid>()))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn get_budgeter_error_400_invalid_uuid_format_in_path() {
        let budgeter_service = MockBudgeterServiceExt::new();

        let app = get_budgeter_routes(Arc::new(budgeter_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/budgeter/{:?}", Faker.fake::<i64>()))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn update_budgeter_success() {
        let budgeter: BudgeterConfig = Faker.fake();

        let mut budgeter_service = MockBudgeterServiceExt::new();
        let budgeter_cloned = budgeter.clone();
        budgeter_service
            .expect_update_budgeter()
            .returning(move |_| Ok(budgeter_cloned.clone()));

        let app = get_budgeter_routes(Arc::new(budgeter_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/budgeter/{:?}", budgeter.id))
                    .method("PUT")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_vec(&budgeter).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: BudgeterConfig = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, budgeter);
    }

    #[tokio::test]
    async fn update_budgeter_error_500() {
        let budgeter: BudgeterConfig = Faker.fake();

        let mut budgeter_service = MockBudgeterServiceExt::new();
        budgeter_service
            .expect_update_budgeter()
            .returning(move |_| {
                Err(AppError::InternalServerError(Into::into(
                    sqlx::Error::RowNotFound,
                )))
            });

        let app = get_budgeter_routes(Arc::new(budgeter_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/budgeter/{:?}", budgeter.id))
                    .method("PUT")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_vec(&budgeter).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn update_budgeter_error_400_invalid_uuid_format_in_path() {
        let budgeter: BudgeterConfig = Faker.fake();

        let budgeter_service = MockBudgeterServiceExt::new();

        let app = get_budgeter_routes(Arc::new(budgeter_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/budgeter/{:?}", Faker.fake::<i64>()))
                    .method("PUT")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_vec(&budgeter).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn update_budgeter_error_422_wrong_body_attribute() {
        #[derive(Debug, Clone, Serialize, Dummy)]
        struct Body {
            id: Uuid,
            nameeeeeeeeeeeeee: String,
            payee_ids: Vec<Uuid>,
        }
        let body: Body = Faker.fake();

        let budgeter_service = MockBudgeterServiceExt::new();

        let app = get_budgeter_routes(Arc::new(budgeter_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/budgeter/{:?}", body.id))
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
    async fn update_budgeter_error_422_missing_body_attribute() {
        #[derive(Debug, Clone, Serialize, Dummy)]
        struct Body {
            id: Uuid,
            payee_ids: Vec<Uuid>,
        }
        let body: Body = Faker.fake();

        let budgeter_service = MockBudgeterServiceExt::new();

        let app = get_budgeter_routes(Arc::new(budgeter_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/budgeter/{:?}", body.id))
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
    async fn update_budgeter_error_422_wrong_body_attribute_type() {
        #[derive(Debug, Clone, Serialize, Dummy)]
        struct Body {
            id: Uuid,
            name: i64,
            payee_ids: Vec<Uuid>,
        }
        let body: Body = Faker.fake();

        let budgeter_service = MockBudgeterServiceExt::new();

        let app = get_budgeter_routes(Arc::new(budgeter_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/budgeter/{:?}", body.id))
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
    async fn update_budgeter_error_400_empty_body() {
        let budgeter_service = MockBudgeterServiceExt::new();

        let app = get_budgeter_routes(Arc::new(budgeter_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/budgeter/{:?}", Faker.fake::<Uuid>()))
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
    async fn update_budgeter_error_415_missing_json_content_type() {
        let budgeter: BudgeterConfig = Faker.fake();

        let budgeter_service = MockBudgeterServiceExt::new();

        let app = get_budgeter_routes(Arc::new(budgeter_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/budgeter/{:?}", budgeter.id))
                    .method("PUT")
                    .body(serde_json::to_vec(&budgeter).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }

    #[tokio::test]
    async fn update_budgeter_error_405_wrong_http_method() {
        let budgeter: BudgeterConfig = Faker.fake();

        let budgeter_service = MockBudgeterServiceExt::new();

        let app = get_budgeter_routes(Arc::new(budgeter_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/budgeter/{:?}", budgeter.id))
                    .method("POST")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_vec(&budgeter).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn update_budgeter_error_404_when_not_found() {
        let budgeter: BudgeterConfig = Faker.fake();

        let mut budgeter_service = MockBudgeterServiceExt::new();

        budgeter_service
            .expect_update_budgeter()
            .returning(|_| Err(AppError::ResourceNotFound));

        let app = get_budgeter_routes(Arc::new(budgeter_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/budgeter/{:?}", budgeter.id))
                    .method("PUT")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_vec(&budgeter).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn delete_budgeter_success() {
        let budgeter: BudgeterConfig = Faker.fake();

        let mut budgeter_service = MockBudgeterServiceExt::new();
        let budgeter_cloned = budgeter.clone();
        budgeter_service
            .expect_delete_budgeter()
            .returning(move |_| Ok(budgeter_cloned.clone()));

        let app = get_budgeter_routes(Arc::new(budgeter_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/budgeter/{:?}", budgeter.id))
                    .method("DELETE")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: BudgeterConfig = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, budgeter);
    }

    #[tokio::test]
    async fn delete_budgeter_error_500() {
        let mut budgeter_service = MockBudgeterServiceExt::new();
        budgeter_service
            .expect_delete_budgeter()
            .returning(move |_| {
                Err(AppError::InternalServerError(Into::into(
                    sqlx::Error::RowNotFound,
                )))
            });

        let app = get_budgeter_routes(Arc::new(budgeter_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/budgeter/{:?}", Faker.fake::<Uuid>()))
                    .method("DELETE")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn delete_budgeter_error_404_non_existing_resource() {
        let mut budgeter_service = MockBudgeterServiceExt::new();
        budgeter_service
            .expect_delete_budgeter()
            .returning(move |_| Err(AppError::ResourceNotFound));

        let app = get_budgeter_routes(Arc::new(budgeter_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/budgeter/{:?}", Faker.fake::<Uuid>()))
                    .method("DELETE")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn delete_budgeter_error_400_invalid_uuid_format_in_path() {
        let budgeter_service = MockBudgeterServiceExt::new();

        let app = get_budgeter_routes(Arc::new(budgeter_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/budgeter/{:?}", Faker.fake::<i64>()))
                    .method("DELETE")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
