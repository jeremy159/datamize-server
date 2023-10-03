use axum::{
    extract::{Path, State},
    Json,
};
use axum_extra::extract::WithRejection;
use datamize_domain::{FinancialResourceYearly, SaveResource, Uuid};

use crate::{
    error::{HttpJsonDatamizeResult, JsonError},
    services::balance_sheet::DynFinResService,
};

/// Returns a specific resource.
#[tracing::instrument(name = "Get a resource", skip_all)]
pub async fn balance_sheet_resource(
    Path(resource_id): Path<Uuid>,
    State(fin_res_service): State<DynFinResService>,
) -> HttpJsonDatamizeResult<FinancialResourceYearly> {
    Ok(Json(fin_res_service.get_fin_res(resource_id).await?))
}

/// Updates the resource. Will create any non-existing months.
/// Will also update the months' and year's net totals.
#[tracing::instrument(skip_all)]
pub async fn update_balance_sheet_resource(
    Path(resource_id): Path<Uuid>,
    State(fin_res_service): State<DynFinResService>,
    WithRejection(Json(body), _): WithRejection<Json<SaveResource>, JsonError>,
) -> HttpJsonDatamizeResult<FinancialResourceYearly> {
    Ok(Json(
        fin_res_service.update_fin_res(resource_id, body).await?,
    ))
}

