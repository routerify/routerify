use hyper::{body::HttpBody, Request, Response};
use std::future::Future;

pub use self::post::PostMiddleware;
pub use self::pre::PreMiddleware;

mod post;
mod pre;

#[derive(Debug)]
pub enum Middleware<B, E> {
    Pre(PreMiddleware<B, E>),
    Post(PostMiddleware<B, E>),
}

impl<B: HttpBody + Send + Sync + Unpin + 'static, E: std::error::Error + Send + Sync + Unpin + 'static>
    Middleware<B, E>
{
    pub fn pre_with_path<P, H, R>(path: P, handler: H) -> crate::Result<Middleware<B, E>>
    where
        P: Into<String>,
        H: FnMut(Request<B>) -> R + Send + Sync + 'static,
        R: Future<Output = Result<Request<B>, E>> + Send + 'static,
    {
        Ok(Middleware::Pre(PreMiddleware::new(path, handler)?))
    }

    pub fn post_with_path<P, H, R>(path: P, handler: H) -> crate::Result<Middleware<B, E>>
    where
        P: Into<String>,
        H: FnMut(Response<B>) -> R + Send + Sync + 'static,
        R: Future<Output = Result<Response<B>, E>> + Send + 'static,
    {
        Ok(Middleware::Post(PostMiddleware::new(path, handler)?))
    }

    pub fn pre<H, R>(handler: H) -> Middleware<B, E>
    where
        H: FnMut(Request<B>) -> R + Send + Sync + 'static,
        R: Future<Output = Result<Request<B>, E>> + Send + 'static,
    {
        Middleware::pre_with_path("/*", handler).unwrap()
    }

    pub fn post<H, R>(handler: H) -> Middleware<B, E>
    where
        H: FnMut(Response<B>) -> R + Send + Sync + 'static,
        R: Future<Output = Result<Response<B>, E>> + Send + 'static,
    {
        Middleware::post_with_path("/*", handler).unwrap()
    }
}
