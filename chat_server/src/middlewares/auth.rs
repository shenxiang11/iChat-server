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


#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use axum::middleware::from_fn_with_state;
    use axum::Router;
    use axum::routing::get;
    use tower::ServiceExt;
    use crate::app_state::AppState;
    use crate::config::AppConfig;
    use super::verify_token;

    #[tokio::test]
    async fn  verify_token_should_work() -> anyhow::Result<()> {
        let config = AppConfig::load()?;
        let state = AppState::try_new(config).await?;


        let app = Router::new()
            .route("/", get(|| async { "Ok" }))
            .layer(from_fn_with_state(state.clone(), verify_token));

        // good token
        let token = state.ek.sign(1)?;
        let req = Request::builder().header("Authorization", format!("Bearer {}", token)).body(Body::empty())?;
        let res = app.clone().oneshot(req).await?;

        assert_eq!(res.status(), StatusCode::OK);

        // bad token
        let token = token + "bad";
        let req = Request::builder().header("Authorization", format!("Bearer {}", token)).body(Body::empty())?;
        let res = app.clone().oneshot(req).await?;
        assert_eq!(res.status(), StatusCode::FORBIDDEN);

        // no token
        let req = Request::builder().body(Body::empty())?;
        let res = app.clone().oneshot(req).await?;
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

        Ok(())
    }
}
