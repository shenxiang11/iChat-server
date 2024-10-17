use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, PartialEq)]
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


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd, sqlx::Type)]
#[sqlx(type_name = "chat_type", rename_all = "snake_case")]
#[serde(rename_all(serialize = "camelCase"))]
pub enum ChatType {
    Private,
    Group,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Chat {
    id: i64,
    owner_id: i64,
    r#type: ChatType,
    created_at: DateTime<Utc>,
}
