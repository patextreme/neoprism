use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use identus_did_core::DidDocument;
use identus_did_prism::did::PrismDidOps;
use utoipa::OpenApi;

use crate::AppState;
use crate::app::service::error::ResolutionError;
use crate::http::features::api::tags;
use crate::http::urls::ApiDid;

#[derive(OpenApi)]
#[openapi(paths(resolve_did))]
pub struct IndexerOpenApiDoc;

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
pub async fn resolve_did(
    Path(did): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<DidDocument>, StatusCode> {
    let (result, _) = state.did_service.resolve_did(&did).await;
    match result {
        Err(ResolutionError::InvalidDid { .. }) => Err(StatusCode::BAD_REQUEST),
        Err(ResolutionError::NotFound) => Err(StatusCode::NOT_FOUND),
        Err(ResolutionError::InternalError { .. }) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        Ok((did, did_state)) => Ok(Json(did_state.to_did_document(&did.to_did()))),
    }
}
