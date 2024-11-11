use std::collections::HashSet;
use std::sync::Arc;
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
    async fn scan_login(&self, ctx: &Context<'_>, device_uuid: String) -> Result<impl Stream<Item = AppEvent>, AppError> {
        let state = ctx.data_unchecked::<AppState>();

        let mut rv = state.sender.subscribe();

        Ok(async_stream::stream! {
            loop {
                let noti = rv.recv().await;
                match noti {
                    Ok(noti) => {
                        match noti.event.clone() {
                            AppEvent::QRCodeConfirmed(payload) => {
                                if payload.device_uuid == device_uuid {
                                    yield noti.event;
                                    break;
                                }
                            },
                            AppEvent::QRCodeScanned(payload) => {
                                if payload.device_uuid == device_uuid {
                                    yield noti.event;
                                }
                            },
                            AppEvent::QRCodeCancel(payload) => {
                                if payload.device_uuid == device_uuid {
                                    yield noti.event;
                                    break;
                                }
                            },
                            _ => {}
                        };
                    },
                    Err(e) => {
                        debug!("Error: {:?}", e);
                    }
                }
            }
        })
    }

    async fn all_messages<'a>(&self, ctx: &'a Context<'a>) -> Result<impl Stream<Item = AppEvent> + 'a, AppError> {
        let state = ctx.data_unchecked::<AppState>();
        let user_id = ctx
            .data::<UserId>()
            .map_err(|_| AppError::GetGraphqlUserIdError)?;

        let mut rv = state.sender.subscribe();

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
                            let chat = state.chat_repo.get_chat_by_id(message.chat_id, message.user_id).await;

                            if let Ok(chat) = chat {
                                let members = state.chat_repo.get_members(chat.id).await;

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
        let state = ctx.data_unchecked::<AppState>();
        let user_id = ctx
            .data::<UserId>()
            .map_err(|_| AppError::GetGraphqlUserIdError)?;

        let mut rv = state.sender.subscribe();

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
        let state = ctx.data_unchecked::<AppState>();

        let mut rv = state.sender.subscribe();

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
}
