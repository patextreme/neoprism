use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::RunMode;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AppMeta {
    pub version: String,
    pub mode: AppMetaRunMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum AppMetaRunMode {
    Indexer,
    Submitter,
    Standalone,
}

impl From<RunMode> for AppMetaRunMode {
    fn from(value: RunMode) -> Self {
        match value {
            RunMode::Indexer => Self::Indexer,
            RunMode::Submitter => Self::Submitter,
            RunMode::Standalone => Self::Standalone,
        }
    }
}
