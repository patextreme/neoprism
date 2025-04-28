use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageQueryParams {
    pub page: Option<u32>
}
