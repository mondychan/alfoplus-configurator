use crate::core::{
    errors::AppError,
    models::{AdapterInfo, SessionJournal, LOCAL_IP, PREFIX_LENGTH},
};

pub trait AddressService: Send + Sync {
    fn add_temporary(&self, adapter: &AdapterInfo) -> Result<SessionJournal, AppError>;
    fn remove_temporary(&self, journal: &SessionJournal) -> Result<(), AppError>;
    fn is_configured(&self, journal: &SessionJournal) -> Result<bool, AppError>;
}

pub fn add(adapter: &AdapterInfo) -> Result<SessionJournal, AppError> {
    crate::platform::add_temporary_address(adapter, LOCAL_IP, PREFIX_LENGTH)
}
pub fn remove(journal: &SessionJournal) -> Result<(), AppError> {
    crate::platform::remove_temporary_address(journal)
}
pub fn configured(journal: &SessionJournal) -> Result<bool, AppError> {
    crate::platform::temporary_address_configured(journal)
}
