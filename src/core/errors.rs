use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum AppError {
    #[error("Operace vyžaduje oprávnění správce.")]
    AccessDenied,
    #[error("Síťové rozhraní již není dostupné.")]
    AdapterUnavailable,
    #[error("Dočasná adresa již existuje nebo koliduje s jinou adresou.")]
    AddressConflict,
    #[error("Operace byla zrušena.")]
    Cancelled,
    #[error("Chyba platformy: {0}")]
    Platform(String),
    #[error("Chyba úložiště: {0}")]
    Storage(String),
}
