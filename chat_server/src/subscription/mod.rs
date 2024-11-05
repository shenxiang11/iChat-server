use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use async_graphql::{Context, Subscription};
use async_graphql::futures_util::Stream;
use tokio::sync::broadcast;
use tracing::debug;
use crate::app_state::AppState;
use crate::error::AppError;
use crate::models::{Chat, Message, UserId};
use crate::notification::{AppEvent, Notification};

pub struct SubscriptionRoot;

#[Subscription]
impl SubscriptionRoot {
    async fn all_messages<'a>(&self, ctx: &'a Context<'a>) -> Result<impl Stream<Item = AppEvent> + 'a, AppError> {
        let user_id = ctx
            .data::<UserId>()
            .map_err(|_| AppError::GetGraphqlUserIdError)?;
        let state = AppState::shared().await;

        let mut rv = ctx
            .data_unchecked::<Arc<broadcast::Sender<Notification>>>()
            .subscribe();

        Ok(async_stream::stream! {
            loop {
                let noti = rv.recv().await;
                match noti {
                    Ok(noti) => {
                        let message: Option<Message> = None;
                        let message = match noti.event.clone() {
                            AppEvent::NewMessage(new_message) => Some(new_message),
                            _ => None
                        };

                        if let Some(message) = message {
                            let chat = message.get_chat().await;

                            if let Ok(chat) = chat {
                                let members = chat.get_members().await;

                                if let Ok(members) = members {
                                    let member_ids: HashSet<i64> = members.iter().map(|u| u.id).collect();
                                    if member_ids.contains(&user_id) {
                                        yield noti.event;
                                    }
                                }
                            }
                        }
                    },
                    Err(e) => {
                        debug!("Error: {:?}", e);
                    }
                }
            }
        })
    }

    async fn message<'a>(&self, ctx: &'a Context<'a>, chat_id: i64) -> Result<impl Stream<Item = AppEvent> + 'a, AppError> {
        let user_id = ctx
            .data::<UserId>()
            .map_err(|_| AppError::GetGraphqlUserIdError)?;
        let state = AppState::shared().await;

        let mut rv = ctx
            .data_unchecked::<Arc<broadcast::Sender<Notification>>>()
            .subscribe();

        Ok(async_stream::stream! {
            loop {
                let noti = rv.recv().await;
                match noti {
                    Ok(noti) => {
                        let message: Option<Message> = None;
                        let message = match noti.event.clone() {
                            AppEvent::NewMessage(new_message) => Some(new_message),
                            _ => None
                        };

                        if let Some(message) = message {
                            if message.chat_id == chat_id {
                                yield noti.event;
                            }
                        }
                    },
                    Err(e) => {
                        debug!("Error: {:?}", e);
                    }
                }
            }
        })
    }

    async fn chat<'a>(&self, ctx: &'a Context<'a>) -> Result<impl Stream<Item = AppEvent> + 'a, AppError> {
        let user_id = ctx
            .data::<UserId>()
            .map_err(|_| AppError::GetGraphqlUserIdError)?;
        let state = AppState::shared().await;

        let mut rv = ctx
            .data_unchecked::<Arc<broadcast::Sender<Notification>>>()
            .subscribe();

        Ok(async_stream::stream! {
            loop {
                let noti = rv.recv().await;
                match noti {
                    Ok(noti) => {
                        let chat: Option<Chat> = None;
                        let chat = match noti.event.clone() {
                            AppEvent::CreatedChat(created_chat) => Some(created_chat.data),
                            AppEvent::ChatOwnerChanged(chat_owner_changed) => Some(chat_owner_changed.data),
                            AppEvent::ChatNameChanged(chat_name_changed) => Some(chat_name_changed.data),
                            AppEvent::ChatDeleted(chat_deleted) => Some(chat_deleted.data),
                            _ => None,
                        };

                        if let Some(_) = chat {
                            yield noti.event;
                        }
                    },
                    Err(e) => {
                        debug!("Error: {:?}", e);
                    }
                }
            }
        })
    }

    async fn interval2(&self, #[graphql(default = 1)] n: i32) -> impl Stream<Item = i32> {
        let mut value = 0;
        debug!("Init Stream value: {}", value);
        async_stream::stream! {
            loop {
                futures_timer::Delay::new(Duration::from_secs(1)).await;
                value += n;
                debug!("Stream value: {}", value);
                yield value;
            }
        }
    }
}
