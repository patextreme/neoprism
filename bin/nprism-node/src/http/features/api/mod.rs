use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use identus_did_core::DidDocument;
use identus_did_prism::did::PrismDidOps;
use lazybe::openapi::{CreateRouterDoc, GetRouterDoc, ListRouterDoc};
use lazybe::router::{CreateRouter, GetRouter, ListRouter};
use node_storage::entity::{StagingOperation, SubmittedOperation};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::app::service::error::ResolutionError;
use crate::http::features::api::models::AppMeta;
use crate::http::urls::{self, ApiAppMeta, ApiDid, ApiHealth};
use crate::{AppState, RunMode, VERSION};

mod models;

#[derive(OpenApi)]
#[openapi(servers(
    (url = "http://localhost:8080", description = "Local"),
    (url = "https://neoprism.patlo.dev", description = "Public - mainnet"),
    (url = "https://neoprism-preprod.patlo.dev", description = "Public - preprod")
), paths(health, app_meta))]
struct SystemOpenApiDoc;

#[derive(OpenApi)]
#[openapi(paths(resolve_did))]
struct IndexerOpenApiDoc;

#[derive(OpenApi)]
struct SubmitterOpenApiDoc;

mod tags {
    pub const SYSTEM: &str = "System";
    pub const DID: &str = "DID";
    pub const OPS_SUBMIT: &str = "Operation submission";
}

pub fn router(mode: RunMode) -> Router<AppState> {
    let system_oas = SystemOpenApiDoc::openapi();
    let indexer_oas = IndexerOpenApiDoc::openapi();
    let submitter_oas = SubmitterOpenApiDoc::openapi()
        .merge_from(StagingOperation::create_endpoint_doc(Some(tags::OPS_SUBMIT)))
        .merge_from(StagingOperation::get_endpoint_doc(Some(tags::OPS_SUBMIT)))
        .merge_from(StagingOperation::list_endpoint_doc(Some(tags::OPS_SUBMIT)))
        .merge_from(SubmittedOperation::get_endpoint_doc(Some(tags::OPS_SUBMIT)))
        .merge_from(SubmittedOperation::list_endpoint_doc(Some(tags::OPS_SUBMIT)));

    let oas = match mode {
        RunMode::Indexer => system_oas.merge_from(indexer_oas),
        RunMode::Submitter => system_oas.merge_from(submitter_oas),
        RunMode::Standalone => system_oas.merge_from(indexer_oas).merge_from(submitter_oas),
    };

    let system_router = Router::new()
        .merge(SwaggerUi::new(urls::Swagger::AXUM_PATH).url("/api/openapi.json", oas))
        .route(urls::ApiHealth::AXUM_PATH, get(health))
        .route(urls::ApiAppMeta::AXUM_PATH, get(app_meta));

    let indexer_router = Router::new().route(urls::ApiDid::AXUM_PATH, get(resolve_did));

    let submitter_router = Router::new()
        .merge(StagingOperation::create_endpoint())
        .merge(StagingOperation::get_endpoint())
        .merge(StagingOperation::list_endpoint())
        .merge(SubmittedOperation::get_endpoint())
        .merge(SubmittedOperation::list_endpoint());

    match mode {
        RunMode::Indexer => system_router.merge(indexer_router),
        RunMode::Submitter => system_router.merge(submitter_router),
        RunMode::Standalone => system_router.merge(indexer_router).merge(submitter_router),
    }
}

#[utoipa::path(
    get,
    path = ApiHealth::AXUM_PATH,
    tags = [tags::SYSTEM],
    responses(
        (status = OK, description = "Healthy", body = String, example = "Ok"),
    )
)]
async fn health() -> &'static str {
    "Ok"
}

#[utoipa::path(
    get,
    path = ApiAppMeta::AXUM_PATH,
    tags = [tags::SYSTEM],
    responses(
        (status = OK, description = "Healthy", body = AppMeta),
    )
)]
async fn app_meta(State(state): State<AppState>) -> Json<AppMeta> {
    Json(AppMeta {
        version: VERSION.to_string(),
        mode: state.run_mode.into(),
    })
}

#[utoipa::path(
    get,
    path = ApiDid::AXUM_PATH,
    tags = [tags::DID],
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
