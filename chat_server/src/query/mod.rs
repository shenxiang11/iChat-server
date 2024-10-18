mod chat;
mod user;

use async_graphql::{MergedObject, Object};

pub(crate) use chat::*;
pub(crate) use user::*;

#[derive(MergedObject, Default)]
pub(crate) struct QueryRoot(DemoQuery, ChatQuery);

#[derive(MergedObject, Default)]
pub(crate) struct MutationRoot(UserMutation);

#[derive(Default)]
struct DemoQuery;

#[Object]
impl DemoQuery {
    async fn hello(&self) -> String {
        "hello world".to_string()
    }
}
