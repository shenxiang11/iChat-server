use std::sync::Arc;
use axum::response::IntoResponse;
use axum::{Json, Router};
use axum::routing::{get, post};
use redis::Connection;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use crate::error::AppError;
use crate::repository::UserRepository;

pub(crate) struct UserHandler {
    repo: UserRepository,
}

impl UserHandler {
    pub(crate) fn new(pool: PgPool) -> Arc<Self> {
        Arc::new(Self {
            repo: UserRepository::new(pool),
        })
    }

    pub(crate) fn register_routes(self: Arc<Self>) -> Router {
        Router::new()
            .route("/email_code", post({
                let mut h = self.clone();
                move |body| async move {
                    h.send_email_code(body).await
                }
            }))
            .route("/signin", post({
                let h = self.clone();
                move |body| async move {
                    h.signin(body).await
                }
            }))
            .route("/signup", get({
                let h = self.clone();
                move |body| async move {
                    h.signup(body).await
                }
            }))
    }

    pub(crate) async fn send_email_code(
        &self,
        Json(input): Json<SendEmail>,
    ) -> Result<impl IntoResponse, AppError> {
        let user = self.repo.find_by_email(&input.email).await?;

        if user.is_some() {
            return Err(AppError::EmailAlreadyExists(input.email));
        }

        self.repo.send_email_code(&input.email).await?;

        Ok(Json("Email code sent!"))
    }

    pub(crate) async fn signin(
        &self,
        Json(input): Json<CreateUser>,
    ) -> Result<impl IntoResponse, AppError> {
        let user = self.repo.create(&input.email, &input.password, &input.fullname).await?;
        Ok(Json(user))
    }

    pub(crate) async fn signup(
        &self,
        Json(input): Json<SigninUser>,
    ) -> impl IntoResponse {
        Json(input)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct SendEmail {
    email: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct CreateUser {
    email: String,
    password: String,
    fullname: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct SigninUser {
    email: String,
    password: String,
}
