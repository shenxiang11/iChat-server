use axum::extract::{FromRequestParts, Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum_extra::headers::Authorization;
use axum_extra::headers::authorization::Bearer;
use axum_extra::TypedHeader;
use tracing::warn;
use crate::app_state::AppState;

pub async fn verify_token(State(state): State<AppState>, req: Request, next: Next) -> Response {
    let (mut parts, body) = req.into_parts();

    let token = match TypedHeader::<Authorization<Bearer>>::from_request_parts(&mut parts, &state).await {
        Ok(TypedHeader(Authorization(bearer))) => bearer.token().to_string(),
        Err(e) => {
            let msg = format!("Token extract failed: {}", e);
            return (StatusCode::UNAUTHORIZED, msg).into_response();
        }
    };

    let req = match state.dk.verify(&token) {
        Ok(user_id) => {
            let mut req = Request::from_parts(parts, body);
            req.extensions_mut().insert(user_id);
            req
        },
        Err(e) => {
            let msg = format!("Token verify failed: {}", e);
            warn!(msg);
            return (StatusCode::FORBIDDEN, msg).into_response();
        }
    };

    next.run(req).await
}
