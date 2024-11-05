use async_graphql::{Context, ErrorExtensions, Object};

use crate::app_state::AppState;
use crate::error::AppError;
use crate::models::{Chat, UserId};

#[derive(Default)]
pub(crate) struct ChatQuery;

#[derive(Default)]
pub(crate) struct ChatMutation;

#[Object]
impl ChatQuery {
    async fn get_chat(
        &self,
        ctx: &Context<'_>,
        id: i64
    ) -> Result<Chat, AppError> {
        let state = AppState::shared().await;
        let user_id = ctx.data::<UserId>().map_err(|_| AppError::GetGraphqlUserIdError)?;

        let res = state.chat_repo.get_chat_by_id(id, *user_id).await;

        match res {
            Ok(chat) => Ok(chat),
            Err(_) => Err(AppError::ChatNotFound)
        }
    }

    async fn get_chats(&self, ctx: &Context<'_>) -> Result<Vec<Chat>, AppError> {
        let state = AppState::shared().await;
        let user_id = ctx.data::<UserId>().map_err(|_| AppError::GetGraphqlUserIdError)?;

        println!("user_id: {:?}", user_id);

        let res = state.chat_repo.get_all_chats(*user_id).await;

        match res {
            Ok(chats) => Ok(chats),
            Err(_) => Ok(vec![])
        }
    }
}

#[Object]
impl ChatMutation {
    async fn drop_chat(
        &self,
        ctx: &Context<'_>,
        id: i64
    ) -> Result<bool, AppError> {
        let state = AppState::shared().await;
        let user_id = ctx.data::<UserId>().map_err(|_| AppError::GetGraphqlUserIdError)?;

        let res = state.chat_repo.drop_chat(id, *user_id).await?;

        Ok(res)
    }

    async fn create_chat(
        &self,
        ctx: &Context<'_>,
        member_ids: Vec<UserId>
    ) -> Result<Chat, AppError> {
        let state = AppState::shared().await;
        let user_id = ctx.data::<UserId>().map_err(|_| AppError::GetGraphqlUserIdError)?;

        let chat = state.chat_repo.create(*user_id, member_ids, "".to_string()).await?;

        Ok(chat)
    }

    async fn chat_read(
        &self,
        ctx: &Context<'_>,
        chat_id: i64
    ) -> Result<bool, AppError> {
        let state = AppState::shared().await;
        let user_id = ctx.data::<UserId>().map_err(|_| AppError::GetGraphqlUserIdError)?;

        let res = state.chat_repo.set_unread_count(chat_id, *user_id, 0).await?;

        Ok(res)
    }
}
