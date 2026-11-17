use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    /// User-facing message (Arabic), shown as-is in the UI.
    #[error("{0}")]
    Msg(String),
}

pub fn msg<T>(s: impl Into<String>) -> Result<T> {
    Err(Error::Msg(s.into()))
}

// Errors cross the IPC boundary as plain strings.
impl Serialize for Error {
    fn serialize<S: serde::Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
