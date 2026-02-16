use thiserror::Error;

#[derive(Debug, Error)]
pub enum RvimError {
    #[error("failed to read file '{path}': {source}")]
    FileRead {
        path: String,
        source: std::io::Error,
    },
}
