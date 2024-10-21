mod chat;
mod user;
mod message;

use async_graphql::{MergedObject};

pub(crate) use chat::*;
pub(crate) use user::*;
pub(crate) use message::*;

#[derive(MergedObject, Default)]
pub(crate) struct QueryRoot(UserQuery, ChatQuery, MessageQuery);

#[derive(MergedObject, Default)]
pub(crate) struct MutationRoot(UserMutation, ChatMutation, MessageMutation);
