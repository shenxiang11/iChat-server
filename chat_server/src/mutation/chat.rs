use async_graphql::{Context, Object};
use crate::app_state::AppState;
use crate::error::AppError;
use crate::models::{Chat, UserId};

#[derive(Default)]
pub(crate) struct ChatMutation;

#[Object]
impl ChatMutation {
    async fn update_chat_name(
        &self,
        ctx: &Context<'_>,
        chat_id: i64,
        name: String,
    ) -> Result<bool, AppError> {
        let state = ctx.data_unchecked::<AppState>();
        let user_id = ctx.data::<UserId>().map_err(|_| AppError::GetGraphqlUserIdError)?;

        state.chat_repo.update_chat_name(name, chat_id, *user_id).await
    }

    async fn drop_chat(
        &self,
        ctx: &Context<'_>,
        id: i64
    ) -> Result<bool, AppError> {
        let state = ctx.data_unchecked::<AppState>();
        let user_id = ctx.data::<UserId>().map_err(|_| AppError::GetGraphqlUserIdError)?;

        let res = state.chat_repo.drop_chat(id, *user_id).await?;

        Ok(res)
    }

    async fn create_chat(
        &self,
        ctx: &Context<'_>,
        member_ids: Vec<UserId>
    ) -> Result<Chat, AppError> {
        let state = ctx.data_unchecked::<AppState>();
        let user_id = ctx.data::<UserId>().map_err(|_| AppError::GetGraphqlUserIdError)?;

        let chat = state.chat_repo.create(*user_id, member_ids, "".to_string()).await?;

        Ok(chat)
    }

    async fn chat_read(
        &self,
        ctx: &Context<'_>,
        chat_id: i64
    ) -> Result<bool, AppError> {
        let state = ctx.data_unchecked::<AppState>();
        let user_id = ctx.data::<UserId>().map_err(|_| AppError::GetGraphqlUserIdError)?;

        let res = state.chat_repo.set_unread_count(chat_id, *user_id, 0).await?;

        Ok(res)
    }
}
