use async_graphql::{ComplexObject, Context, InputObject, SimpleObject};
use chrono::{DateTime, Utc};
use jwt_simple::prelude::{Deserialize, Serialize};
use sqlx::FromRow;
use crate::app_state::AppState;
use crate::error::AppError;
use crate::models::{ChatType, Message, User, UserId};

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
    async fn display_name(&self, ctx: &Context<'_>) -> anyhow::Result<String, AppError> {
        let user_id = ctx
            .data::<UserId>()
            .map_err(|_| AppError::GetGraphqlUserIdError)?;

        if self.r#type == ChatType::Private {
            let members = self.members(ctx).await?;
            let member = members.into_iter().find(|m| m.id != *user_id);
            match member {
                Some(member) => Ok(member.fullname),
                None => Err(AppError::UserNotFound),
            }
        } else {
            Ok(self.name.clone())
        }
    }

    async fn original_9_users(&self, ctx : &Context<'_>) -> anyhow::Result<Vec<User>, AppError> {
        let state = ctx.data_unchecked::<AppState>();
        let users = state.chat_repo.get_members(self.id).await?;
        let users = users.into_iter().take(9).collect();

        Ok(users)
    }

    async fn is_owner(&self, ctx : &Context<'_>) -> anyhow::Result<bool, AppError> {
        let user_id = ctx
            .data::<UserId>()
            .map_err(|_| AppError::GetGraphqlUserIdError)?;

        Ok(self.owner_id == *user_id)
    }

    async fn owner(&self, ctx : &Context<'_>) -> anyhow::Result<User, AppError> {
        let state = ctx.data_unchecked::<AppState>();
        let user = state.user_repo.find_by_id(self.owner_id).await?;

        match user {
            Some(user) => Ok(user),
            None => Err(AppError::UserNotFound),
        }
    }

    async fn latest_message(&self, ctx : &Context<'_>) -> anyhow::Result<Option<Message>, AppError> {
        let state = ctx.data_unchecked::<AppState>();
        let message = state.chat_repo.get_latest_message(self.id).await?;
        Ok(message)
    }

    async fn members(&self, ctx : &Context<'_>) -> anyhow::Result<Vec<User>, AppError> {
        let state = ctx.data_unchecked::<AppState>();
        let users = state.chat_repo.get_members(self.id).await?;
        Ok(users)
    }

    async fn unread_count(&self, ctx: &Context<'_>) -> anyhow::Result<i32, AppError> {
        let user_id = ctx
            .data::<UserId>()
            .map_err(|_| AppError::GetGraphqlUserIdError)?;

        let state = ctx.data_unchecked::<AppState>();
        let count = state.chat_repo.get_unread_count(self.id, *user_id).await?;
        Ok(count)
    }
}
