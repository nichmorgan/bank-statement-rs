use thiserror::Error;

#[derive(Error, Debug)]
pub enum StatementParseError {
    #[error("Parse failed: {0}")]
    ParseFailed(String),
    #[error("Unsupported file format")]
    UnsupportedFormat,
    #[error("Read content failed: {0}")]
    ReadContentFailed(#[from] std::io::Error),
    #[error("Content or filepath is required")]
    MissingContentAndFilepath,
    #[error("QFX date invalid format")]
    QfxDateInvalidFormat,
}
