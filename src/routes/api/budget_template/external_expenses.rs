use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_extra::extract::WithRejection;

use crate::{
    error::{DatamizeResult, HttpJsonDatamizeResult, JsonError},
    models::budget_template::{ExternalExpense, SaveExternalExpense},
    services::budget_template::DynExternalExpenseService,
};

/// Returns all external_expenses.
#[tracing::instrument(skip_all)]
pub async fn get_all_external_expenses(
    State(external_expense_service): State<DynExternalExpenseService>,
) -> HttpJsonDatamizeResult<Vec<ExternalExpense>> {
    Ok(Json(
        external_expense_service.get_all_external_expenses().await?,
    ))
}

/// Creates a new budgeter if it doesn't already exist and returns the newly created entity.
#[tracing::instrument(skip_all)]
pub async fn create_external_expense(
    State(external_expense_service): State<DynExternalExpenseService>,
    WithRejection(Json(body), _): WithRejection<Json<SaveExternalExpense>, JsonError>,
) -> DatamizeResult<impl IntoResponse> {
    Ok((
        StatusCode::CREATED,
        Json(
            external_expense_service
                .create_external_expense(body)
                .await?,
        ),
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
        error::AppError,
        models::budget_template::{ExpenseType, SubExpenseType},
        routes::api::budget_template::get_external_expense_routes,
        services::budget_template::MockExternalExpenseServiceExt,
    };

    use super::*;

    #[tokio::test]
    async fn get_all_external_expenses_success() {
        let external_expenses: Vec<ExternalExpense> = Faker.fake();

        let mut external_expense_service = MockExternalExpenseServiceExt::new();
        let external_expenses_cloned = external_expenses.clone();
        external_expense_service
            .expect_get_all_external_expenses()
            .returning(move || Ok(external_expenses_cloned.clone()));

        let app = get_external_expense_routes(Arc::new(external_expense_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/external_expenses")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: Vec<ExternalExpense> = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, external_expenses);
    }

    #[tokio::test]
    async fn get_all_external_expenses_error_500() {
        let mut external_expense_service = MockExternalExpenseServiceExt::new();
        external_expense_service
            .expect_get_all_external_expenses()
            .returning(|| {
                Err(AppError::InternalServerError(Into::into(
                    sqlx::Error::RowNotFound,
                )))
            });

        let app = get_external_expense_routes(Arc::new(external_expense_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/external_expenses")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn create_external_expense_success() {
        let new_external_expense = Faker.fake::<SaveExternalExpense>();

        let mut external_expense_service = MockExternalExpenseServiceExt::new();
        let external_expense_cloned: ExternalExpense = new_external_expense.clone().into();
        external_expense_service
            .expect_create_external_expense()
            .returning(move |_| Ok(external_expense_cloned.clone()));

        let app = get_external_expense_routes(Arc::new(external_expense_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/external_expense")
                    .method("POST")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_vec(&new_external_expense).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: ExternalExpense = serde_json::from_slice(&body).unwrap();
        assert_eq!(body.name, new_external_expense.name);
        assert_eq!(body.expense_type, new_external_expense.expense_type);
        assert_eq!(body.sub_expense_type, new_external_expense.sub_expense_type);
        assert_eq!(body.projected_amount, new_external_expense.projected_amount);
    }

    #[tokio::test]
    async fn create_external_expense_error_500() {
        let new_external_expense = Faker.fake::<SaveExternalExpense>();

        let mut external_expense_service = MockExternalExpenseServiceExt::new();
        external_expense_service
            .expect_create_external_expense()
            .returning(move |_| {
                Err(AppError::InternalServerError(Into::into(
                    sqlx::Error::RowNotFound,
                )))
            });

        let app = get_external_expense_routes(Arc::new(external_expense_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/external_expense")
                    .method("POST")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_vec(&new_external_expense).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn create_external_expense_error_422_wrong_body_attribute() {
        #[derive(Debug, Clone, Serialize, Dummy)]
        struct Body {
            nameeeeeeeeeeeeee: String,
            #[serde(rename = "type")]
            expense_type: ExpenseType,
            #[serde(rename = "sub_type")]
            sub_expense_type: SubExpenseType,
            projected_amount: i64,
        }
        let body: Body = Faker.fake();

        let external_expense_service = MockExternalExpenseServiceExt::new();

        let app = get_external_expense_routes(Arc::new(external_expense_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/external_expense")
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
    async fn create_external_expense_error_422_missing_body_attribute() {
        #[derive(Debug, Clone, Serialize, Dummy)]
        struct Body {
            #[serde(rename = "type")]
            expense_type: ExpenseType,
            #[serde(rename = "sub_type")]
            sub_expense_type: SubExpenseType,
            projected_amount: i64,
        }
        let body: Body = Faker.fake();

        let external_expense_service = MockExternalExpenseServiceExt::new();

        let app = get_external_expense_routes(Arc::new(external_expense_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/external_expense")
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
    async fn create_external_expense_error_422_wrong_body_attribute_type() {
        #[derive(Debug, Clone, Serialize, Dummy)]
        struct Body {
            name: i64,
            #[serde(rename = "type")]
            expense_type: ExpenseType,
            #[serde(rename = "sub_type")]
            sub_expense_type: SubExpenseType,
            projected_amount: i64,
        }
        let body: Body = Faker.fake();

        let external_expense_service = MockExternalExpenseServiceExt::new();

        let app = get_external_expense_routes(Arc::new(external_expense_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/external_expense")
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
    async fn create_external_expense_error_400_empty_body() {
        let external_expense_service = MockExternalExpenseServiceExt::new();

        let app = get_external_expense_routes(Arc::new(external_expense_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/external_expense")
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
    async fn create_external_expense_error_415_missing_json_content_type() {
        let new_external_expense: SaveExternalExpense = Faker.fake();

        let external_expense_service = MockExternalExpenseServiceExt::new();

        let app = get_external_expense_routes(Arc::new(external_expense_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/external_expense")
                    .method("POST")
                    .body(serde_json::to_vec(&new_external_expense).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }

    #[tokio::test]
    async fn create_external_expense_error_405_wrong_http_method() {
        let new_external_expense: SaveExternalExpense = Faker.fake();

        let external_expense_service = MockExternalExpenseServiceExt::new();

        let app = get_external_expense_routes(Arc::new(external_expense_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/external_expense")
                    .method("PUT")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_vec(&new_external_expense).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn create_external_expense_error_409_when_already_exists() {
        let new_external_expense: SaveExternalExpense = Faker.fake();

        let mut external_expense_service = MockExternalExpenseServiceExt::new();

        external_expense_service
            .expect_create_external_expense()
            .returning(|_| Err(AppError::ResourceAlreadyExist));

        let app = get_external_expense_routes(Arc::new(external_expense_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/external_expense")
                    .method("POST")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_vec(&new_external_expense).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CONFLICT);
    }
}
