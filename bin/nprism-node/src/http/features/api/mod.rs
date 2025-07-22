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
use crate::{AppState, RunMode, VERSION};

mod models;

#[derive(OpenApi)]
#[openapi(servers(
    (url = "http://localhost:8080", description = "Local"),
    (url = "https://neoprism.patlo.dev", description = "Public - mainnet"),
    (url = "https://neoprism-preprod.patlo.dev", description = "Public - preprod")
), paths(health, build_meta))]
struct SystemOpenApiDoc;

#[derive(OpenApi)]
#[openapi(paths(resolve_did))]
struct IndexerOpenApiDoc;

#[derive(OpenApi)]
struct SubmitterOpenApiDoc;

pub fn router(mode: RunMode) -> Router<AppState> {
    let system_oas = SystemOpenApiDoc::openapi();
    let indexer_oas = IndexerOpenApiDoc::openapi();
    let submitter_oas = SubmitterOpenApiDoc::openapi();

    let mut combined_oas = system_oas;
    match mode {
        RunMode::Indexer => combined_oas.merge(indexer_oas),
        RunMode::Submitter => combined_oas.merge(submitter_oas),
    }

    let system_router = Router::new()
        .merge(SwaggerUi::new(urls::Swagger::AXUM_PATH).url("/api/openapi.json", combined_oas))
        .route(urls::ApiHealth::AXUM_PATH, get(health))
        .route(urls::ApiBuildMeta::AXUM_PATH, get(build_meta));

    let indexer_router = Router::new().route(urls::ApiDid::AXUM_PATH, get(resolve_did));

    let submitter_router = Router::new();

    match mode {
        RunMode::Indexer => system_router.merge(indexer_router),
        RunMode::Submitter => system_router.merge(submitter_router),
    }
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
        (status = OK, description = "Healthy", body = BuildMeta),
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
