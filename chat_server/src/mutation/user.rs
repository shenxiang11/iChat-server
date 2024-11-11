use async_graphql::{ComplexObject, Context, InputObject, Object, SimpleObject};
use jwt_simple::prelude::{Deserialize, Serialize};
use tracing_subscriber::filter::combinator::Not;
use crate::app_state::AppState;
use crate::config::AppConfig;
use crate::error::AppError;
use crate::models::{User, UserId};
use crate::notification::{AppEvent, Notification, QRCodeCancel, QRCodeConfirmed, QRCodeScanned};

#[derive(Default)]
pub(crate) struct UserMutation;


#[Object]
impl UserMutation {
    async fn signup(
        &self,
        ctx: &Context<'_>,
        input: CreateUser
    ) -> anyhow::Result<User, AppError> {
        let state = ctx.data_unchecked::<AppState>();

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

    async fn cancel_scanned(&self, ctx: &Context<'_>, device_uuid: String) -> anyhow::Result<bool, AppError> {
        let state = ctx.data_unchecked::<AppState>();
        let _ = ctx.data::<UserId>().map_err(|_| AppError::GetGraphqlUserIdError)?;

        let event = AppEvent::QRCodeCancel(QRCodeCancel {
            device_uuid,
        });

        let ret = state.sender.send(Notification{ event });

        match ret {
            Err(_) => Ok(false),
            Ok(_) => Ok(true),
        }
    }

    async fn scanned(&self, ctx: &Context<'_>, device_uuid: String) -> anyhow::Result<bool, AppError> {
        let state = ctx.data_unchecked::<AppState>();
        let _ = ctx.data::<UserId>().map_err(|_| AppError::GetGraphqlUserIdError)?;

        let event = AppEvent::QRCodeScanned(QRCodeScanned {
            device_uuid,
        });

        let ret = state.sender.send(Notification{ event });

        match ret {
            Err(_) => Ok(false),
            Ok(_) => Ok(true),
        }
    }

    async fn scan_signin(&self, ctx: &Context<'_>, device_uuid: String) -> anyhow::Result<bool, AppError> {
        let state = ctx.data_unchecked::<AppState>();
        let user_id = ctx.data::<UserId>().map_err(|_| AppError::GetGraphqlUserIdError)?;

        let token = state.ek.sign(*user_id, state.config.jwt.period_seconds)?;

        let event = AppEvent::QRCodeConfirmed(QRCodeConfirmed {
            device_uuid,
            token,
        });

        let ret = state.sender.send(Notification{ event });

        match ret {
            Err(_) => Ok(false),
            Ok(_) => Ok(true),
        }
    }

    async fn signin(
        &self,
        ctx: &Context<'_>,
        input: SigninUser,
    ) -> anyhow::Result<AuthOutput, AppError> {
        let state = ctx.data_unchecked::<AppState>();
        let config = &state.config;
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
        ctx: &Context<'_>,
        input: SendEmail
    ) -> anyhow::Result<MessageOutput, AppError> {
        let state = ctx.data_unchecked::<AppState>();
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
    async fn user(&self, ctx: &Context<'_>) -> anyhow::Result<User, AppError> {
        let state = ctx.data_unchecked::<AppState>();
        let user = state.user_repo.find_by_id(self.user_id).await?;

        match user {
            None => return Err(AppError::ChatNotFound),
            Some(u) => return Ok(u)
        }
    }
}