/// Deletes the resource and returns the entity
#[tracing::instrument(skip_all)]
pub async fn delete_balance_sheet_resource(
    Path(resource_id): Path<Uuid>,
    State(fin_res_service): State<DynFinResService>,
) -> HttpJsonDatamizeResult<FinancialResourceYearly> {
    Ok(Json(fin_res_service.delete_fin_res(resource_id).await?))
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, sync::Arc};

    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use datamize_domain::{MonthNum, ResourceCategory, ResourceType};
    use fake::{Dummy, Fake, Faker};
    use serde::Serialize;
    use tower::ServiceExt;

    use crate::{
        error::AppError, routes::api::balance_sheet::get_fin_res_routes,
        services::balance_sheet::MockFinResServiceExt,
    };

    use super::*;

    #[tokio::test]
    async fn balance_sheet_resource_success() {
        let resource: FinancialResourceYearly = Faker.fake();

        let mut fin_res_service = MockFinResServiceExt::new();
        let resource_cloned = resource.clone();
        fin_res_service
            .expect_get_fin_res()
            .returning(move |_| Ok(resource_cloned.clone()));

        let app = get_fin_res_routes(Arc::new(fin_res_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/resources/{:?}", resource.base.id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: FinancialResourceYearly = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, resource);
    }

    #[tokio::test]
    async fn balance_sheet_resource_error_500() {
        let mut fin_res_service = MockFinResServiceExt::new();
        fin_res_service.expect_get_fin_res().returning(move |_| {
            Err(AppError::InternalServerError(Into::into(
                sqlx::Error::RowNotFound,
            )))
        });

        let app = get_fin_res_routes(Arc::new(fin_res_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/resources/{:?}", Faker.fake::<Uuid>()))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn balance_sheet_resource_error_404_non_existing_resource() {
        let mut fin_res_service = MockFinResServiceExt::new();
        fin_res_service
            .expect_get_fin_res()
            .returning(move |_| Err(AppError::ResourceNotFound));

        let app = get_fin_res_routes(Arc::new(fin_res_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/resources/{:?}", Faker.fake::<Uuid>()))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn balance_sheet_resource_error_400_invalid_uuid_format_in_path() {
        let fin_res_service = MockFinResServiceExt::new();

        let app = get_fin_res_routes(Arc::new(fin_res_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/resources/{:?}", Faker.fake::<i64>()))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn update_balance_sheet_resource_success() {
        let res_update: SaveResource = Faker.fake();
        let res: FinancialResourceYearly = res_update.clone().into();

        let mut fin_res_service = MockFinResServiceExt::new();
        let res_cloned = res.clone();
        fin_res_service
            .expect_update_fin_res()
            .returning(move |_, _| Ok(res_cloned.clone()));

        let app = get_fin_res_routes(Arc::new(fin_res_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/resources/{:?}", res.base.id))
                    .method("PUT")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_vec(&res_update).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: FinancialResourceYearly = serde_json::from_slice(&body).unwrap();
        assert_eq!(body.balance_per_month, res_update.balance_per_month);
        assert_eq!(body.year, res_update.year);
        assert_eq!(body.base.name, res_update.name);
        assert_eq!(body.base.category, res_update.category);
        assert_eq!(body.base.r_type, res_update.r_type);
        assert_eq!(body.base.editable, res_update.editable);
        assert_eq!(body.base.ynab_account_ids, res_update.ynab_account_ids);
        assert_eq!(
            body.base.external_account_ids,
            res_update.external_account_ids
        );
    }

    #[tokio::test]
    async fn update_balance_sheet_resource_error_500() {
        let res_update: SaveResource = Faker.fake();

        let mut fin_res_service = MockFinResServiceExt::new();
        fin_res_service
            .expect_update_fin_res()
            .returning(move |_, _| {
                Err(AppError::InternalServerError(Into::into(
                    sqlx::Error::RowNotFound,
                )))
            });

        let app = get_fin_res_routes(Arc::new(fin_res_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/resources/{:?}", Faker.fake::<Uuid>()))
                    .method("PUT")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_vec(&res_update).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn update_balance_sheet_resource_error_400_invalid_uuid_format_in_path() {
        let res_update: SaveResource = Faker.fake();

        let fin_res_service = MockFinResServiceExt::new();

        let app = get_fin_res_routes(Arc::new(fin_res_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/resources/{:?}", Faker.fake::<i64>()))
                    .method("PUT")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_vec(&res_update).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn update_balance_sheet_resource_error_422_wrong_body_attribute() {
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
                    .uri(format!("/resources/{:?}", Faker.fake::<Uuid>()))
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
    async fn update_balance_sheet_resource_error_422_missing_body_attribute() {
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
                    .uri(format!("/resources/{:?}", Faker.fake::<Uuid>()))
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
    async fn update_balance_sheet_resource_error_422_wrong_body_attribute_type() {
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
                    .uri(format!("/resources/{:?}", Faker.fake::<Uuid>()))
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
    async fn update_balance_sheet_resource_error_400_empty_body() {
        let fin_res_service = MockFinResServiceExt::new();

        let app = get_fin_res_routes(Arc::new(fin_res_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/resources/{:?}", Faker.fake::<Uuid>()))
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
    async fn update_balance_sheet_resource_error_415_missing_json_content_type() {
        let res_update: SaveResource = Faker.fake();

        let fin_res_service = MockFinResServiceExt::new();

        let app = get_fin_res_routes(Arc::new(fin_res_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/resources/{:?}", Faker.fake::<Uuid>()))
                    .method("PUT")
                    .body(serde_json::to_vec(&res_update).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }

    #[tokio::test]
    async fn update_balance_sheet_resource_error_405_wrong_http_method() {
        let res_update: SaveResource = Faker.fake();

        let fin_res_service = MockFinResServiceExt::new();

        let app = get_fin_res_routes(Arc::new(fin_res_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/resources/{:?}", Faker.fake::<Uuid>()))
                    .method("POST")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_vec(&res_update).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn update_balance_sheet_resource_error_404_when_not_found() {
        let res_update: SaveResource = Faker.fake();

        let mut fin_res_service = MockFinResServiceExt::new();

        fin_res_service
            .expect_update_fin_res()
            .returning(|_, _| Err(AppError::ResourceNotFound));

        let app = get_fin_res_routes(Arc::new(fin_res_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/resources/{:?}", Faker.fake::<Uuid>()))
                    .method("PUT")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_vec(&res_update).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn delete_balance_sheet_resource_success() {
        let resource: FinancialResourceYearly = Faker.fake();

        let mut fin_res_service = MockFinResServiceExt::new();
        let resource_cloned = resource.clone();
        fin_res_service
            .expect_delete_fin_res()
            .returning(move |_| Ok(resource_cloned.clone()));

        let app = get_fin_res_routes(Arc::new(fin_res_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/resources/{:?}", resource.base.id))
                    .method("DELETE")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: FinancialResourceYearly = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, resource);
    }

    #[tokio::test]
    async fn delete_balance_sheet_resource_error_500() {
        let mut fin_res_service = MockFinResServiceExt::new();
        fin_res_service.expect_delete_fin_res().returning(move |_| {
            Err(AppError::InternalServerError(Into::into(
                sqlx::Error::RowNotFound,
            )))
        });

        let app = get_fin_res_routes(Arc::new(fin_res_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/resources/{:?}", Faker.fake::<Uuid>()))
                    .method("DELETE")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn delete_balance_sheet_resource_error_404_non_existing_resource() {
        let mut fin_res_service = MockFinResServiceExt::new();
        fin_res_service
            .expect_delete_fin_res()
            .returning(move |_| Err(AppError::ResourceNotFound));

        let app = get_fin_res_routes(Arc::new(fin_res_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/resources/{:?}", Faker.fake::<Uuid>()))
                    .method("DELETE")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn delete_balance_sheet_resource_error_400_invalid_uuid_format_in_path() {
        let fin_res_service = MockFinResServiceExt::new();

        let app = get_fin_res_routes(Arc::new(fin_res_service));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/resources/{:?}", Faker.fake::<i64>()))
                    .method("DELETE")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
