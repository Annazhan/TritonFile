use std::{error::Error, fmt::Display};

#[derive(Debug, Clone)]
pub enum TritonFileError {
    FileDoesNotExist(String),
    PathTaken(String),
    InvalidFilename(String),
    RpcError(String),
    FilesTooMany,
    /// catch-all error for other issues
    Unknown(String),
}

impl Display for TritonFileError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let x = match self {
            TritonFileError::FileDoesNotExist(x) => format!("file \"{}\" does not exist", x),
            TritonFileError::InvalidFilename(x) => format!("file name \"{}\" already taken", x),
            TritonFileError::PathTaken(x) => format!("path \"{}\" is invalid", x),
            TritonFileError::RpcError(x) => format!("rpc error: {}", x),
            TritonFileError::FilesTooMany => "too many files".to_string(),
        };
        write!(f, "{}", x)
    }
}

impl std::error::Error for TritonFileError {}

impl From<tonic::Status> for TritonFileError {
    fn from(v: tonic::Status) -> Self {
        TritonFileError::RpcError(format!("{:?}", v))
    }
}

impl From<tonic::transport::Error> for TritonFileError {
    fn from(v: tonic::transport::Error) -> Self {
        TritonFileError::RpcError(format!("{:?}", v))
    }
}

/// A [Result] type which either returns `T` or a [boxed error](https://doc.rust-lang.org/rust-by-example/error/multiple_error_types/boxing_errors.html)
pub type TritonFileResult<T> = Result<T, Box<(dyn Error + Send + Sync)>>;

impl From<Box<dyn Error>> for TritonFileError {
    fn from(x: Box<dyn Error>) -> Self {
        TritonFileError::Unknown(x.to_string())
    }
}
