use async_graphql::{Context, ErrorExtensions, InputObject, Object};
use anyhow::Result;
use jwt_simple::prelude::{Deserialize, Serialize};
use crate::app_state::AppState;
use crate::error::AppError;
use crate::models::{Message, MessageType, UserId};

#[derive(Default)]
pub(crate) struct MessageQuery;

#[Object]
impl MessageQuery {
    async fn get_messages(
        &self,
        ctx: &Context<'_>,
        chat_id: i64,
        cursor_id: Option<i64>,
    ) -> Result<Vec<Message>, AppError> {
        let state = ctx.data_unchecked::<AppState>();
        let user_id = ctx.data::<UserId>().map_err(|_| AppError::GetGraphqlUserIdError)?;

        let messages = state.message_repo.get_messages(chat_id, *user_id, cursor_id).await?;

        Ok(messages)
    }
}
