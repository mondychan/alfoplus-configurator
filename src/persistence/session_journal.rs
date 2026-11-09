use crate::core::{errors::AppError, models::SessionJournal};
use std::{fs, path::PathBuf};

fn path() -> Result<PathBuf, AppError> {
    dirs::data_local_dir()
        .map(|p| p.join("AlfoPlusConfigurator").join("session.json"))
        .ok_or_else(|| AppError::Storage("Nelze najít LocalAppData.".into()))
}
pub fn save(j: &SessionJournal) -> Result<(), AppError> {
    let p = path()?;
    let dir = p
        .parent()
        .ok_or_else(|| AppError::Storage("Neplatná cesta.".into()))?;
    fs::create_dir_all(dir).map_err(|e| AppError::Storage(e.to_string()))?;
    let tmp = p.with_extension("json.tmp");
    fs::write(
        &tmp,
        serde_json::to_vec_pretty(j).map_err(|e| AppError::Storage(e.to_string()))?,
    )
    .map_err(|e| AppError::Storage(e.to_string()))?;
    fs::rename(tmp, p).map_err(|e| AppError::Storage(e.to_string()))
}
pub fn load() -> Result<Option<SessionJournal>, AppError> {
    let p = path()?;
    if !p.exists() {
        return Ok(None);
    }
    let data = fs::read(p).map_err(|e| AppError::Storage(e.to_string()))?;
    serde_json::from_slice(&data)
        .map(Some)
        .map_err(|e| AppError::Storage(e.to_string()))
}
pub fn clear() -> Result<(), AppError> {
    let p = path()?;
    if p.exists() {
        fs::remove_file(p).map_err(|e| AppError::Storage(e.to_string()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn journal_roundtrip_shape() {
        let j = SessionJournal {
            adapter_id: "x".into(),
            temporary_ip: "172.20.253.14".parse().unwrap(),
            prefix_length: 16,
            created_by_application: true,
        };
        let encoded = serde_json::to_string(&j).unwrap();
        assert!(encoded.contains("172.20.253.14"));
    }
}
