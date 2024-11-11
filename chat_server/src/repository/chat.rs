use log::debug;
use crate::error::AppError;
use sqlx::{PgPool};
use tracing::field::debug;
use crate::models::{Chat, ChatType, Message, User, UserId};

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

    pub(crate) async fn drop_chat(&self, chat_id: i64, user_id: UserId) -> Result<bool, AppError> {
        let mut tx = self.pool.begin().await?;

        let chat: Chat = sqlx::query_as(
            r#"
            SELECT id, name, type, owner_id, created_at
            FROM chats
            WHERE id = $1
            "#,
        )
            .bind(chat_id)
            .fetch_one(&mut *tx)
            .await?;

        let members: Vec<_> = sqlx::query_as::<_, (i64,)>(
            r#"
            SELECT user_id
            FROM chat_members
            WHERE chat_id = $1
            "#,
        )
            .bind(chat_id)
            .fetch_all(&mut *tx)
            .await?
            .into_iter()
            .map(|(user_id,)| user_id)
            .collect();

        if chat.r#type == ChatType::Group && chat.owner_id != user_id {
            return Err(AppError::ChatError("You are not the owner of group chat".to_string()));
        } else if chat.r#type == ChatType::Private && !members.contains(&user_id) {
            return Err(AppError::ChatError("You are not the member of private chat".to_string()));
        }

        let _ = match sqlx::query(
            r#"
            DELETE FROM chat_members
            WHERE chat_id = $1
            "#,
        )
            .bind(chat_id)
            .execute(&mut *tx)
            .await {
            Ok(_) => {
                debug!("success 1");
            },
            Err(e) => {
                tx.rollback().await?;
                return Err(AppError::SqlxError(e));
            }
        };

        let sql = format!("ALTER TABLE messages DETACH PARTITION zzz_messages_chat_{};", chat_id);
        let _ = match sqlx::query(&sql)
            .execute(&mut *tx)
            .await {
            Ok(_) => {},
            Err(e) => {
                tx.rollback().await?;
                return Err(AppError::SqlxError(e));
            }
        };

        let sql = format!("DROP TABLE zzz_messages_chat_{};", chat_id);
        let _ = match sqlx::query(&sql)
            .execute(&mut *tx)
            .await {
            Ok(_) => {},
            Err(e) => {
                tx.rollback().await?;
                return Err(AppError::SqlxError(e));
            }
        };

        tx.commit().await?;

        Ok(true)
    }

    pub(crate) async fn set_unread_count(&self, chat_id: i64, user_id: UserId, count: i64) -> Result<bool, AppError> {
        let ret = sqlx::query(
            r#"
            UPDATE chat_members
            SET unread_count = $3
            WHERE chat_id = $1 AND user_id = $2
            "#,
        )
            .bind(chat_id)
            .bind(user_id)
            .bind(count)
            .execute(&self.pool)
            .await?;

        Ok(ret.rows_affected() == 1)
    }

    pub(crate) async fn get_unread_count(&self, chat_id: i64, user_id: UserId) -> Result<i32, AppError> {
        let count: (i32,) = sqlx::query_as(
            r#"
            SELECT unread_count
            FROM chat_members
            WHERE chat_id = $1 AND user_id = $2
            "#,
        )
            .bind(chat_id)
            .bind(user_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(count.0)
    }

    pub(crate) async fn get_members(&self, chat_id: i64) -> Result<Vec<User>, AppError> {
        let users: Vec<User> = sqlx::query_as(
            r#"
            SELECT u.id, u.fullname, u.email, u.avatar, u.created_at
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

    pub(crate) async fn get_chat_by_id(&self, id: i64, user_id: UserId) -> Result<Chat, AppError> {
        let chat: Chat = sqlx::query_as(
            r#"
            SELECT c.id, c.name, c.owner_id, c.type, c.created_at
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

    pub(crate) async fn get_all_chats(&self, user_id: UserId) -> Result<Vec<Chat>, AppError> {
        let chats: Vec<Chat> = sqlx::query_as(
            r#"
            SELECT c.id, c.name, c.owner_id, c.type, c.created_at
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

    pub(crate) async fn get_latest_message(&self, chat_id: i64) -> Result<Option<Message>, AppError> {
        let message: Option<Message> = sqlx::query_as(
            r#"
            SELECT id, chat_id, user_id, type, content, created_at
            FROM messages
            WHERE chat_id = $1
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
            .bind(chat_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(message)
    }

    pub(crate) async fn update_chat_name(
        &self,
        name: String,
        chat_id: i64,
        owner_id: UserId,
    ) -> Result<bool, AppError> {
        let ret = sqlx::query(
            r#"
            UPDATE chats
            SET name = $1
            WHERE id = $2 AND owner_id = $3
            RETURNING id
            "#
        )
            .bind(name)
            .bind(chat_id)
            .bind(owner_id)
            .execute(&self.pool)
            .await?;

        Ok(ret.rows_affected() == 1)
    }

    pub(crate) async fn create(
        &self,
        owner_id: UserId,
        mut member_ids: Vec<UserId>,
        mut name: String,
    ) -> Result<Chat, AppError> {
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

        if name.len() == 0 && chat_type == ChatType::Group {
            let query_member_ids: Vec<_> = member_ids.iter().take(3).collect();
            let ret: Result<Vec<User>, _> = sqlx::query_as(
                r#"
            SELECT id, fullname, email, avatar, created_at
            FROM users
            WHERE id = ANY($1)
            "#,
            )
                .bind(query_member_ids)
                .fetch_all(&mut *tx)
                .await;

            match ret {
                Ok(users) => {
                    name = users.iter().map(|u| u.fullname.clone()).collect::<Vec<String>>().join(",");
                    name = format!("{}等的群聊", name);
                }
                Err(err) => {
                    tx.rollback().await?;
                    return Err(AppError::CreateChatError("Can not to create without name".to_string()));
                }
            }
        }

        if chat_type == ChatType::Private {
            let mut another_user_id = owner_id;
            for member_id in member_ids.clone() {
                if member_id != another_user_id {
                    another_user_id = member_id;
                    break;
                }
            }

            let ret: Result<Chat, _> = sqlx::query_as(
                r#"
                SELECT c.id, c.owner_id, c."type", c.name, c.created_at
                FROM chats c
                JOIN chat_members cm
                ON cm.chat_id = c.id
                WHERE c."type" = 'private' AND (c.owner_id = $1 OR c.owner_id = $2) AND cm.user_id = $2
                "#,
            )
                .bind(owner_id)
                .bind(another_user_id)
                .fetch_one(&mut *tx)
                .await;

            debug!("ret: {:?}", ret);

            if let Ok(chat) = ret {
                tx.commit().await?;
                return Ok(chat);
            }
        }


        let ret: Result<Chat, _> = sqlx::query_as(
            r#"
            INSERT INTO chats (owner_id, type, name, created_at)
            VALUES ($1, $2, $3, now())
            RETURNING id, owner_id, type, name, created_at
            "#,
        )
        .bind(owner_id)
        .bind(chat_type)
        .bind(name)
        .fetch_one(&mut *tx)
        .await;

        match ret {
            Ok(chat) => {
                for member_id in member_ids {
                    let ret = sqlx::query(
                        r#"
                INSERT INTO chat_members (chat_id, user_id, created_at)
                VALUES ($1, $2, now())
                "#,
                    )
                    .bind(chat.id)
                    .bind(member_id)
                    .execute(&mut *tx)
                    .await;

                    if let Err(e) = ret {
                        tx.rollback().await?;
                        return Err(AppError::SqlxError(e));
                    }
                }

                // I want this table to be sorted to the end，so named start with zzz
                let plain_sql = format!(
                    r#"
                    CREATE TABLE zzz_messages_chat_{}
                    PARTITION OF messages FOR VALUES IN ({})
                    "#,
                    chat.id,
                    chat.id
                );
                let ret = sqlx::query(plain_sql.as_str()).execute(&mut *tx).await;

                match ret {
                    Ok(_) => {},
                    Err(e) => {
                        tx.rollback().await?;
                        return Err(AppError::SqlxError(e));
                    }
                }

                tx.commit().await?;

                Ok(chat)
            }
            Err(e) => {
                tx.rollback().await?;
                return Err(AppError::SqlxError(e));
            }
        }
    }
}

