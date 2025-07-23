use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use utoipa::OpenApi;

use crate::AppState;
use crate::http::features::api::submitter::models::{
    SignedOperationSubmissionRequest, SignedOperationSubmissionResponse,
};
use crate::http::features::api::tags;
use crate::http::urls;

#[derive(OpenApi)]
#[openapi(paths(submit_signed_operations))]
pub struct SubmitterOpenApiDoc;

mod models {
    use identus_did_prism_submitter::dlt::TxId;
    use serde::{Deserialize, Serialize};
    use utoipa::ToSchema;

    use crate::http::features::api::models::SignedOperationHexStr;

    #[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
    pub struct SignedOperationSubmissionRequest {
        pub signed_operations: Vec<SignedOperationHexStr>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
    pub struct SignedOperationSubmissionResponse {
        pub tx_id: TxId,
    }
}

#[utoipa::path(
    post,
    path = urls::ApiSignedOpSubmissions::AXUM_PATH,
    tags = [tags::OP_SUBMIT],
    request_body = SignedOperationSubmissionRequest,
    responses(
        (status = OK, description = "Operations submitted successfully", body = SignedOperationSubmissionResponse)
    )
)]
pub async fn submit_signed_operations(
    State(state): State<AppState>,
    Json(req): Json<SignedOperationSubmissionRequest>,
) -> Result<Json<SignedOperationSubmissionResponse>, StatusCode> {
    let ops = req.signed_operations.into_iter().map(|i| i.into()).collect();
    let result = state
        .dlt_sink
        .expect("DLT sink is not configured for operation submission")
        .publish_operations(ops)
        .await;

    // TODO: improve error handling
    match result {
        Ok(tx_id) => Ok(Json(SignedOperationSubmissionResponse { tx_id })),
        Err(e) => {
            tracing::error!("{}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
