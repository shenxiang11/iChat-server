use async_graphql::{ComplexObject, Context, Enum, Object, SimpleObject};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, PartialEq, SimpleObject)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: i64,
    pub fullname: String,
    pub email: String,
    #[sqlx(default)]
    #[serde(skip)]
    pub password_hash: Option<String>,
    pub created_at: DateTime<Utc>,
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd, sqlx::Type, Enum, Copy, Eq)]
#[sqlx(type_name = "chat_type", rename_all = "snake_case")]
#[serde(rename_all(serialize = "camelCase"))]
pub enum ChatType {
    Private,
    Group,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, PartialEq, SimpleObject)]
#[graphql(complex)]
#[serde(rename_all = "camelCase")]
pub struct Chat {
    pub(crate) id: i64,
    pub(crate) owner_id: i64,
    pub(crate) r#type: ChatType,
    pub(crate) created_at: DateTime<Utc>,
}

#[ComplexObject]
impl Chat {
    async fn owner(&self, ctx: &Context<'_>) -> anyhow::Result<Option<User>, AppError> {
        let state = ctx.data::<crate::AppState>().unwrap();
        let user = state.user_repo.find_by_id(self.owner_id).await?;
        Ok(user)
    }

    async fn members(&self, ctx: &Context<'_>) -> anyhow::Result<Vec<User>, AppError> {
        let state = ctx.data::<crate::AppState>().unwrap();
        let users = state.chat_repo.get_members(self.id).await?;
        Ok(users)
    }
}
