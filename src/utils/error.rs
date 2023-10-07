use std::fmt;

#[derive(Debug, Clone)]
pub enum RustWireError {
    HttpError(String),
    HttpStatusCodeError(String),
    IOError(String),
    TaskError(String),
}

impl fmt::Display for RustWireError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RustWireError::HttpError(err) => write!(f, "HTTP Error: {}", err),
            RustWireError::HttpStatusCodeError(err) => write!(f, "HTTP Status Code Error: {}", err),
            RustWireError::IOError(err) => write!(f, "IO Error: {}", err),
            RustWireError::TaskError(err) => write!(f, "Task Error: {}", err),
        }
    }
}

impl std::error::Error for RustWireError {}
