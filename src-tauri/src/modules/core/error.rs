use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum BackendError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Tauri error: {0}")]
    Tauri(#[from] tauri::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Custom error: {0}")]
    Custom(String),
}

impl Serialize for BackendError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

// Helper for converting strings to custom errors
impl From<String> for BackendError {
    fn from(s: String) -> Self {
        BackendError::Custom(s)
    }
}

impl From<&str> for BackendError {
    fn from(s: &str) -> Self {
        BackendError::Custom(s.to_string())
    }
}
