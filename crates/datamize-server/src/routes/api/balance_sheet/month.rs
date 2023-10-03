use axum::{
    extract::{Path, State},
    Json,
};
use datamize_domain::{Month, MonthNum};

use crate::{error::HttpJsonDatamizeResult, services::balance_sheet::DynMonthService};

/// Returns a specific month with its financial resources and net totals.
#[tracing::instrument(name = "Get a month", skip_all)]
pub async fn balance_sheet_month(
    Path((year, month)): Path<(i32, MonthNum)>,
    State(month_service): State<DynMonthService>,
) -> HttpJsonDatamizeResult<Month> {
    Ok(Json(month_service.get_month(month, year).await?))
}

/// Deletes the month and returns the entity
#[tracing::instrument(skip_all)]
pub async fn delete_balance_sheet_month(
    Path((year, month)): Path<(i32, MonthNum)>,
    State(month_service): State<DynMonthService>,
) -> HttpJsonDatamizeResult<Month> {
    Ok(Json(month_service.delete_month(month, year).await?))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use chrono::{Datelike, NaiveDate};
    use datamize_domain::Uuid;
    use fake::{faker::chrono::en::Date, Fake, Faker};
    use tower::ServiceExt;

    use crate::{
        error::AppError, routes::api::balance_sheet::get_month_routes,
        services::balance_sheet::MockMonthServiceExt,
    };

    use super::*;

    #[tokio::test]
    async fn balance_sheet_month_success() {
        let date = Date().fake::<NaiveDate>();
        let month = Month {
            month: date.month().try_into().unwrap(),
            year: date.year(),
            ..Faker.fake()
        };

        let mut month_service = MockMonthServiceExt::new();
        let month_cloned = month.clone();
        month_service
            .expect_get_month()
            .returning(move |_, _| Ok(month_cloned.clone()));

        let app = get_month_routes(Arc::new(month_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/years/{:?}/months/{:?}", month.year, date.month()))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: Month = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, month);
    }

    #[tokio::test]
    async fn balance_sheet_month_error_500() {
        let mut month_service = MockMonthServiceExt::new();
        month_service.expect_get_month().returning(move |_, _| {
            Err(AppError::InternalServerError(Into::into(
                sqlx::Error::RowNotFound,
            )))
        });

        let app = get_month_routes(Arc::new(month_service));
        let date = Date().fake::<NaiveDate>();
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/years/{:?}/months/{:?}",
                        date.year(),
                        date.month()
                    ))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn balance_sheet_month_error_404_non_existing_resource() {
        let mut month_service = MockMonthServiceExt::new();
        month_service
            .expect_get_month()
            .returning(move |_, _| Err(AppError::ResourceNotFound));

        let app = get_month_routes(Arc::new(month_service));
        let date = Date().fake::<NaiveDate>();
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/years/{:?}/months/{:?}",
                        date.year(),
                        date.month()
                    ))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn balance_sheet_month_error_400_invalid_year_i32_format_in_path() {
        let month_service = MockMonthServiceExt::new();

        let app = get_month_routes(Arc::new(month_service));
        let date = Date().fake::<NaiveDate>();
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/years/{:?}/months/{:?}",
                        Faker.fake::<Uuid>(),
                        date.month()
                    ))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn balance_sheet_month_error_400_invalid_month_i16_format_in_path() {
        let month_service = MockMonthServiceExt::new();

        let app = get_month_routes(Arc::new(month_service));
        let date = Date().fake::<NaiveDate>();
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/years/{:?}/months/{:?}",
                        date.year(),
                        Faker.fake::<Uuid>()
                    ))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn delete_balance_sheet_month_success() {
        let month: Month = Faker.fake();

        let mut month_service = MockMonthServiceExt::new();
        let month_cloned = month.clone();
        month_service
            .expect_delete_month()
            .returning(move |_, _| Ok(month_cloned.clone()));

        let app = get_month_routes(Arc::new(month_service));
        let date = Date().fake::<NaiveDate>();
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/years/{:?}/months/{:?}",
                        date.year(),
                        date.month()
                    ))
                    .method("DELETE")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: Month = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, month);
    }

    #[tokio::test]
    async fn delete_balance_sheet_month_error_500() {
        let mut month_service = MockMonthServiceExt::new();
        month_service.expect_delete_month().returning(move |_, _| {
            Err(AppError::InternalServerError(Into::into(
                sqlx::Error::RowNotFound,
            )))
        });

        let app = get_month_routes(Arc::new(month_service));
        let date = Date().fake::<NaiveDate>();
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/years/{:?}/months/{:?}",
                        date.year(),
                        date.month()
                    ))
                    .method("DELETE")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn delete_balance_sheet_month_error_404_non_existing_resource() {
        let mut month_service = MockMonthServiceExt::new();
        month_service
            .expect_delete_month()
            .returning(move |_, _| Err(AppError::ResourceNotFound));

        let app = get_month_routes(Arc::new(month_service));
        let date = Date().fake::<NaiveDate>();
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/years/{:?}/months/{:?}",
                        date.year(),
                        date.month()
                    ))
                    .method("DELETE")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn delete_balance_sheet_month_error_400_invalid_year_i32_format_in_path() {
        let month_service = MockMonthServiceExt::new();

        let app = get_month_routes(Arc::new(month_service));
        let date = Date().fake::<NaiveDate>();
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/years/{:?}/months/{:?}",
                        Faker.fake::<Uuid>(),
                        date.month()
                    ))
                    .method("DELETE")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn delete_balance_sheet_month_error_400_invalid_month_i16_format_in_path() {
        let month_service = MockMonthServiceExt::new();

        let app = get_month_routes(Arc::new(month_service));
        let date = Date().fake::<NaiveDate>();
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/years/{:?}/months/{:?}",
                        date.year(),
                        Faker.fake::<Uuid>()
                    ))
                    .method("DELETE")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
