mod ssi;
mod storage;

pub use ssi::*;
pub use storage::*;

#[derive(Debug, Clone)]
pub struct OperationParameters {
    pub max_services: usize,
    pub max_public_keys: usize,
    pub max_id_size: usize,
    pub max_type_size: usize,
    pub max_service_endpoint_size: usize,
}

impl OperationParameters {
    pub fn v1() -> Self {
        Self {
            max_services: 50,
            max_public_keys: 50,
            max_id_size: 50,
            max_type_size: 100,
            max_service_endpoint_size: 300,
        }
    }
}
