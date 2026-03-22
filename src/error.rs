use reqwest::StatusCode;

#[derive(Debug, thiserror::Error)]
pub enum GrvtError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("API returned {status}: {body}")]
    ApiStatus { status: StatusCode, body: String },

    #[error("JSON deserialization failed: {0}")]
    Json(#[from] serde_json::Error),

    #[error("missing session cookie in login response")]
    MissingSessionCookie,

    #[error("missing account ID in login response")]
    MissingAccountId,

    #[error("configuration error: {0}")]
    Config(String),

    #[error("signing error: {0}")]
    Signing(String),

    #[error("WebSocket error: {0}")]
    WebSocket(String),

    #[error("hex decode error: {0}")]
    Hex(#[from] hex::FromHexError),
}

pub type Result<T> = std::result::Result<T, GrvtError>;
