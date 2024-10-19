mod chat;
mod user;
mod message;

use async_graphql::{MergedObject, Object};

pub(crate) use chat::*;
pub(crate) use user::*;
pub(crate) use message::*;

#[derive(MergedObject, Default)]
pub(crate) struct QueryRoot(DemoQuery, ChatQuery, MessageQuery);

#[derive(MergedObject, Default)]
pub(crate) struct MutationRoot(UserMutation, ChatMutation, MessageMutation);

#[derive(Default)]
struct DemoQuery;

#[Object]
impl DemoQuery {
    async fn hello(&self) -> String {
        "hello world".to_string()
    }
}
