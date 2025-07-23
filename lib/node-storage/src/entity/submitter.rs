use chrono::{DateTime, Utc};
use lazybe::macros::Entity;
use lazybe::uuid::Uuid;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::entity::BytesHex;

#[derive(Entity, Serialize, Deserialize, ToSchema)]
#[lazybe(table = "staging_operation", endpoint = "/api/staging-operations", derive_to_schema)]
pub struct StagingOperation {
    #[lazybe(primary_key)]
    pub id: Uuid,
    pub signed_operation: BytesHex,
    #[lazybe(created_at)]
    pub submitted_at: DateTime<Utc>,
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
    pub signed_operation: BytesHex,
    #[lazybe(created_at)]
    pub submitted_at: DateTime<Utc>,
}
