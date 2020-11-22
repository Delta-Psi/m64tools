use thiserror::Error;
#[derive(Error, Debug)]
pub enum AiffError {
    #[error("invalid format")]
    InvalidFormat,
    #[error("form type is not AIFF")]
    InvalidFormType,
    #[error("missing common chunk")]
    MissingComm,
}

pub type Result<T> = std::result::Result<T, AiffError>;
