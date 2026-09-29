use thiserror::Error;

pub type Result<T> = std::result::Result<T, AevoError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuilderErrorCode {
    BuilderAlreadyExists,
    BuilderCursorWithOffset,
    BuilderFeeExceedsAevoLimit,
    BuilderFeeExceedsUserLimit,
    BuilderInvalidFeeRate,
    BuilderInvalidId,
    BuilderInvalidSignature,
    BuilderNotActive,
    BuilderNotApproved,
    BuilderNotFound,
    BuilderNotSupported,
    Unknown(String),
}

impl BuilderErrorCode {
    pub fn as_str(&self) -> &str {
        match self {
            Self::BuilderAlreadyExists => "BUILDER_ALREADY_EXISTS",
            Self::BuilderCursorWithOffset => "BUILDER_CURSOR_WITH_OFFSET",
            Self::BuilderFeeExceedsAevoLimit => "BUILDER_FEE_EXCEEDS_AEVO_LIMIT",
            Self::BuilderFeeExceedsUserLimit => "BUILDER_FEE_EXCEEDS_USER_LIMIT",
            Self::BuilderInvalidFeeRate => "BUILDER_INVALID_FEE_RATE",
            Self::BuilderInvalidId => "BUILDER_INVALID_ID",
            Self::BuilderInvalidSignature => "BUILDER_INVALID_SIGNATURE",
            Self::BuilderNotActive => "BUILDER_NOT_ACTIVE",
            Self::BuilderNotApproved => "BUILDER_NOT_APPROVED",
            Self::BuilderNotFound => "BUILDER_NOT_FOUND",
            Self::BuilderNotSupported => "BUILDER_NOT_SUPPORTED",
            Self::Unknown(value) => value.as_str(),
        }
    }
}

impl From<&str> for BuilderErrorCode {
    fn from(value: &str) -> Self {
        match value {
            "BUILDER_ALREADY_EXISTS" => Self::BuilderAlreadyExists,
            "BUILDER_CURSOR_WITH_OFFSET" => Self::BuilderCursorWithOffset,
            "BUILDER_FEE_EXCEEDS_AEVO_LIMIT" => Self::BuilderFeeExceedsAevoLimit,
            "BUILDER_FEE_EXCEEDS_USER_LIMIT" => Self::BuilderFeeExceedsUserLimit,
            "BUILDER_INVALID_FEE_RATE" => Self::BuilderInvalidFeeRate,
            "BUILDER_INVALID_ID" => Self::BuilderInvalidId,
            "BUILDER_INVALID_SIGNATURE" => Self::BuilderInvalidSignature,
            "BUILDER_NOT_ACTIVE" => Self::BuilderNotActive,
            "BUILDER_NOT_APPROVED" => Self::BuilderNotApproved,
            "BUILDER_NOT_FOUND" => Self::BuilderNotFound,
            "BUILDER_NOT_SUPPORTED" => Self::BuilderNotSupported,
            other => Self::Unknown(other.to_string()),
        }
    }
}

#[derive(Debug, Error)]
pub enum AevoError {
    #[error("Aevo API error {status}: {message}")]
    Api {
        status: u16,
        code: Option<BuilderErrorCode>,
        message: String,
    },

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Signing error: {0}")]
    Signing(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("WebSocket error: {0}")]
    WebSocket(String),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("URL error: {0}")]
    Url(#[from] url::ParseError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

impl AevoError {
    pub(crate) fn api(status: u16, payload: &serde_json::Value) -> Self {
        let code_text = payload
            .get("code")
            .and_then(serde_json::Value::as_str)
            .or_else(|| payload.get("error").and_then(serde_json::Value::as_str));
        let message = payload
            .get("message")
            .and_then(serde_json::Value::as_str)
            .or(code_text)
            .unwrap_or("request failed")
            .to_string();
        let code = code_text.map(BuilderErrorCode::from);
        Self::Api {
            status,
            code,
            message,
        }
    }
}
