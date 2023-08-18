use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_extra::extract::WithRejection;

use crate::{
    error::{AppError, HttpJsonDatamizeResult, JsonError},
    models::balance_sheet::{SaveYear, YearSummary},
    services::balance_sheet::DynYearService,
};

/// Returns a summary of all the years with balance sheets.
#[tracing::instrument(name = "Get a summary of all years", skip_all)]
pub async fn balance_sheet_years(
    State(year_service): State<DynYearService>,
) -> HttpJsonDatamizeResult<Vec<YearSummary>> {
    Ok(Json(year_service.get_all_years().await?))
}

/// Creates a new year if it doesn't already exist and returns the newly created entity.
/// Will also update net totals for this year compared to previous one if any.
#[tracing::instrument(skip_all)]
pub async fn create_balance_sheet_year(
    State(year_service): State<DynYearService>,
    WithRejection(Json(body), _): WithRejection<Json<SaveYear>, JsonError>,
) -> impl IntoResponse {
    Ok::<_, AppError>((
        StatusCode::CREATED,
        Json(year_service.create_year(body).await?),
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
        error::AppError, models::balance_sheet::YearDetail,
        routes::api::balance_sheet::get_year_routes, services::balance_sheet::MockYearServiceExt,
    };

    use super::*;

    #[tokio::test]
    async fn balance_sheet_years_success() {
        let years: Vec<YearSummary> = Faker.fake();

        let mut year_service = MockYearServiceExt::new();
        let years_cloned = years.clone();
        year_service
            .expect_get_all_years()
            .returning(move || Ok(years_cloned.clone()));

        let app = get_year_routes(Arc::new(year_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/years")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: Vec<YearSummary> = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, years);
    }

    #[tokio::test]
    async fn balance_sheet_years_error_500() {
        let mut year_service = MockYearServiceExt::new();
        year_service.expect_get_all_years().returning(|| {
            Err(AppError::InternalServerError(Into::into(
                sqlx::Error::RowNotFound,
            )))
        });

        let app = get_year_routes(Arc::new(year_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/years")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn create_balance_sheet_year_success() {
        let new_year = Faker.fake::<SaveYear>();

        let mut year_service = MockYearServiceExt::new();
        let year = YearDetail::new(new_year.year);
        year_service
            .expect_create_year()
            .returning(move |_| Ok(year.clone()));

        let app = get_year_routes(Arc::new(year_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/years")
                    .method("POST")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_vec(&new_year).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: YearDetail = serde_json::from_slice(&body).unwrap();
        assert_eq!(body.year, new_year.year);
    }

    #[tokio::test]
    async fn create_balance_sheet_year_error_500() {
        let new_year = Faker.fake::<SaveYear>();

        let mut year_service = MockYearServiceExt::new();
        year_service.expect_create_year().returning(move |_| {
            Err(AppError::InternalServerError(Into::into(
                sqlx::Error::RowNotFound,
            )))
        });

        let app = get_year_routes(Arc::new(year_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/years")
                    .method("POST")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_vec(&new_year).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn create_balance_sheet_year_error_422_wrong_body_attribute() {
        #[derive(Debug, Clone, Serialize, Dummy)]
        struct Body {
            yearrrrrrrrr: i32,
        }
        let body: Body = Faker.fake();

        let year_service = MockYearServiceExt::new();

        let app = get_year_routes(Arc::new(year_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/years")
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
    async fn create_balance_sheet_year_error_422_missing_body_attribute() {
        #[derive(Debug, Clone, Serialize, Dummy)]
        struct Body {
            id: Uuid,
        }
        let body: Body = Faker.fake();

        let year_service = MockYearServiceExt::new();

        let app = get_year_routes(Arc::new(year_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/years")
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
    async fn create_balance_sheet_year_error_422_wrong_body_attribute_type() {
        #[derive(Debug, Clone, Serialize, Dummy)]
        struct Body {
            year: String,
        }
        let body: Body = Faker.fake();

        let year_service = MockYearServiceExt::new();

        let app = get_year_routes(Arc::new(year_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/years")
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
    async fn create_balance_sheet_year_error_400_empty_body() {
        let year_service = MockYearServiceExt::new();

        let app = get_year_routes(Arc::new(year_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/years")
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
    async fn create_balance_sheet_year_error_415_missing_json_content_type() {
        let new_year: SaveYear = Faker.fake();

        let year_service = MockYearServiceExt::new();

        let app = get_year_routes(Arc::new(year_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/years")
                    .method("POST")
                    .body(serde_json::to_vec(&new_year).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }

    #[tokio::test]
    async fn create_balance_sheet_year_error_405_wrong_http_method() {
        let new_year: SaveYear = Faker.fake();

        let year_service = MockYearServiceExt::new();

        let app = get_year_routes(Arc::new(year_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/years")
                    .method("PUT")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_vec(&new_year).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn create_balance_sheet_year_error_409_when_already_exists() {
        let new_year: SaveYear = Faker.fake();

        let mut year_service = MockYearServiceExt::new();

        year_service
            .expect_create_year()
            .returning(|_| Err(AppError::ResourceAlreadyExist));

        let app = get_year_routes(Arc::new(year_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/years")
                    .method("POST")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_vec(&new_year).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CONFLICT);
    }
}
