use crate::regex_generator::generate_exact_match_regex;
use crate::types::RequestInfo;
use crate::Error;
use hyper::{body::HttpBody, Response};
use regex::Regex;
use std::fmt::{self, Debug, Formatter};
use std::future::Future;
use std::pin::Pin;

type HandlerWithoutInfo<B, E> = Box<dyn Fn(Response<B>) -> HandlerWithoutInfoReturn<B, E> + Send + Sync + 'static>;
type HandlerWithoutInfoReturn<B, E> = Box<dyn Future<Output = Result<Response<B>, E>> + Send + 'static>;

type HandlerWithInfo<B, E> =
    Box<dyn Fn(Response<B>, RequestInfo) -> HandlerWithInfoReturn<B, E> + Send + Sync + 'static>;
type HandlerWithInfoReturn<B, E> = Box<dyn Future<Output = Result<Response<B>, E>> + Send + 'static>;

/// The post middleware type. Refer to [Post Middleware](./index.html#post-middleware) for more info.
///
/// This `PostMiddleware<B, E>` type accepts two type parameters: `B` and `E`.
///
/// * The `B` represents the response body type which will be used by route handlers and the middlewares and this body type must implement
///   the [HttpBody](https://docs.rs/hyper/0.14.4/hyper/body/trait.HttpBody.html) trait. For an instance, `B` could be [hyper::Body](https://docs.rs/hyper/0.14.4/hyper/body/struct.Body.html)
///   type.
/// * The `E` represents any error type which will be used by route handlers and the middlewares. This error type must implement the [std::error::Error](https://doc.rust-lang.org/std/error/trait.Error.html).
pub struct PostMiddleware<B, E> {
    pub(crate) path: String,
    pub(crate) regex: Regex,
    // Make it an option so that when a router is used to scope in another router,
    // It can be extracted out by 'opt.take()' without taking the whole router's ownership.
    pub(crate) handler: Option<Handler<B, E>>,
    // Scope depth with regards to the top level router.
    pub(crate) scope_depth: u32,
}

pub(crate) enum Handler<B, E> {
    WithoutInfo(HandlerWithoutInfo<B, E>),
    WithInfo(HandlerWithInfo<B, E>),
}

impl<B: HttpBody + Send + Sync + 'static, E: Into<Box<dyn std::error::Error + Send + Sync>> + 'static>
    PostMiddleware<B, E>
{
    pub(crate) fn new_with_boxed_handler<P: Into<String>>(
        path: P,
        handler: Handler<B, E>,
        scope_depth: u32,
    ) -> crate::Result<PostMiddleware<B, E>> {
        let path = path.into();
        let (re, _) = generate_exact_match_regex(path.as_str()).map_err(|e| {
            Error::new(format!(
                "Could not create an exact match regex for the post middleware path: {}",
                e
            ))
        })?;

        Ok(PostMiddleware {
            path,
            regex: re,
            handler: Some(handler),
            scope_depth,
        })
    }

    /// Creates a post middleware with a handler at the specified path.
    ///
    /// # Examples
    ///
    /// ```
    /// use routerify::{Router, Middleware, PostMiddleware};
    /// use hyper::{Response, Body};
    /// use std::convert::Infallible;
    ///
    /// # fn run() -> Router<Body, Infallible> {
    /// let router = Router::builder()
    ///      .middleware(Middleware::Post(PostMiddleware::new("/abc", |res| async move { /* Do some operations */ Ok(res) }).unwrap()))
    ///      .build()
    ///      .unwrap();
    /// # router
    /// # }
    /// # run();
    /// ```
    pub fn new<P, H, R>(path: P, handler: H) -> crate::Result<PostMiddleware<B, E>>
    where
        P: Into<String>,
        H: Fn(Response<B>) -> R + Send + Sync + 'static,
        R: Future<Output = Result<Response<B>, E>> + Send + 'static,
    {
        let handler: HandlerWithoutInfo<B, E> = Box::new(move |res: Response<B>| Box::new(handler(res)));
        PostMiddleware::new_with_boxed_handler(path, Handler::WithoutInfo(handler), 1)
    }

    /// Creates a post middleware which can access [request info](./struct.RequestInfo.html) e.g. headers, method, uri etc. It should be used when the post middleware trandforms the response based on
    /// the request information.
    ///
    /// # Examples
    ///
    /// ```
    /// use routerify::{Router, Middleware, PostMiddleware, RequestInfo};
    /// use hyper::{Response, Body};
    /// use std::convert::Infallible;
    ///
    /// async fn post_middleware_with_info_handler(res: Response<Body>, req_info: RequestInfo) -> Result<Response<Body>, Infallible> {
    ///     let headers = req_info.headers();
    ///     
    ///     // Do some response transformation based on the request headers, method etc.
    ///     
    ///     Ok(res)
    /// }
    ///
    /// # fn run() -> Router<Body, Infallible> {
    /// let router = Router::builder()
    ///      .middleware(Middleware::Post(PostMiddleware::new_with_info("/abc", post_middleware_with_info_handler).unwrap()))
    ///      .build()
    ///      .unwrap();
    /// # router
    /// # }
    /// # run();
    /// ```
    pub fn new_with_info<P, H, R>(path: P, handler: H) -> crate::Result<PostMiddleware<B, E>>
    where
        P: Into<String>,
        H: Fn(Response<B>, RequestInfo) -> R + Send + Sync + 'static,
        R: Future<Output = Result<Response<B>, E>> + Send + 'static,
    {
        let handler: HandlerWithInfo<B, E> =
            Box::new(move |res: Response<B>, req_info: RequestInfo| Box::new(handler(res, req_info)));
        PostMiddleware::new_with_boxed_handler(path, Handler::WithInfo(handler), 1)
    }

    pub(crate) fn should_require_req_meta(&self) -> bool {
        if let Some(ref handler) = self.handler {
            match handler {
                Handler::WithInfo(_) => true,
                Handler::WithoutInfo(_) => false,
            }
        } else {
            false
        }
    }

    pub(crate) async fn process(&self, res: Response<B>, req_info: Option<RequestInfo>) -> crate::Result<Response<B>> {
        let handler = self
            .handler
            .as_ref()
            .expect("A router can not be used after mounting into another router");

        match handler {
            Handler::WithoutInfo(ref handler) => Pin::from(handler(res)).await.map_err(Into::into),
            Handler::WithInfo(ref handler) => Pin::from(handler(res, req_info.expect("No RequestInfo is provided")))
                .await
                .map_err(Into::into),
        }
    }
}

impl<B, E> Debug for PostMiddleware<B, E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{{ path: {:?}, regex: {:?} }}", self.path, self.regex)
    }
}
