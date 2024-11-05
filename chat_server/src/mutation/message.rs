use async_graphql::{Context, InputObject, Object};
use jwt_simple::prelude::{Deserialize, Serialize};
use crate::app_state::AppState;
use crate::error::AppError;
use crate::models::{Message, MessageType, UserId};

#[derive(Default)]
pub(crate) struct MessageMutation;

#[Object]
impl MessageMutation {
    async fn send_message(
        &self,
        ctx: &Context<'_>,
        input: CreateMessage
    ) -> anyhow::Result<Message, AppError> {
        let state = ctx.data_unchecked::<AppState>();
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
