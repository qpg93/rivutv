use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Config parse error: {0}")]
    Config(String),

    #[error("Spider error: {0}")]
    Spider(String),

    #[error("Player error: {0}")]
    Player(String),
}

pub type Result<T> = std::result::Result<T, CoreError>;
