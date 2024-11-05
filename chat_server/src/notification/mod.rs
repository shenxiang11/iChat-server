use std::sync::Arc;
use async_graphql::{OutputType, SimpleObject, Union};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgListener;
use tokio::sync::broadcast;
use tokio_stream::StreamExt;
use tracing::error;
use crate::app_state::AppState;
use crate::config::AppConfig;
use crate::error::AppError;
use crate::handler::MutationType;
use crate::models::{Chat, Message};

pub(crate) async fn setup_pg_listener(state: AppState) -> anyhow::Result<()> {
    let config = &state.config;;
    let mut listener = PgListener::connect(config.server.postgres_url.as_str()).await?;

    listener.listen("chat_change").await?;
    listener.listen("new_message").await?;

    let mut stream = listener.into_stream();

    tokio::spawn(async move {
        while let Some(Ok(notification)) = stream.next().await {
            let noti = Notification::load(notification.channel(), notification.payload());

            match noti {
                Ok(noti) => {
                    if state.sender.send(noti).is_err() {
                        error!("Failed to send notification to channel");
                    }
                },
                Err(e) => {
                    error!("Failed to parse notification: {:?}", e);
                    continue;
                }
            }
        }
    });

    Ok(())
}

#[derive(Debug, Serialize, Deserialize, Union, Clone)]
#[serde(tag = "event")]
pub(crate) enum AppEvent {
    CreatedChat(CreatedChat),
    ChatOwnerChanged(ChatOwnerChanged),
    ChatNameChanged(ChatNameChanged),
    ChatDeleted(ChatDeleted),
    NewMessage(Message),
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub(crate) struct CreatedChat {
    pub(crate) data: Chat,
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub(crate) struct ChatOwnerChanged {
    pub(crate) data: Chat,
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub(crate) struct ChatNameChanged {
    pub(crate) data: Chat,
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub(crate) struct ChatDeleted {
    pub(crate) data: Chat,
}

#[derive(Debug, Clone)]
pub(crate) struct Notification {
    pub(crate) event: AppEvent,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatUpdated {
    op: String,
    old: Option<Chat>,
    new: Option<Chat>,
}

#[derive(Debug, Serialize, Deserialize, SimpleObject)]
struct SubscriptionPayload<T>
where
    T: Serialize + Send + Sync + OutputType,
{
    mutation_type: MutationType,
    data: T,
}

impl Notification {
    pub(crate) fn load(r#type: &str, payload: &str) -> Result<Self, AppError> {
        let event = match r#type {
            "chat_change" => Self::handle_chat_change(payload)?,
            "new_message" => Self::handle_new_message(payload)?,
            _ => {
                return Err(AppError::NotificationError("Invalid operation".to_string()));
            }
        };

        Ok(Self { event })
    }

    pub(crate) fn handle_chat_change(payload: &str) -> Result<AppEvent, AppError> {
        let payload: ChatUpdated = serde_json::from_str(payload)?;

        let event = match payload.op.as_str() {
            "INSERT" => match payload.new {
                Some(new) => AppEvent::CreatedChat(CreatedChat { data: new }),
                None => {
                    return Err(AppError::NotificationError("Invalid operation".to_string()));
                }
            },
            "UPDATE" => match (payload.old, payload.new) {
                (Some(old), Some(new)) => {
                    if old.owner_id != new.owner_id {
                        AppEvent::ChatOwnerChanged(ChatOwnerChanged { data: new })
                    } else if old.name != new.name {
                        AppEvent::ChatNameChanged(ChatNameChanged { data: new })
                    } else {
                        return Err(AppError::NotificationError("Invalid operation".to_string()));
                    }
                }
                _ => {
                    return Err(AppError::NotificationError("Invalid operation".to_string()));
                }
            },
            "DELETE" => match payload.old {
                Some(old) => AppEvent::ChatDeleted(ChatDeleted { data: old }),
                None => {
                    return Err(AppError::NotificationError("Invalid operation".to_string()));
                }
            },
            _ => {
                return Err(AppError::NotificationError("Invalid operation".to_string()));
            }
        };

        Ok(event)
    }

    pub(crate) fn handle_new_message(payload: &str) -> Result<AppEvent, AppError> {
        let message: Message = serde_json::from_str(payload)?;

        Ok(AppEvent::NewMessage(message))
    }
}
