use async_graphql::{Context, InputObject, Object};
use anyhow::Result;
use jwt_simple::prelude::{Deserialize, Serialize};
use crate::app_state::AppState;
use crate::error::AppError;
use crate::models::{Message, MessageType, UserId};

#[derive(Default)]
pub(crate) struct MessageQuery;

#[derive(Default)]
pub(crate) struct MessageMutation;

#[Object]
impl MessageQuery {
    async fn get_messages(
        &self,
        ctx: &Context<'_>,
        chat_id: i64,
        cursor_id: Option<i64>,
    ) -> Result<Vec<Message>, AppError> {
        let state = AppState::shared().await;
        let user_id = ctx.data::<UserId>().map_err(|_| AppError::GetGraphqlUserIdError)?;

        let messages = state.message_repo.get_messages(chat_id, *user_id, cursor_id).await?;

        Ok(messages)
    }
}

#[Object]
impl MessageMutation {
    async fn send_message(
        &self,
        ctx: &Context<'_>,
        input: CreateMessage
    ) -> Result<Message, AppError> {
        let state = AppState::shared().await;
        let user_id = ctx.data::<UserId>().map_err(|_| AppError::GetGraphqlUserIdError)?;

        let message = state.message_repo.create_message(input.chat_id, *user_id, MessageType::Text, input.content).await?;

        Ok(message)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, InputObject)]
struct CreateMessage {
    chat_id: i64,
    content: String,
}
