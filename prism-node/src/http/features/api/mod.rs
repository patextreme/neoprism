use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use models::DidDocument;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::AppState;
use crate::app::service::error::ResolutionError;
use crate::http::urls;

mod models;

#[derive(OpenApi)]
#[openapi(servers((url = "http://localhost:8080", description = "Local")), paths(resolve_did))]
struct OpenApiDoc;

pub fn router() -> Router<AppState> {
    let openapi = OpenApiDoc::openapi();
    Router::new()
        .merge(SwaggerUi::new(urls::Swagger::axum_url()).url("/api/openapi.json", openapi))
        .route(urls::DidResolver::axum_url(), get(resolve_did))
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
        Ok((did, did_state)) => Ok(Json(DidDocument::new(&did.to_string(), did_state))),
    }
}
