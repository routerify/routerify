use hyper::{Body, Request};
use std::future::Future;
use std::pin::Pin;

type BoxedPreHandler = Box<dyn Fn(Request<Body>) -> BoxedTransformedRequest + Send + Sync + 'static>;
type BoxedTransformedRequest = Box<dyn Future<Output = crate::Result<Request<Body>>> + Send + Sync + 'static>;

pub struct PreMiddleware {
    handler: BoxedPreHandler,
}

impl PreMiddleware {
    pub fn new<H, R>(handler: H) -> Self
    where
        H: Fn(Request<Body>) -> R + Send + Sync + 'static,
        R: Future<Output = crate::Result<Request<Body>>> + Send + Sync + 'static,
    {
        let handler: BoxedPreHandler = Box::new(move |req: Request<Body>| Box::new(handler(req)));
        PreMiddleware { handler }
    }

    pub async fn process(&self, req: Request<Body>) -> crate::Result<Request<Body>> {
        Pin::from((self.handler)(req)).await
    }
}
