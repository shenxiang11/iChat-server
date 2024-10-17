use crate::error::AppError;
use sqlx::{PgPool};
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

    pub(crate) async fn get_members(&self, chat_id: i64) -> Result<Vec<User>, AppError> {
        let users: Vec<User> = sqlx::query_as(
            r#"
            SELECT u.id, u.fullname, u.email, u.created_at
            FROM users u
            JOIN chat_members cm ON u.id = cm.user_id
            WHERE cm.chat_id = $1
            "#,
        )
            .bind(chat_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(users)
    }

    pub(crate) async fn get_chat_by_id(&self, id: i64, user_id: i64) -> Result<Chat, AppError> {
        let chat: Chat = sqlx::query_as(
            r#"
            SELECT c.id, c.owner_id, c.type, c.created_at
            FROM chats c
            JOIN chat_members cm ON c.id = cm.chat_id
            WHERE c.id = $1 AND cm.user_id = $2
            "#,
        )
            .bind(id)
            .bind(user_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(chat)
    }

    pub(crate) async fn get_all_chats(&self, user_id: i64) -> Result<Vec<Chat>, AppError> {
        let chats: Vec<Chat> = sqlx::query_as(
            r#"
            SELECT c.id, c.owner_id, c.type, c.created_at
            FROM chats c
            JOIN chat_members cm ON c.id = cm.chat_id
            WHERE cm.user_id = $1
            "#,
        )
            .bind(user_id)
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

