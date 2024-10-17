use axum::{Extension, Json, Router};
use axum::routing::{get, post};
use anyhow::Result;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};

use crate::app_state::AppState;
use crate::error::AppError;

pub(crate) fn register_routes() -> Router<AppState> {
    Router::new()
        .route("/", post(create_chat))
        .route("/", get(get_all_chats))
        .route("/:id", get(get_chat_info_by_id))
}

pub(crate) async fn create_chat(
    Extension(user_id): Extension<i64>,
    State(state): State<AppState>,
    Json(input): Json<CreateChat>,
) -> Result<impl IntoResponse, AppError> {
    let chat_id = state.chat_repo.create(user_id, input.member_ids).await?;

    Ok(Json(chat_id))
}

pub(crate) async fn get_all_chats(
    Extension(user_id): Extension<i64>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let chats = state.chat_repo.get_all_chats(user_id).await?;

    Ok(Json(chats))
}

pub(crate) async fn get_chat_info_by_id(
    Extension(user_id): Extension<i64>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let chats = state.chat_repo.get_chat_info_by_id(id, user_id).await?;

    Ok(Json(chats))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CreateChat {
    member_ids: Vec<i64>,
}
