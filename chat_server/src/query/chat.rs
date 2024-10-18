use async_graphql::{Context, Object};

use crate::app_state::AppState;
use crate::models::{Chat, UserId};

#[derive(Default)]
pub(crate) struct ChatQuery;

#[Object]
impl ChatQuery {
    async fn get_chat(
        &self,
        ctx: &Context<'_>,
        id: i64
    ) -> anyhow::Result<Option<Chat>> {
        let state = AppState::shared().await;
        let user_id = ctx.data::<UserId>().unwrap();

        let res = state.chat_repo.get_chat_by_id(id, user_id).await;

        match res {
            Ok(chat) => Ok(Some(chat)),
            Err(_) => Ok(None)
        }
    }

    // async fn get_chats(&self, ctx: &Context<'_>) -> Result<Vec<Chat>> {
    //     let state = ctx.data::<AppState>().unwrap();
    //     let res = state.chat_repo.get_all_chats(1).await;
    //
    //     match res {
    //         Ok(chats) => Ok(chats),
    //         Err(_) => Ok(vec![])
    //     }
    // }
}
