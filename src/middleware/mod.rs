use hyper::{body::HttpBody, Request, Response};
use std::future::Future;

pub use self::post::PostMiddleware;
pub use self::pre::PreMiddleware;

mod post;
mod pre;

/// The enum type for all the middleware types. Please refer to the [Middleware](./index.html#middleware) to know more about middlewares.
#[derive(Debug)]
pub enum Middleware<B, E> {
    /// Variant for the pre middleware. Refer to [Pre Middleware](./index.html#pre-middleware) for more info.
    Pre(PreMiddleware<B, E>),

    /// Variant for the post middleware. Refer to [Post Middleware](./index.html#post-middleware) for more info.
    Post(PostMiddleware<B, E>),
}

impl<B: HttpBody + Send + Sync + Unpin + 'static, E: std::error::Error + Send + Sync + Unpin + 'static>
    Middleware<B, E>
{
    /// Creates a pre middleware with a handler at the `/*` path.
    ///
    /// # Examples
    ///
    /// ```
    /// use routerify::{Router, Middleware};
    /// use hyper::{Request, Body};
    /// use std::convert::Infallible;
    ///
    /// # fn run() -> Router<Body, Infallible> {
    /// let router = Router::builder()
    ///      .middleware(Middleware::pre(|req| async move { /* Do some operations */ Ok(req) }))
    ///      .build()
    ///      .unwrap();
    /// # router
    /// # }
    /// # run();
    /// ```
    pub fn pre<H, R>(handler: H) -> Middleware<B, E>
    where
        H: FnMut(Request<B>) -> R + Send + Sync + 'static,
        R: Future<Output = Result<Request<B>, E>> + Send + 'static,
    {
        Middleware::pre_with_path("/*", handler).unwrap()
    }

    /// Creates a post middleware with a handler at the `/*` path.
    ///
    /// # Examples
    ///
    /// ```
    /// use routerify::{Router, Middleware};
    /// use hyper::{Response, Body};
    /// use std::convert::Infallible;
    ///
    /// # fn run() -> Router<Body, Infallible> {
    /// let router = Router::builder()
    ///      .middleware(Middleware::post(|res| async move { /* Do some operations */ Ok(res) }))
    ///      .build()
    ///      .unwrap();
    /// # router
    /// # }
    /// # run();
    /// ```
    pub fn post<H, R>(handler: H) -> Middleware<B, E>
    where
        H: FnMut(Response<B>) -> R + Send + Sync + 'static,
        R: Future<Output = Result<Response<B>, E>> + Send + 'static,
    {
        Middleware::post_with_path("/*", handler).unwrap()
    }

    /// Create a pre middleware with a handler at the specified path.
    ///
    /// # Examples
    ///
    /// ```
    /// use routerify::{Router, Middleware};
    /// use hyper::{Request, Body};
    /// use std::convert::Infallible;
    ///
    /// # fn run() -> Router<Body, Infallible> {
    /// let router = Router::builder()
    ///      .middleware(Middleware::pre_with_path("/my-path", |req| async move { /* Do some operations */ Ok(req) }).unwrap())
    ///      .build()
    ///      .unwrap();
    /// # router
    /// # }
    /// # run();
    /// ```
    pub fn pre_with_path<P, H, R>(path: P, handler: H) -> crate::Result<Middleware<B, E>>
    where
        P: Into<String>,
        H: FnMut(Request<B>) -> R + Send + Sync + 'static,
        R: Future<Output = Result<Request<B>, E>> + Send + 'static,
    {
        Ok(Middleware::Pre(PreMiddleware::new(path, handler)?))
    }

    /// Creates a post middleware with a handler at the specified path.
    ///
    /// # Examples
    ///
    /// ```
    /// use routerify::{Router, Middleware};
    /// use hyper::{Response, Body};
    /// use std::convert::Infallible;
    ///
    /// # fn run() -> Router<Body, Infallible> {
    /// let router = Router::builder()
    ///      .middleware(Middleware::post_with_path("/my-path", |res| async move { /* Do some operations */ Ok(res) }).unwrap())
    ///      .build()
    ///      .unwrap();
    /// # router
    /// # }
    /// # run();
    /// ```
    pub fn post_with_path<P, H, R>(path: P, handler: H) -> crate::Result<Middleware<B, E>>
    where
        P: Into<String>,
        H: FnMut(Response<B>) -> R + Send + Sync + 'static,
        R: Future<Output = Result<Response<B>, E>> + Send + 'static,
    {
        Ok(Middleware::Post(PostMiddleware::new(path, handler)?))
    }
}
