use axum::http::{Response, StatusCode};
use axum::response::IntoResponse;
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
    RedisError(#[from] redis::RedisError),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response<axum::body::Body> {
        let status = match &self {
            Self::EmailAlreadyExists(_) => StatusCode::CONFLICT,
            Self::SqlxError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::PasswordHashError(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::SmtpError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::RedisError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, self.to_string()).into_response()
    }
}
