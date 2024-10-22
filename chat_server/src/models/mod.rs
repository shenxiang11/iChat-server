use async_graphql::{ComplexObject, Enum, InputObject, SimpleObject};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use anyhow::Result;

use crate::app_state::AppState;
use crate::error::AppError;

pub type UserId = i64;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, PartialEq, SimpleObject)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: UserId,
    pub fullname: String,
    pub email: String,
    #[sqlx(default)]
    #[serde(skip)]
    pub password_hash: Option<String>,
    pub created_at: DateTime<Utc>,
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd, sqlx::Type, Enum, Copy, Eq)]
#[sqlx(type_name = "chat_type", rename_all = "snake_case")]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub enum ChatType {
    Private,
    Group,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, PartialEq, SimpleObject, InputObject)]
#[graphql(complex)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
pub struct Chat {
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) owner_id: UserId,
    pub(crate) r#type: ChatType,
    pub(crate) created_at: DateTime<Utc>,
}

#[ComplexObject]
impl Chat {
    async fn owner(&self) -> Result<User, AppError> {
        let state = AppState::shared().await;
        let user = state.user_repo.find_by_id(self.owner_id).await?;

        match user {
            Some(user) => Ok(user),
            None => Err(AppError::UserNotFound),
        }
    }

    async fn latest_message(&self) -> Result<Option<Message>, AppError> {
        let state = AppState::shared().await;
        let message = state.chat_repo.get_latest_message(self.id).await?;
        Ok(message)
    }

    async fn members(&self) -> Result<Vec<User>, AppError> {
        let state = AppState::shared().await;
        let users = state.chat_repo.get_members(self.id).await?;
        Ok(users)
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd, sqlx::Type, Enum, Copy, Eq)]
#[sqlx(type_name = "message_type", rename_all = "snake_case")]
#[serde(rename_all(serialize = "camelCase"))]
pub enum MessageType {
    Text,
    Image,
    Video,
    Audio,
    File,
}


#[derive(Debug, Clone, Serialize, Deserialize, FromRow, PartialEq, SimpleObject)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub id: i64,
    pub chat_id: i64,
    pub user_id: UserId,
    pub r#type: MessageType,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

