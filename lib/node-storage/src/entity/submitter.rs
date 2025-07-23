use chrono::{DateTime, Utc};
use lazybe::macros::Entity;
use lazybe::uuid::Uuid;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::entity::BytesHex;

#[derive(Entity, Serialize, Deserialize, ToSchema)]
#[lazybe(table = "scheduled_operation", endpoint = "/scheduled-operations", derive_to_schema)]
pub struct ScheduledOperation {
    #[lazybe(primary_key)]
    pub id: Uuid, // TODO: use hash
    pub signed_operation: BytesHex,
    #[lazybe(created_at)]
    pub submitted_at: DateTime<Utc>,
}
