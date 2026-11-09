use crate::core::{errors::AppError, models::AdapterInfo};

pub trait AdapterService: Send + Sync {
    fn enumerate(&self) -> Result<Vec<AdapterInfo>, AppError>;
}

pub fn enumerate() -> Result<Vec<AdapterInfo>, AppError> {
    crate::platform::enumerate_adapters()
}
