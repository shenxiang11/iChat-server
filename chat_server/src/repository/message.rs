use sqlx::PgPool;
use crate::error::AppError;
use crate::models::{Message, MessageType, UserId};

pub struct MessageRepository {
    biz: String,
    pub(crate) pool: PgPool,
}

impl MessageRepository {
    pub(crate) fn new(pool: PgPool) -> Self {
        Self {
            biz: "user".to_string(),
            pool,
        }
    }

    pub(crate) async fn get_messages(&self, chat_id: i64, user_id: UserId) -> Result<Vec<Message>, AppError> {
        let messages: Vec<Message> = sqlx::query_as(
            r#"
            SELECT m.id, m.chat_id, m.user_id, m.content, m.created_at
            FROM messages m
            JOIN chat_members cm ON m.chat_id = cm.chat_id
            WHERE m.chat_id = $1 AND cm.user_id = $2
            "#,
        )
            .bind(chat_id)
            .bind(user_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(messages)
    }

    pub(crate) async fn create_message(&self, chat_id: i64, user_id: UserId, r#type: MessageType, content: String) -> Result<Message, AppError> {
        let message: Message = sqlx::query_as(
            r#"
            INSERT INTO messages (chat_id, user_id, type, content)
            VALUES ($1, $2, $3, $4)
            RETURNING id, chat_id, user_id, type, content, created_at
            "#,
        )
            .bind(chat_id)
            .bind(user_id)
            .bind(r#type)
            .bind(content)
            .fetch_one(&self.pool)
            .await?;

        Ok(message)
    }
}
