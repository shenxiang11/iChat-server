use axum::response::IntoResponse;
use axum::{Json, Router};
use axum::extract::State;
use axum::routing::{post};
use serde::{Deserialize, Serialize};
use crate::app_state::AppState;
use crate::error::AppError;

pub(crate) fn register_routes() -> Router<AppState> {
    Router::new()
        .route("/email_code", post(send_email_code))
        .route("/signin", post(signin))
        .route("/signup", post(signup))
}

pub(crate) async fn send_email_code(
    State(state): State<AppState>,
    Json(input): Json<SendEmail>,
) -> Result<impl IntoResponse, AppError> {
    let user = state.user_repo.find_by_email(&input.email).await?;

    if user.is_some() {
        return Err(AppError::EmailAlreadyExists(input.email));
    }

    state.user_repo.send_email_code(&input.email).await?;
    Ok("Send email code")
}

pub(crate) async fn signin(
    State(state): State<AppState>,
    Json(input): Json<SigninUser>,
) -> Result<impl IntoResponse, AppError> {
    Ok("Signin")
}

pub(crate) async fn signup(
    State(state): State<AppState>,
    Json(input): Json<CreateUser>,
) -> Result<impl IntoResponse, AppError> {
    let is_code_correct = state.user_repo.verify_email_code(&input.email, &input.code).await?;

    if !is_code_correct {
        return Err(AppError::EmailCodeIncorrect);
    }

    let user = state.user_repo.find_by_email(&input.email).await?;

    if user.is_some() {
        return Err(AppError::EmailAlreadyExists(input.email));
    }

    let user = state.user_repo.create(&input.email, &input.password, &input.fullname).await?;
    Ok(Json(user))
}


#[derive(Debug, Clone, Deserialize, Serialize)]
struct SendEmail {
    email: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct CreateUser {
    email: String,
    code: String,
    password: String,
    fullname: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct SigninUser {
    email: String,
    password: String,
}
