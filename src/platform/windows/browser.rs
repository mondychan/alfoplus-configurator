use crate::core::errors::AppError;
pub fn launch_browser(url: &str) -> Result<(), AppError> {
    std::process::Command::new("cmd")
        .args(["/C", "start", "", url])
        .spawn()
        .map(|_| ())
        .map_err(|e| AppError::Platform(e.to_string()))
}
