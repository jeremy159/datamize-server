use axum::{extract::State, Json};
use datamize_domain::Uuid;

use crate::{error::HttpJsonDatamizeResult, services::balance_sheet::DynRefreshFinResService};

/// Endpoint to refresh financial resources.
/// Only resources from the current month will be refreshed by this endpoint.
/// If current month does not exists, it will create it.
/// This endpoint basically calls the YNAB api for some resources and starts a web scrapper for others.
/// Will return an array of ids for Financial Resources updated.
#[tracing::instrument(skip_all)]
pub async fn refresh_balance_sheet_resources(
    State(mut fin_res_service): State<DynRefreshFinResService>,
) -> HttpJsonDatamizeResult<Vec<Uuid>> {
    Ok(Json(fin_res_service.refresh_fin_res().await?))
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use datamize_domain::async_trait;
    use fake::{Fake, Faker};
    use tower::ServiceExt;

    use crate::{
        error::{AppError, DatamizeResult},
        routes::api::balance_sheet::get_refresh_fin_res_routes,
        services::balance_sheet::RefreshFinResServiceExt,
    };

    use super::*;

    #[tokio::test]
    async fn refresh_balance_sheet_resources_success() {
        #[derive(Clone)]
        struct MockRefreshFinResService {}
        #[async_trait]
        impl RefreshFinResServiceExt for MockRefreshFinResService {
            async fn refresh_fin_res(&mut self) -> DatamizeResult<Vec<Uuid>> {
                Ok(vec![])
            }
        }
        let fin_res_service = Box::new(MockRefreshFinResService {});

        let app = get_refresh_fin_res_routes(fin_res_service);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/resources/refresh")
                    .method("POST")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: Vec<Uuid> = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, vec![]);
    }

    #[tokio::test]
    async fn refresh_balance_sheet_resources_error_500() {
        #[derive(Clone)]
        struct MockRefreshFinResService {}
        #[async_trait]
        impl RefreshFinResServiceExt for MockRefreshFinResService {
            async fn refresh_fin_res(&mut self) -> DatamizeResult<Vec<Uuid>> {
                Err(AppError::InternalServerError(
                    ynab::Error::Api(ynab::ApiError {
                        id: Faker.fake(),
                        name: Faker.fake(),
                        detail: Faker.fake(),
                    })
                    .into(),
                ))
            }
        }
        let fin_res_service = Box::new(MockRefreshFinResService {});

        let app = get_refresh_fin_res_routes(fin_res_service);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/resources/refresh")
                    .method("POST")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
