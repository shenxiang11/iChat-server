use async_graphql::{ComplexObject, Context, InputObject, Object, SimpleObject};
use jwt_simple::prelude::{Deserialize, Serialize};
use anyhow::Result;
use crate::app_state::AppState;
use crate::config::AppConfig;
use crate::error::{AppError};
use crate::models::{User, UserId};

#[derive(Default)]
pub(crate) struct UserQuery;

#[derive(Default)]
pub(crate) struct UserMutation;

#[Object]
impl UserQuery {
    async fn get_users(
        &self,
        ctx: &Context<'_>,
    ) ->Result<Vec<User>, AppError> {
        let _user_id = ctx.data::<UserId>().map_err(|_| AppError::GetGraphqlUserIdError)?;

        let state = AppState::shared().await;
        let users = state.user_repo.get_all_users().await?;

        Ok(users)
    }
    async fn get_self(&self, ctx: &Context<'_>) -> Result<User, AppError> {
        let user_id = ctx.data::<UserId>().map_err(|_| AppError::GetGraphqlUserIdError)?;

        let state = AppState::shared().await;
        let user = state.user_repo.find_by_id(*user_id).await?;

        match user {
            None => return Err(AppError::GetGraphqlUserIdError),
            Some(u) => return Ok(u)
        }
    }
}

#[Object]
impl UserMutation {
    async fn signup(
        &self,
        _ctx: &Context<'_>,
        input: CreateUser
    ) -> Result<User, AppError> {
        let state = AppState::shared().await;

        let is_code_correct = state.user_repo.verify_email_code(&input.email, &input.code).await?;

        if !is_code_correct {
            return Err(AppError::EmailCodeIncorrect);
        }

        let user = state.user_repo.find_by_email(&input.email).await?;

        if user.is_some() {
            return Err(AppError::EmailAlreadyExists(input.email));
        }

        let user = state.user_repo.create(&input.email, &input.password, &input.fullname).await?;
        Ok(user)
    }

    async fn signin(
        &self,
        _ctx: &Context<'_>,
        input: SigninUser
    ) -> Result<AuthOutput, AppError> {
        let config = AppConfig::shared().await;
        let state = AppState::shared().await;
        let user = state.user_repo.verify_password(&input.email, &input.password).await;

        match user {
            Ok(u) => {
                let token = state.ek.sign(u.id, config.jwt.period_seconds)?;

                Ok(AuthOutput {
                    token,
                    user_id: u.id
                })
            },
            Err(_) => {
                Err(AppError::UserNotFound)
            }
        }
    }

    async fn send_email(
        &self,
        _ctx: &Context<'_>,
        input: SendEmail
    ) -> Result<MessageOutput, AppError> {
        let state = AppState::shared().await;
        let _ = state.user_repo.send_email_code(&input.email).await?;

        Ok(MessageOutput {
            message: "Send success.".to_string(),
        })
    }
}


#[derive(Debug, Clone, Deserialize, Serialize, InputObject)]
struct SendEmail {
    email: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, InputObject)]
struct CreateUser {
    email: String,
    code: String,
    password: String,
    fullname: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, InputObject)]
struct SigninUser {
    email: String,
    password: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, SimpleObject)]
#[graphql(complex)]
struct AuthOutput {
    token: String,
    user_id: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, SimpleObject)]
struct MessageOutput {
    message: String,
}

#[ComplexObject]
impl AuthOutput {
    async fn user(&self, _ctx: &Context<'_>) -> Result<User, AppError> {
        let state = AppState::shared().await;
        let user = state.user_repo.find_by_id(self.user_id).await?;

        match user {
            None => return Err(AppError::ChatNotFound),
            Some(u) => return Ok(u)
        }
    }
}
