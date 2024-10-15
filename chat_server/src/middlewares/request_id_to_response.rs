use axum::extract::Request;
use axum::http::HeaderName;
use axum::response::Response;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::{Layer, Service};

#[derive(Clone)]
pub struct RequestIdToResponseLayer {
    header_name: HeaderName,
}

impl RequestIdToResponseLayer {
    pub fn new(header_name: HeaderName) -> Self {
        Self { header_name }
    }
}

impl<S> Layer<S> for RequestIdToResponseLayer {
    type Service = RequestIdToResponseMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RequestIdToResponseMiddleware { inner, header_name: self.header_name.clone() }
    }
}

#[derive(Clone)]
pub struct RequestIdToResponseMiddleware<S> {
    inner: S,
    header_name: HeaderName,
}

impl<S> Service<Request> for RequestIdToResponseMiddleware<S>
where
    S: Service<Request, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        let id = match request.headers().get(self.header_name.clone()) {
            Some(id) => Some(id.clone()),
            None => None,
        };

        let future = self.inner.call(request);
        Box::pin(async move {
            let mut res: Response = future.await?;

            if let Some(id) = id {
                res.headers_mut().insert("ichat-request-id", id.clone());
            }

            Ok(res)
        })
    }
}


#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{HeaderName, HeaderValue, Request};
    use axum::Router;
    use axum::routing::get;
    use tower::{ServiceBuilder, ServiceExt};
    use tower_http::request_id;
    use tower_http::request_id::{MakeRequestId, PropagateRequestIdLayer, RequestId};
    use crate::middlewares::RequestIdToResponseLayer;

    #[derive(Debug, Clone)]
    struct TestIdGenerator;

    impl MakeRequestId for TestIdGenerator {
        fn make_request_id<B>(&mut self, request: &axum::http::Request<B>) -> Option<RequestId> {
            let request_id = "123";
            HeaderValue::from_str(&request_id).ok().map(RequestId::from)
        }
    }

    #[tokio::test]
    async fn request_id_is_added_to_response_headers() {
        let header_name = HeaderName::from_static("ichat-request-id");
        let layer = RequestIdToResponseLayer::new(header_name.clone());

        let app = Router::new()
            .route("/", get(|| async { "Ok" }))
            .layer(layer)
            .layer(request_id::SetRequestIdLayer::new(header_name.clone(), TestIdGenerator))
            .layer(PropagateRequestIdLayer::new(header_name.clone()));

        let req = Request::builder().body(Body::empty()).unwrap();
        let res = ServiceBuilder::new().service(app).oneshot(req).await.unwrap();

        assert_eq!(res.headers().get("ichat-request-id").unwrap(), "123");
    }

    #[tokio::test]
    async fn request_id_is_not_added_to_response_headers() {
        let header_name = HeaderName::from_static("ichat-request-id");
        let layer = RequestIdToResponseLayer::new(header_name.clone());

        let app = Router::new()
            .route("/", get(|| async { "Ok" }))
            .layer(layer);

        let req = Request::builder().body(Body::empty()).unwrap();
        let res = ServiceBuilder::new().service(app).oneshot(req).await.unwrap();

        assert_eq!(res.headers().get("ichat-request-id"), None);
    }
}
