use sqlx::PgPool;
use crate::error::AppError;
use crate::models::{Chat, Message, MessageType, UserId};

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

    pub(crate) async fn get_messages(&self, chat_id: i64, user_id: UserId, cursor_id: Option<i64>) -> Result<Vec<Message>, AppError> {
        let mut messages: Vec<Message> = sqlx::query_as(
            r#"
            SELECT m.id, m.chat_id, m.user_id, m.type, m.content, m.created_at
            FROM messages m
            JOIN chat_members cm ON m.chat_id = cm.chat_id
            WHERE m.chat_id = $1 AND cm.user_id = $2
            AND m.id < $3
            ORDER BY m.created_at DESC
            LIMIT 20
            "#,
        )
            .bind(chat_id)
            .bind(user_id)
            .bind(cursor_id.unwrap_or(i64::MAX))
            .fetch_all(&self.pool)
            .await?;

        messages.reverse();

        Ok(messages)
    }

    pub(crate) async fn create_message(&self, chat_id: i64, user_id: UserId, r#type: MessageType, content: String) -> Result<Message, AppError> {
        let chat: Option<Chat> = sqlx::query_as(
            r#"
            SELECT c.id, c.name, c.owner_id, c.type, c.created_at
            FROM chats c
            JOIN chat_members cm ON c.id = cm.chat_id
            WHERE c.id = $1 AND cm.user_id = $2
            "#,
        )
            .bind(chat_id)
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?;

        if chat.is_none() {
            return Err(AppError::ChatError("Cant not send message to chat".to_string()));
        }

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
