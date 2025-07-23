use chrono::{DateTime, Utc};
use identus_did_prism::prelude::SignedPrismOperation;
use identus_did_prism::proto::MessageExt;
use lazybe::macros::Entity;
use lazybe::router::{ErrorResponse, ValidationHook};
use lazybe::uuid::Uuid;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::entity::HexStrBytes;

#[derive(Entity, Serialize, Deserialize, ToSchema)]
#[lazybe(
    table = "staging_operation",
    endpoint = "/api/staging-operations",
    validation = "manual",
    derive_to_schema
)]
pub struct StagingOperation {
    #[lazybe(primary_key)]
    pub id: Uuid,
    pub signed_operation: HexStrBytes,
    #[lazybe(created_at)]
    pub submitted_at: DateTime<Utc>,
}

impl ValidationHook for StagingOperation {
    fn before_create(input: &Self::Create) -> Result<(), ErrorResponse> {
        SignedPrismOperation::decode(&input.signed_operation.0).map_err(|e| ErrorResponse {
            title: "Invalid SignedOperation".to_string(),
            detail: Some(format!(
                "The signed_operation cannot be decoded to a protobuf message: {e}"
            )),
            instance: None,
        })?;
        Ok(())
    }
}

#[derive(Entity, Serialize, Deserialize, ToSchema)]
#[lazybe(
    table = "submitted_operation",
    endpoint = "/api/submitted-operations",
    derive_to_schema
)]
pub struct SubmittedOperation {
    #[lazybe(primary_key)]
    pub id: Uuid,
    pub signed_operation: HexStrBytes,
    #[lazybe(created_at)]
    pub submitted_at: DateTime<Utc>,
}
