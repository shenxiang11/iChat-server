use crate::error::AppError;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres, Transaction};
use crate::models::{Chat, ChatType, User};

pub struct ChatRepository {
    biz: String,
    pub(crate) pool: PgPool,
}

impl ChatRepository {
    pub(crate) fn new(pool: PgPool) -> Self {
        Self {
            biz: "chat".to_string(),
            pool,
        }
    }

    pub(crate) async fn get_all_chats(&self) -> Result<Vec<Chat>, AppError> {
        let chats: Vec<Chat> = sqlx::query_as(
            r#"
            SELECT id, owner_id, type, created_at
            FROM chats
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(chats)
    }

    pub(crate) async fn create(
        &self,
        owner_id: i64,
        mut member_ids: Vec<i64>,
    ) -> Result<i64, AppError> {
        if !member_ids.contains(&owner_id) {
            member_ids.push(owner_id);
        }

        if member_ids.len() < 2 {
            return Err(AppError::CreateChatError(
                "Member ids must be at least 2".to_string(),
            ));
        }

        let chat_type = if member_ids.len() == 2 {
            ChatType::Private
        } else {
            ChatType::Group
        };

        let mut tx = self.pool.begin().await?;

        let ret: Result<i64, _> = sqlx::query_scalar(
            r#"
            INSERT INTO chats (owner_id, type, created_at)
            VALUES ($1, $2, now())
            RETURNING id
            "#,
        )
        .bind(owner_id)
        .bind(chat_type)
        .fetch_one(&mut *tx)
        .await;

        match ret {
            Ok(chat_id) => {
                for member_id in member_ids {
                    let ret = sqlx::query(
                        r#"
                INSERT INTO chat_members (chat_id, user_id, created_at)
                VALUES ($1, $2, now())
                "#,
                    )
                    .bind(chat_id)
                    .bind(member_id)
                    .execute(&mut *tx)
                    .await;

                    if let Err(e) = ret {
                        tx.rollback().await?;
                        return Err(AppError::SqlxError(e));
                    }
                }

                tx.commit().await?;

                Ok(chat_id)
            }
            Err(e) => {
                tx.rollback().await?;
                return Err(AppError::SqlxError(e));
            }
        }
    }
}
