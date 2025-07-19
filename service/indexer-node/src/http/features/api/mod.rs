use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use identus_did_core::DidDocument;
use identus_did_prism::did::PrismDidOps;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::app::service::error::ResolutionError;
use crate::http::features::api::models::BuildMeta;
use crate::http::urls;
use crate::{AppState, VERSION};

mod models;

#[derive(OpenApi)]
#[openapi(servers(
    (url = "http://localhost:8080", description = "Local"),
    (url = "https://neoprism.patlo.dev", description = "Public - mainnet"),
    (url = "https://neoprism-preprod.patlo.dev", description = "Public - preprod")
), paths(resolve_did, health, build_meta))]
struct OpenApiDoc;

pub fn router() -> Router<AppState> {
    let openapi = OpenApiDoc::openapi();
    Router::new()
        .merge(SwaggerUi::new(urls::Swagger::AXUM_PATH).url("/api/openapi.json", openapi))
        .route(urls::ApiDid::AXUM_PATH, get(resolve_did))
        .route(urls::ApiHealth::AXUM_PATH, get(health))
        .route(urls::ApiBuildMeta::AXUM_PATH, get(build_meta))
}

#[utoipa::path(
    get,
    path = "/api/_system/health",
    tags = ["System"],
    responses(
        (status = OK, description = "Healthy", body = String, example = "Ok"),
    )
)]
async fn health() -> &'static str {
    "Ok"
}

#[utoipa::path(
    get,
    path = "/api/_system/build",
    tags = ["System"],
    responses(
        (status = OK, description = "Healthy", body = String, example = "Ok"),
    )
)]
async fn build_meta() -> Json<BuildMeta> {
    Json(BuildMeta {
        version: VERSION.to_string(),
    })
}

#[utoipa::path(
    get,
    path = "/api/dids/{did}",
    tags = ["DIDs"],
    responses(
        (status = OK, description = "Resolve DID successfully", body = DidDocument),
        (status = BAD_REQUEST, description = "Invalid DID"),
        (status = NOT_FOUND, description = "DID not found"),
        (status = INTERNAL_SERVER_ERROR, description = "Internal server error"),
    ),
    params(
        ("did" = String, Path, description = "The DID to resolve", example = "did:prism:b02cc5ce2300b3c6d38496fbc2762eaf07a51cabc8708e8f1eb114d0f14398c5"),
    )
)]
async fn resolve_did(Path(did): Path<String>, State(state): State<AppState>) -> Result<Json<DidDocument>, StatusCode> {
    let (result, _) = state.did_service.resolve_did(&did).await;
    match result {
        Err(ResolutionError::InvalidDid { .. }) => Err(StatusCode::BAD_REQUEST),
        Err(ResolutionError::NotFound) => Err(StatusCode::NOT_FOUND),
        Err(ResolutionError::InternalError { .. }) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        Ok((did, did_state)) => Ok(Json(did_state.to_did_document(&did.to_did()))),
    }
}
