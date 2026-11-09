use crate::{
    core::{errors::AppError, models::SessionJournal},
    persistence::session_journal,
};
pub fn recoverable() -> Result<Option<SessionJournal>, AppError> {
    session_journal::load()
}
pub fn save(journal: &SessionJournal) -> Result<(), AppError> {
    session_journal::save(journal)
}
pub fn clear() -> Result<(), AppError> {
    session_journal::clear()
}
