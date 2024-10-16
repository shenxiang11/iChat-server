use axum::response::IntoResponse;
use axum::{http, Json, Router};
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{post};
use serde::{Deserialize, Serialize};
use crate::app_state::AppState;
use crate::error::{AppError, ErrorOutput};

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
    let user = state.user_repo.verify_password(&input.email, &input.password).await;

    match user {
        Ok(u) => {
            let token = state.ek.sign(u.id)?;

            Ok((StatusCode::OK, Json(AuthOutput { token })).into_response())
        },
        Err(_) => {
            Ok((StatusCode::FORBIDDEN, Json(ErrorOutput::new("Email or password is incorrect"))).into_response())
        }
    }
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
    Ok((StatusCode::CREATED, Json(user)))
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

#[derive(Debug, Clone, Deserialize, Serialize)]
struct AuthOutput {
    token: String,
}


#[cfg(test)]
mod tests {
    use axum::extract::State;
    use axum::http::StatusCode;
    use axum::Json;
    use axum::response::IntoResponse;
    use crate::app_state::AppState;
    use crate::config::AppConfig;
    use crate::handler::user::{CreateUser, register_routes, send_email_code, SendEmail, signin, SigninUser, signup};

    #[tokio::test]
    async fn register_routes_should_work() {
        let config = AppConfig::load().unwrap();
        let state = AppState::new(config).await;
        let _ = register_routes();
    }

    #[tokio::test]
    async fn handler_send_email_code_should_work() {
        let config = AppConfig::load().unwrap();
        let state = AppState::new(config).await;

        let input = SendEmail {
            email: "sx931210@qq.com".to_string(),
        };

        let ret = send_email_code(State(state.clone()), Json(input)).await.into_response();
        assert_eq!(ret.status(), StatusCode::OK);

        let input = SendEmail {
            email: "863461783@qq.com".to_string(),
        };
        let ret = send_email_code(State(state), Json(input)).await.into_response();
        assert_eq!(ret.status(), StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn handler_signin_should_work() {
        let config = AppConfig::load().unwrap();
        let state = AppState::new(config).await;

        let input = SigninUser {
            email: "863461783@qq.com".to_string(),
            password: "1234567".to_string(),
        };
        let ret = signin(State(state.clone()), Json(input)).await.into_response();
        assert_eq!(ret.status(), StatusCode::FORBIDDEN);

        let input = SigninUser {
            email: "863461783@qq.com".to_string(),
            password: "123456".to_string(),
        };
        let ret = signin(State(state.clone()), Json(input)).await.into_response();
        assert_eq!(ret.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn handler_signup_should_work() {
        let config = AppConfig::load().unwrap();
        let state = AppState::new(config).await;
        let repo = &state.user_repo;

        let input = CreateUser {
            email: "863461783@qq.com".to_string(),
            code: "12345".to_string(),
            password: "123456".to_string(),
            fullname: "Unit Test".to_string(),
        };

        let ret = signup(State(state.clone()), Json(input)).await.into_response();
        assert_eq!(ret.status(), StatusCode::UNPROCESSABLE_ENTITY);

        let code = repo.send_email_code("sx931210@qq.com").await.unwrap();
        let input = CreateUser {
            email: "sx931210@qq.com".to_string(),
            code,
            password: "123456".to_string(),
            fullname: "Unit Test".to_string(),
        };

        let ret = signup(State(state.clone()), Json(input)).await.into_response();
        assert_eq!(ret.status(), StatusCode::CREATED);

        let code = repo.send_email_code("863461783@qq.com").await.unwrap();
        let input = CreateUser {
            email: "863461783@qq.com".to_string(),
            code,
            password: "123456".to_string(),
            fullname: "Unit Test".to_string(),
        };

        let ret = signup(State(state.clone()), Json(input)).await.into_response();
        assert_eq!(ret.status(), StatusCode::CONFLICT);
    }
}
