use crate::prelude::*;
use crate::regex_generator::generate_exact_match_regex;
use hyper::{body::HttpBody, Response};
use regex::Regex;
use std::fmt::{self, Debug, Formatter};
use std::future::Future;
use std::pin::Pin;

type Handler<B, E> = Box<dyn FnMut(Response<B>) -> HandlerReturn<B, E> + Send + Sync + 'static>;
type HandlerReturn<B, E> = Box<dyn Future<Output = Result<Response<B>, E>> + Send + 'static>;

pub struct PostMiddleware<B, E> {
    pub(crate) path: String,
    regex: Regex,
    // Make it an option so that when a router is used to scope in another router,
    // It can be extracted out by 'opt.take()' without taking the whole router's ownership.
    pub(crate) handler: Option<Handler<B, E>>,
}

impl<B: HttpBody + Send + Sync + Unpin + 'static, E: std::error::Error + Send + Sync + Unpin + 'static>
    PostMiddleware<B, E>
{
    pub(crate) fn new_with_boxed_handler<P: Into<String>>(
        path: P,
        handler: Handler<B, E>,
    ) -> crate::Result<PostMiddleware<B, E>> {
        let path = path.into();
        let (re, _) = generate_exact_match_regex(path.as_str())
            .context("Could not create an exact match regex for the post middleware path")?;

        Ok(PostMiddleware {
            path,
            regex: re,
            handler: Some(handler),
        })
    }

    pub fn new<P, H, R>(path: P, mut handler: H) -> crate::Result<PostMiddleware<B, E>>
    where
        P: Into<String>,
        H: FnMut(Response<B>) -> R + Send + Sync + 'static,
        R: Future<Output = Result<Response<B>, E>> + Send + 'static,
    {
        let handler: Handler<B, E> = Box::new(move |req: Response<B>| Box::new(handler(req)));
        PostMiddleware::new_with_boxed_handler(path, handler)
    }

    pub(crate) fn is_match(&self, target_path: &str) -> bool {
        self.regex.is_match(target_path)
    }

    pub(crate) async fn process(&mut self, res: Response<B>) -> crate::Result<Response<B>> {
        let handler = self
            .handler
            .as_mut()
            .expect("A router can not be used after mounting into another router");

        Pin::from(handler(res)).await.wrap()
    }
}

impl<B, E> Debug for PostMiddleware<B, E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{{ path: {:?}, regex: {:?} }}", self.path, self.regex)
    }
}
