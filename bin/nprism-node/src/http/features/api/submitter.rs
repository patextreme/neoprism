use axum::Json;
use utoipa::OpenApi;

use crate::http::features::api::submitter::models::{
    SignedOperationSubmissionRequest, SignedOperationSubmissionResponse,
};
use crate::http::features::api::tags;
use crate::http::urls;

#[derive(OpenApi)]
#[openapi(paths(submit_signed_operations))]
pub struct SubmitterOpenApiDoc;

mod models {
    use serde::{Deserialize, Serialize};
    use utoipa::ToSchema;

    use crate::http::features::api::models::HexStrBytes;

    #[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
    pub struct SignedOperationSubmissionRequest {
        pub signed_operations: Vec<HexStrBytes>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
    pub struct SignedOperationSubmissionResponse {
        pub tx_id: String,
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
    _: Json<SignedOperationSubmissionRequest>,
) -> Json<SignedOperationSubmissionResponse> {
    Json(SignedOperationSubmissionResponse {
        tx_id: "TODO".to_string(),
    })
}
