use axum::{
    extract::{Path, State},
    Json,
};

use crate::{
    error::HttpJsonDatamizeResult, models::balance_sheet::Year,
    services::balance_sheet::DynYearService,
};

/// Returns a detailed year with its balance sheet, its saving rates, its months and its financial resources.
#[tracing::instrument(name = "Get a detailed year", skip_all)]
pub async fn balance_sheet_year(
    Path(year): Path<i32>,
    State(year_service): State<DynYearService>,
) -> HttpJsonDatamizeResult<Year> {
    Ok(Json(year_service.get_year(year).await?))
}

/// Deletes the year and returns the entity.
#[tracing::instrument(skip_all)]
pub async fn delete_balance_sheet_year(
    Path(year): Path<i32>,
    State(year_service): State<DynYearService>,
) -> HttpJsonDatamizeResult<Year> {
    Ok(Json(year_service.delete_year(year).await?))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use chrono::{Datelike, NaiveDate};
    use fake::{faker::chrono::en::Date, Fake, Faker};
    use tower::ServiceExt;
    use uuid::Uuid;

    use crate::{
        error::AppError, routes::api::balance_sheet::get_year_routes,
        services::balance_sheet::MockYearServiceExt,
    };

    use super::*;

    #[tokio::test]
    async fn balance_sheet_year_success() {
        let year: Year = Faker.fake();

        let mut year_service = MockYearServiceExt::new();
        let year_cloned = year.clone();
        year_service
            .expect_get_year()
            .returning(move |_| Ok(year_cloned.clone()));

        let app = get_year_routes(Arc::new(year_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/years/{:?}", year.year))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: Year = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, year);
    }

    #[tokio::test]
    async fn balance_sheet_year_error_500() {
        let mut year_service = MockYearServiceExt::new();
        year_service.expect_get_year().returning(move |_| {
            Err(AppError::InternalServerError(Into::into(
                sqlx::Error::RowNotFound,
            )))
        });

        let app = get_year_routes(Arc::new(year_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/years/{:?}", Date().fake::<NaiveDate>().year()))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn balance_sheet_year_error_404_non_existing_resource() {
        let mut year_service = MockYearServiceExt::new();
        year_service
            .expect_get_year()
            .returning(move |_| Err(AppError::ResourceNotFound));

        let app = get_year_routes(Arc::new(year_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/years/{:?}", Date().fake::<NaiveDate>().year()))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn balance_sheet_year_error_400_invalid_i32_format_in_path() {
        let year_service = MockYearServiceExt::new();

        let app = get_year_routes(Arc::new(year_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/years/{:?}", Faker.fake::<Uuid>()))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn delete_balance_sheet_year_success() {
        let year: Year = Faker.fake();

        let mut year_service = MockYearServiceExt::new();
        let year_cloned = year.clone();
        year_service
            .expect_delete_year()
            .returning(move |_| Ok(year_cloned.clone()));

        let app = get_year_routes(Arc::new(year_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/years/{:?}", year.year))
                    .method("DELETE")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: Year = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, year);
    }

    #[tokio::test]
    async fn delete_balance_sheet_year_error_500() {
        let mut year_service = MockYearServiceExt::new();
        year_service.expect_delete_year().returning(move |_| {
            Err(AppError::InternalServerError(Into::into(
                sqlx::Error::RowNotFound,
            )))
        });

        let app = get_year_routes(Arc::new(year_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/years/{:?}", Date().fake::<NaiveDate>().year()))
                    .method("DELETE")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn delete_balance_sheet_year_error_404_non_existing_resource() {
        let mut year_service = MockYearServiceExt::new();
        year_service
            .expect_delete_year()
            .returning(move |_| Err(AppError::ResourceNotFound));

        let app = get_year_routes(Arc::new(year_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/years/{:?}", Date().fake::<NaiveDate>().year()))
                    .method("DELETE")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn delete_balance_sheet_year_error_400_invalid_i32_format_in_path() {
        let year_service = MockYearServiceExt::new();

        let app = get_year_routes(Arc::new(year_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/years/{:?}", Faker.fake::<Uuid>()))
                    .method("DELETE")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
