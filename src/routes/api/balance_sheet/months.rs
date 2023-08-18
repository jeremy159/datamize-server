use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_extra::extract::WithRejection;

use crate::{
    error::{AppError, HttpJsonDatamizeResult, JsonError},
    models::balance_sheet::{Month, SaveMonth},
    services::balance_sheet::DynMonthService,
};

/// Returns all months of all years.
#[tracing::instrument(name = "Get all months from all years", skip_all)]
pub async fn all_balance_sheet_months(
    State(month_service): State<DynMonthService>,
) -> HttpJsonDatamizeResult<Vec<Month>> {
    Ok(Json(month_service.get_all_months().await?))
}

/// Returns all the months within a year with balance sheets.
#[tracing::instrument(name = "Get all months from a year", skip_all)]
pub async fn balance_sheet_months(
    Path(year): Path<i32>,
    State(month_service): State<DynMonthService>,
) -> HttpJsonDatamizeResult<Vec<Month>> {
    Ok(Json(month_service.get_all_months_from_year(year).await?))
}

/// Creates a new month if it doesn't already exist and returns the newly created entity.
/// Will also update net totals for this month compared to previous one if any.
#[tracing::instrument(skip_all)]
pub async fn create_balance_sheet_month(
    Path(year): Path<i32>,
    State(month_service): State<DynMonthService>,
    WithRejection(Json(body), _): WithRejection<Json<SaveMonth>, JsonError>,
) -> impl IntoResponse {
    Ok::<_, AppError>((
        StatusCode::CREATED,
        Json(month_service.create_month(year, body).await?),
    ))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use chrono::{Datelike, NaiveDate};
    use fake::{faker::chrono::en::Date, Dummy, Fake, Faker};
    use serde::Serialize;
    use tower::ServiceExt;
    use uuid::Uuid;

    use crate::{
        models::balance_sheet::MonthNum, routes::api::balance_sheet::get_month_routes,
        services::balance_sheet::MockMonthServiceExt,
    };

    use super::*;

    #[tokio::test]
    async fn all_balance_sheet_months_success() {
        let months: Vec<Month> = Faker.fake();

        let mut month_service = MockMonthServiceExt::new();
        let months_cloned = months.clone();
        month_service
            .expect_get_all_months()
            .returning(move || Ok(months_cloned.clone()));

        let app = get_month_routes(Arc::new(month_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/months")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: Vec<Month> = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, months);
    }

    #[tokio::test]
    async fn all_balance_sheet_months_error_500() {
        let mut month_service = MockMonthServiceExt::new();
        month_service.expect_get_all_months().returning(|| {
            Err(AppError::InternalServerError(Into::into(
                sqlx::Error::RowNotFound,
            )))
        });

        let app = get_month_routes(Arc::new(month_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/months")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn balance_sheet_months_success() {
        let mut months: Vec<Month> = Faker.fake();
        let year = Date().fake::<NaiveDate>().year();
        months.iter_mut().for_each(|m| {
            m.year = year;
        });

        let mut month_service = MockMonthServiceExt::new();
        let months_cloned = months.clone();
        month_service
            .expect_get_all_months_from_year()
            .returning(move |_| Ok(months_cloned.clone()));

        let app = get_month_routes(Arc::new(month_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/years/{:?}/months", year))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: Vec<Month> = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, months);
    }

    #[tokio::test]
    async fn balance_sheet_months_error_500() {
        let mut month_service = MockMonthServiceExt::new();
        month_service
            .expect_get_all_months_from_year()
            .returning(|_| {
                Err(AppError::InternalServerError(Into::into(
                    sqlx::Error::RowNotFound,
                )))
            });

        let app = get_month_routes(Arc::new(month_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/years/{:?}/months",
                        Date().fake::<NaiveDate>().year()
                    ))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn create_balance_sheet_month_success() {
        let new_month = Faker.fake::<SaveMonth>();
        let year = Date().fake::<NaiveDate>().year();

        let mut month_service = MockMonthServiceExt::new();
        let month = Month::new(new_month.month, year);
        let month_cloned = month.clone();
        month_service
            .expect_create_month()
            .returning(move |_, _| Ok(month_cloned.clone()));

        let app = get_month_routes(Arc::new(month_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/years/{:?}/months", year))
                    .method("POST")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_vec(&new_month).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: Month = serde_json::from_slice(&body).unwrap();
        assert_eq!(body.year, month.year);
        assert_eq!(body.month, month.month);
        assert_eq!(body.net_assets, month.net_assets);
        assert_eq!(body.net_portfolio, month.net_portfolio);
        assert_eq!(body.resources, month.resources);
    }

    #[tokio::test]
    async fn create_balance_sheet_month_error_500() {
        let new_month = Faker.fake::<SaveMonth>();

        let mut month_service = MockMonthServiceExt::new();
        month_service.expect_create_month().returning(move |_, _| {
            Err(AppError::InternalServerError(Into::into(
                sqlx::Error::RowNotFound,
            )))
        });

        let app = get_month_routes(Arc::new(month_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/years/{:?}/months",
                        Date().fake::<NaiveDate>().year()
                    ))
                    .method("POST")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_vec(&new_month).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn create_balance_sheet_month_error_422_wrong_body_attribute() {
        #[derive(Debug, Clone, Serialize, Dummy)]
        struct Body {
            monthhhh: MonthNum,
        }
        let body: Body = Faker.fake();

        let month_service = MockMonthServiceExt::new();

        let app = get_month_routes(Arc::new(month_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/years/{:?}/months",
                        Date().fake::<NaiveDate>().year()
                    ))
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
    async fn create_balance_sheet_month_error_422_missing_body_attribute() {
        #[derive(Debug, Clone, Serialize, Dummy)]
        struct Body {
            id: Uuid,
        }
        let body: Body = Faker.fake();

        let month_service = MockMonthServiceExt::new();

        let app = get_month_routes(Arc::new(month_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/years/{:?}/months",
                        Date().fake::<NaiveDate>().year()
                    ))
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
    async fn create_balance_sheet_month_error_422_wrong_body_attribute_type() {
        #[derive(Debug, Clone, Serialize, Dummy)]
        struct Body {
            month: String,
        }
        let body: Body = Faker.fake();

        let month_service = MockMonthServiceExt::new();

        let app = get_month_routes(Arc::new(month_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/years/{:?}/months",
                        Date().fake::<NaiveDate>().year()
                    ))
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
    async fn create_balance_sheet_month_error_400_empty_body() {
        let month_service = MockMonthServiceExt::new();

        let app = get_month_routes(Arc::new(month_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/years/{:?}/months",
                        Date().fake::<NaiveDate>().year()
                    ))
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
    async fn create_balance_sheet_month_error_415_missing_json_content_type() {
        let new_month: SaveMonth = Faker.fake();

        let month_service = MockMonthServiceExt::new();

        let app = get_month_routes(Arc::new(month_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/years/{:?}/months",
                        Date().fake::<NaiveDate>().year()
                    ))
                    .method("POST")
                    .body(serde_json::to_vec(&new_month).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }

    #[tokio::test]
    async fn create_balance_sheet_month_error_405_wrong_http_method() {
        let new_month: SaveMonth = Faker.fake();

        let month_service = MockMonthServiceExt::new();

        let app = get_month_routes(Arc::new(month_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/years/{:?}/months",
                        Date().fake::<NaiveDate>().year()
                    ))
                    .method("PUT")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_vec(&new_month).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn create_balance_sheet_month_error_409_when_already_exists() {
        let new_month: SaveMonth = Faker.fake();

        let mut month_service = MockMonthServiceExt::new();

        month_service
            .expect_create_month()
            .returning(|_, _| Err(AppError::ResourceAlreadyExist));

        let app = get_month_routes(Arc::new(month_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/years/{:?}/months",
                        Date().fake::<NaiveDate>().year()
                    ))
                    .method("POST")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_vec(&new_month).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CONFLICT);
    }
}
