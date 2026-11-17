use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("not implemented yet: {0}")]
    NotImplemented(&'static str),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

// Errors cross the IPC boundary as plain strings.
impl Serialize for Error {
    fn serialize<S: serde::Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
