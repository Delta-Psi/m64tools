use crate::types::ID;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AiffError {
    #[error("invalid format")]
    InvalidFormat,
    #[error("form type is not AIFF but {0}")]
    InvalidFormType(ID),

    #[error("missing common chunk")]
    MissingComm,
    #[error("missing sound data chunk")]
    MissingSsnd,

    #[error("invalid play mode {0}")]
    InvalidPlayMode(u16),
}

pub type Result<T> = std::result::Result<T, AiffError>;
