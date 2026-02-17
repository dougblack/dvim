use thiserror::Error;

#[derive(Debug, Error)]
pub enum DvimError {
    #[error("failed to read file '{path}': {source}")]
    FileRead {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to write file '{path}': {source}")]
    FileWrite {
        path: String,
        source: std::io::Error,
    },
}
