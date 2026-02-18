use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum CoreError {
    InvalidJson(String),
    SerializationError(String),
}

impl Display for CoreError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CoreError::InvalidJson(e) => write!(f, "invalid json: {e}"),
            CoreError::SerializationError(e) => write!(f, "serialization error: {e}"),
        }
    }
}

impl std::error::Error for CoreError {}

pub type CoreResult<T> = Result<T, CoreError>;
