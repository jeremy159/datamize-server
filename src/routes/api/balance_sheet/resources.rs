use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_extra::extract::WithRejection;

use crate::{
    error::{AppError, HttpJsonDatamizeResult, JsonError},
    models::balance_sheet::{FinancialResourceYearly, SaveResource},
    services::balance_sheet::DynFinResService,
};

/// Returns all resources of all years.
#[tracing::instrument(name = "Get all resources from all years", skip_all)]
pub async fn all_balance_sheet_resources(
    State(fin_res_service): State<DynFinResService>,
) -> HttpJsonDatamizeResult<Vec<FinancialResourceYearly>> {
    Ok(Json(fin_res_service.get_all_fin_res().await?))
}

/// Endpoint to get all financial resources of a particular year.
#[tracing::instrument(name = "Get all resources from a year", skip_all)]
pub async fn balance_sheet_resources(
    Path(year): Path<i32>,
    State(fin_res_service): State<DynFinResService>,
) -> HttpJsonDatamizeResult<Vec<FinancialResourceYearly>> {
    Ok(Json(fin_res_service.get_all_fin_res_from_year(year).await?))
}

#[tracing::instrument(skip_all)]
pub async fn create_balance_sheet_resource(
    State(fin_res_service): State<DynFinResService>,
    WithRejection(Json(body), _): WithRejection<Json<SaveResource>, JsonError>,
) -> Result<impl IntoResponse, AppError> {
    Ok((
        StatusCode::CREATED,
        Json(fin_res_service.create_fin_res(body).await?),
    ))
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, sync::Arc};

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
        models::balance_sheet::{MonthNum, ResourceCategory, ResourceType},
        routes::api::balance_sheet::get_fin_res_routes,
        services::balance_sheet::MockFinResServiceExt,
    };

    use super::*;

    #[tokio::test]
    async fn all_balance_sheet_resources_success() {
        let resources: Vec<FinancialResourceYearly> = Faker.fake();

        let mut fin_res_service = MockFinResServiceExt::new();
        let resources_cloned = resources.clone();
        fin_res_service
            .expect_get_all_fin_res()
            .returning(move || Ok(resources_cloned.clone()));

        let app = get_fin_res_routes(Arc::new(fin_res_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/resources")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: Vec<FinancialResourceYearly> = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, resources);
    }

    #[tokio::test]
    async fn all_balance_sheet_resources_error_500() {
        let mut fin_res_service = MockFinResServiceExt::new();
        fin_res_service.expect_get_all_fin_res().returning(|| {
            Err(AppError::InternalServerError(Into::into(
                sqlx::Error::RowNotFound,
            )))
        });

        let app = get_fin_res_routes(Arc::new(fin_res_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/resources")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn balance_sheet_resources_success() {
        let mut resources: Vec<FinancialResourceYearly> = Faker.fake();
        let year = Date().fake::<NaiveDate>().year();
        resources.iter_mut().for_each(|r| {
            r.year = year;
        });

        let mut fin_res_service = MockFinResServiceExt::new();
        let resources_cloned = resources.clone();
        fin_res_service
            .expect_get_all_fin_res_from_year()
            .returning(move |_| Ok(resources_cloned.clone()));

        let app = get_fin_res_routes(Arc::new(fin_res_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/years/{:?}/resources", year))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: Vec<FinancialResourceYearly> = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, resources);
    }

    #[tokio::test]
    async fn balance_sheet_resources_error_500() {
        let mut fin_res_service = MockFinResServiceExt::new();
        fin_res_service
            .expect_get_all_fin_res_from_year()
            .returning(|_| {
                Err(AppError::InternalServerError(Into::into(
                    sqlx::Error::RowNotFound,
                )))
            });

        let app = get_fin_res_routes(Arc::new(fin_res_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/years/{:?}/resources",
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
    async fn create_balance_sheet_resource_success() {
        let new_res = Faker.fake::<SaveResource>();

        let mut fin_res_service = MockFinResServiceExt::new();
        let res: FinancialResourceYearly = new_res.clone().into();
        let res_cloned = res.clone();
        fin_res_service
            .expect_create_fin_res()
            .returning(move |_| Ok(res_cloned.clone()));

        let app = get_fin_res_routes(Arc::new(fin_res_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/resources")
                    .method("POST")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_vec(&new_res).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: FinancialResourceYearly = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, res);
    }

    #[tokio::test]
    async fn create_balance_sheet_resource_error_500() {
        let new_res = Faker.fake::<SaveResource>();

        let mut fin_res_service = MockFinResServiceExt::new();
        fin_res_service.expect_create_fin_res().returning(move |_| {
            Err(AppError::InternalServerError(Into::into(
                sqlx::Error::RowNotFound,
            )))
        });

        let app = get_fin_res_routes(Arc::new(fin_res_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/resources")
                    .method("POST")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_vec(&new_res).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn create_balance_sheet_resource_error_422_wrong_body_attribute() {
        #[derive(Debug, Clone, Serialize, Dummy)]
        struct Body {
            nameeeeeeeeeeeee: String,
            category: ResourceCategory,
            #[serde(rename = "type")]
            r_type: ResourceType,
            editable: bool,
            year: i32,
            balance_per_month: BTreeMap<MonthNum, i64>,
            ynab_account_ids: Option<Vec<Uuid>>,
            external_account_ids: Option<Vec<Uuid>>,
        }
        let body: Body = Faker.fake();

        let fin_res_service = MockFinResServiceExt::new();

        let app = get_fin_res_routes(Arc::new(fin_res_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/resources")
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
    async fn create_balance_sheet_resource_error_422_missing_body_attribute() {
        #[derive(Debug, Clone, Serialize, Dummy)]
        struct Body {
            category: ResourceCategory,
            #[serde(rename = "type")]
            r_type: ResourceType,
            editable: bool,
            year: i32,
            balance_per_month: BTreeMap<MonthNum, i64>,
            ynab_account_ids: Option<Vec<Uuid>>,
            external_account_ids: Option<Vec<Uuid>>,
        }
        let body: Body = Faker.fake();

        let fin_res_service = MockFinResServiceExt::new();

        let app = get_fin_res_routes(Arc::new(fin_res_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/resources")
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
    async fn create_balance_sheet_resource_error_422_wrong_body_attribute_type() {
        #[derive(Debug, Clone, Serialize, Dummy)]
        struct Body {
            name: i64,
            category: ResourceCategory,
            #[serde(rename = "type")]
            r_type: ResourceType,
            editable: bool,
            year: i32,
            balance_per_month: BTreeMap<MonthNum, i64>,
            ynab_account_ids: Option<Vec<Uuid>>,
            external_account_ids: Option<Vec<Uuid>>,
        }
        let body: Body = Faker.fake();

        let fin_res_service = MockFinResServiceExt::new();

        let app = get_fin_res_routes(Arc::new(fin_res_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/resources")
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
    async fn create_balance_sheet_resource_error_400_empty_body() {
        let fin_res_service = MockFinResServiceExt::new();

        let app = get_fin_res_routes(Arc::new(fin_res_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/resources")
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
    async fn create_balance_sheet_resource_error_415_missing_json_content_type() {
        let new_res: SaveResource = Faker.fake();

        let fin_res_service = MockFinResServiceExt::new();

        let app = get_fin_res_routes(Arc::new(fin_res_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/resources")
                    .method("POST")
                    .body(serde_json::to_vec(&new_res).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }

    #[tokio::test]
    async fn create_balance_sheet_resource_error_405_wrong_http_method() {
        let new_res: SaveResource = Faker.fake();

        let fin_res_service = MockFinResServiceExt::new();

        let app = get_fin_res_routes(Arc::new(fin_res_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/resources")
                    .method("PUT")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_vec(&new_res).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn create_balance_sheet_resource_error_409_when_already_exists() {
        let new_res: SaveResource = Faker.fake();

        let mut fin_res_service = MockFinResServiceExt::new();

        fin_res_service
            .expect_create_fin_res()
            .returning(|_| Err(AppError::ResourceAlreadyExist));

        let app = get_fin_res_routes(Arc::new(fin_res_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/resources")
                    .method("POST")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_vec(&new_res).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CONFLICT);
    }
}
