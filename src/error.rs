use thiserror::Error;

#[derive(Error, Debug)]
pub enum EpxError {
    #[error("invalid EPUB: {0}")]
    InvalidEpub(String),

    #[error("XML error: {0}")]
    Xml(#[from] quick_xml::Error),

    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml_ng::Error),

    #[error("regex error: {0}")]
    Regex(#[from] regex::Error),
}

pub type Result<T> = std::result::Result<T, EpxError>;
