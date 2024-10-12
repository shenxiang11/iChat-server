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
