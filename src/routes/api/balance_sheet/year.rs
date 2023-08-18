use axum::{
    extract::{Path, State},
    Json,
};
use axum_extra::extract::WithRejection;

use crate::{
    error::{HttpJsonDatamizeResult, JsonError},
    models::balance_sheet::{UpdateYear, YearDetail},
    services::balance_sheet::DynYearService,
};

/// Returns a detailed year with its balance sheet, its saving rates, its months and its financial resources.
#[tracing::instrument(name = "Get a detailed year", skip_all)]
pub async fn balance_sheet_year(
    Path(year): Path<i32>,
    State(year_service): State<DynYearService>,
) -> HttpJsonDatamizeResult<YearDetail> {
    Ok(Json(year_service.get_year(year).await?))
}

/// Updates the saving rates of the received year.
#[tracing::instrument(skip_all)]
pub async fn update_balance_sheet_year(
    Path(year): Path<i32>,
    State(year_service): State<DynYearService>,
    WithRejection(Json(body), _): WithRejection<Json<UpdateYear>, JsonError>,
) -> HttpJsonDatamizeResult<YearDetail> {
    Ok(Json(year_service.update_year(year, body).await?))
}

/// Deletes the year and returns the entity.
#[tracing::instrument(skip_all)]
pub async fn delete_balance_sheet_year(
    Path(year): Path<i32>,
    State(year_service): State<DynYearService>,
) -> HttpJsonDatamizeResult<YearDetail> {
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
    use fake::{faker::chrono::en::Date, Dummy, Fake, Faker};
    use serde::Serialize;
    use tower::ServiceExt;
    use uuid::Uuid;

    use crate::{
        error::AppError, models::balance_sheet::SavingRatesPerPerson,
        routes::api::balance_sheet::get_year_routes, services::balance_sheet::MockYearServiceExt,
    };

    use super::*;

    #[tokio::test]
    async fn balance_sheet_year_success() {
        let year: YearDetail = Faker.fake();

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
        let body: YearDetail = serde_json::from_slice(&body).unwrap();
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
    async fn update_balance_sheet_year_success() {
        let year_update: UpdateYear = Faker.fake();
        let year = YearDetail {
            saving_rates: year_update.saving_rates.clone(),
            year: Date().fake::<NaiveDate>().year(),
            ..Faker.fake()
        };

        let mut year_service = MockYearServiceExt::new();
        let year_cloned = year.clone();
        year_service
            .expect_update_year()
            .returning(move |_, _| Ok(year_cloned.clone()));

        let app = get_year_routes(Arc::new(year_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/years/{:?}", year.year))
                    .method("PUT")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_vec(&year_update).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: YearDetail = serde_json::from_slice(&body).unwrap();
        assert_eq!(body.saving_rates, year.saving_rates);
    }

    #[tokio::test]
    async fn update_balance_sheet_year_error_500() {
        let year_update: UpdateYear = Faker.fake();

        let mut year_service = MockYearServiceExt::new();
        year_service.expect_update_year().returning(move |_, _| {
            Err(AppError::InternalServerError(Into::into(
                sqlx::Error::RowNotFound,
            )))
        });

        let app = get_year_routes(Arc::new(year_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/years/{:?}", Date().fake::<NaiveDate>().year()))
                    .method("PUT")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_vec(&year_update).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn update_balance_sheet_year_error_400_invalid_i32_format_in_path() {
        let year_update: UpdateYear = Faker.fake();

        let year_service = MockYearServiceExt::new();

        let app = get_year_routes(Arc::new(year_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/years/{:?}", Faker.fake::<Uuid>()))
                    .method("PUT")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_vec(&year_update).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn update_balance_sheet_year_error_422_wrong_body_attribute() {
        #[derive(Debug, Clone, Serialize, Dummy)]
        struct Body {
            saving_rateeeeeeeeees: Vec<SavingRatesPerPerson>,
        }
        let body: Body = Faker.fake();

        let year_service = MockYearServiceExt::new();

        let app = get_year_routes(Arc::new(year_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/years/{:?}", Date().fake::<NaiveDate>().year()))
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
    async fn update_balance_sheet_year_error_422_missing_body_attribute() {
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
                    .uri(format!("/years/{:?}", Date().fake::<NaiveDate>().year()))
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
    async fn update_balance_sheet_year_error_422_wrong_body_attribute_type() {
        #[derive(Debug, Clone, Serialize, Dummy)]
        struct Body {
            saving_rates: Vec<Uuid>,
        }
        let body: Body = Faker.fake();

        let year_service = MockYearServiceExt::new();

        let app = get_year_routes(Arc::new(year_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/years/{:?}", Date().fake::<NaiveDate>().year()))
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
    async fn update_balance_sheet_year_error_400_empty_body() {
        let year_service = MockYearServiceExt::new();

        let app = get_year_routes(Arc::new(year_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/years/{:?}", Date().fake::<NaiveDate>().year()))
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
    async fn update_balance_sheet_year_error_415_missing_json_content_type() {
        let year_update: UpdateYear = Faker.fake();

        let year_service = MockYearServiceExt::new();

        let app = get_year_routes(Arc::new(year_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/years/{:?}", Date().fake::<NaiveDate>().year()))
                    .method("PUT")
                    .body(serde_json::to_vec(&year_update).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }

    #[tokio::test]
    async fn update_balance_sheet_year_error_405_wrong_http_method() {
        let year_update: UpdateYear = Faker.fake();

        let year_service = MockYearServiceExt::new();

        let app = get_year_routes(Arc::new(year_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/years/{:?}", Date().fake::<NaiveDate>().year()))
                    .method("POST")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_vec(&year_update).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn update_balance_sheet_year_error_404_when_not_found() {
        let year_update: UpdateYear = Faker.fake();

        let mut year_service = MockYearServiceExt::new();

        year_service
            .expect_update_year()
            .returning(|_, _| Err(AppError::ResourceNotFound));

        let app = get_year_routes(Arc::new(year_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/years/{:?}", Date().fake::<NaiveDate>().year()))
                    .method("PUT")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_vec(&year_update).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn delete_balance_sheet_year_success() {
        let year: YearDetail = Faker.fake();

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
        let body: YearDetail = serde_json::from_slice(&body).unwrap();
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
