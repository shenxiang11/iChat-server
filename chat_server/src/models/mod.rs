mod chat;

use async_graphql::{ComplexObject, Context, Enum, SimpleObject};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use anyhow::Result;

use crate::app_state::AppState;
use crate::error::AppError;
pub(crate) use chat::*;

pub type UserId = i64;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, PartialEq, SimpleObject)]
#[graphql(complex)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: UserId,
    pub fullname: String,
    pub email: String,
    #[sqlx(default)]
    #[serde(skip)]
    pub password_hash: Option<String>,
    pub avatar: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[ComplexObject]
impl User {
    async fn is_self(&self, ctx: &Context<'_>) -> Result<bool, AppError> {
        let user_id = ctx
            .data::<UserId>()
            .map_err(|_| AppError::GetGraphqlUserIdError)?;

        Ok(self.id == *user_id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd, sqlx::Type, Enum, Copy, Eq)]
#[sqlx(type_name = "chat_type", rename_all = "snake_case")]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub enum ChatType {
    Private,
    Group,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd, sqlx::Type, Enum, Copy, Eq)]
#[sqlx(type_name = "message_type", rename_all = "snake_case")]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub enum MessageType {
    Text,
    Image,
    Video,
    Audio,
    File,
}


#[derive(Debug, Clone, Serialize, Deserialize, FromRow, PartialEq, SimpleObject)]
#[graphql(complex)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
pub struct Message {
    pub id: i64,
    pub chat_id: i64,
    pub user_id: UserId,
    pub r#type: MessageType,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[ComplexObject]
impl Message {
    async fn user(&self, ctx: &Context<'_>) -> Result<User, AppError> {
        let state = ctx.data_unchecked::<AppState>();
        let user = state.user_repo.find_by_id(self.user_id).await?;

        match user {
            Some(user) => Ok(user),
            None => Err(AppError::UserNotFound),
        }
    }

    async fn is_mine(&self, ctx: &Context<'_>) -> Result<bool, AppError> {
        let user_id = ctx
            .data::<UserId>()
            .map_err(|_| AppError::GetGraphqlUserIdError)?;

        Ok(self.user_id == *user_id)
    }
}

impl Message {
    pub(crate) async fn get_chat(&self, ctx: &Context<'_>) -> Result<Chat, AppError> {
        let state = ctx.data_unchecked::<AppState>();
        let chat = state.chat_repo.get_chat_by_id(self.chat_id, self.user_id).await?;

        Ok(chat)
    }
}
