use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageQueryParams {
    pub page: Option<u32>,
}
