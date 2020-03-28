use hyper::{Body, Request, Response};
use std::future::Future;

pub use self::post::PostMiddleware;
pub use self::pre::PreMiddleware;

mod post;
mod pre;

pub enum Middleware {
    Pre(PreMiddleware),
    Post(PostMiddleware),
}

impl Middleware {
    pub fn pre<H, R>(handler: H) -> Middleware
    where
        H: Fn(Request<Body>) -> R + Send + Sync + 'static,
        R: Future<Output = crate::Result<Request<Body>>> + Send + Sync + 'static,
    {
        Middleware::Pre(PreMiddleware::new(handler))
    }

    pub fn post<H, R>(handler: H) -> Middleware
    where
        H: Fn(Response<Body>) -> R + Send + Sync + 'static,
        R: Future<Output = crate::Result<Response<Body>>> + Send + Sync + 'static,
    {
        Middleware::Post(PostMiddleware::new(handler))
    }
}
