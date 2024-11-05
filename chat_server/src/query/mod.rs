mod chat;
mod user;
mod message;
mod file;

use async_graphql::{MergedObject};

pub(crate) use chat::*;
pub(crate) use user::*;
pub(crate) use message::*;
pub(crate) use file::*;

#[derive(MergedObject, Default)]
pub(crate) struct QueryRoot(UserQuery, ChatQuery, MessageQuery, FileQuery);

#[derive(MergedObject, Default)]
pub(crate) struct MutationRoot(UserMutation, ChatMutation, MessageMutation);
