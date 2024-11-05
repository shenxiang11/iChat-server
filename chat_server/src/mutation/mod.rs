use async_graphql::MergedObject;
use crate::mutation::chat::ChatMutation;
use crate::mutation::message::MessageMutation;
use crate::mutation::user::UserMutation;

mod chat;
mod message;
mod user;

#[derive(MergedObject, Default)]
pub(crate) struct MutationRoot(UserMutation, ChatMutation, MessageMutation);
