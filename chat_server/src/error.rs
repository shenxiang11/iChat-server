use axum::http::{Response, StatusCode};
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum AppError {
    #[error("email already exists: {0}")]
    EmailAlreadyExists(String),

    #[error("sql error: {0}")]
    SqlxError(#[from] sqlx::Error),

    #[error("password hash error: {0}")]
    PasswordHashError(#[from] argon2::password_hash::Error),

    #[error("Smtp error: {0}")]
    SmtpError(String),

    #[error("Redis error: {0}")]
    RedisError(#[from] r2d2_redis::redis::RedisError),

    #[error("R2D2 error: {0}")]
    R2D2Error(#[from] r2d2::Error),

    #[error("Email code incorrect")]
    EmailCodeIncorrect,

    #[error("User or password incorrect")]
    PasswordError,

    #[error("User not found")]
    UserNotFound,

    #[error("JwtSimple error: {0}")]
    JwtSimpleErr(#[from] jwt_simple::Error),

    #[error("Chat error: {0}")]
    CreateChatError(String),

    #[error("Chat error: {0}")]
    ChatError(String),

    #[error("Chat not found")]
    ChatNotFound,

    #[error("Get graphql user id error")]
    GetGraphqlUserIdError,

    #[error("Notification error: {0}")]
    NotificationError(String),

    #[error("SerdeJson error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),

    #[error("Unauthorized")]
    Unauthorized,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response<axum::body::Body> {
        let status = match &self {
            Self::EmailAlreadyExists(_) => StatusCode::CONFLICT,
            Self::SqlxError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::PasswordHashError(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::SmtpError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::RedisError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::R2D2Error(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::EmailCodeIncorrect => StatusCode::UNPROCESSABLE_ENTITY,
            Self::JwtSimpleErr(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::PasswordError => StatusCode::FORBIDDEN,
            Self::UserNotFound => StatusCode::NOT_FOUND,
            Self::CreateChatError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::ChatError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::ChatNotFound => StatusCode::NOT_FOUND,
            Self::GetGraphqlUserIdError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::NotificationError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::SerdeJsonError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
        };

        (status, self.to_string()).into_response()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorOutput {
    pub error: String,
}

impl ErrorOutput {
    pub fn new(error: impl Into<String>) -> Self {
        Self {
            error: error.into(),
        }
    }
}
