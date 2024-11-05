use async_graphql::{Context, Object};
use anyhow::Result;
use crate::app_state::AppState;
use crate::error::{AppError};
use crate::models::{User, UserId};

#[derive(Default)]
pub(crate) struct UserQuery;

#[Object]
impl UserQuery {
    async fn get_users(
        &self,
        ctx: &Context<'_>,
    ) ->Result<Vec<User>, AppError> {
        let state = ctx.data_unchecked::<AppState>();
        let _user_id = ctx.data::<UserId>().map_err(|_| AppError::GetGraphqlUserIdError)?;

        let users = state.user_repo.get_all_users().await?;

        Ok(users)
    }
    async fn get_self(&self, ctx: &Context<'_>) -> Result<User, AppError> {
        let state = ctx.data_unchecked::<AppState>();
        let user_id = ctx.data::<UserId>().map_err(|_| AppError::GetGraphqlUserIdError)?;

        let user = state.user_repo.find_by_id(*user_id).await?;

        match user {
            None => return Err(AppError::GetGraphqlUserIdError),
            Some(u) => return Ok(u)
        }
    }
}
