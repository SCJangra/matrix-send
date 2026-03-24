use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("request failed with status {status}: {body}")]
    Api { status: u16, body: String },

    #[error("request error: {0}")]
    Ureq(#[from] ureq::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
