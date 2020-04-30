use crate::helpers;
use crate::prelude::*;
use crate::regex_generator::generate_exact_match_regex;
use crate::types::{PathParams, RequestMeta};
use hyper::{body::HttpBody, Method, Request, Response};
use regex::Regex;
use std::fmt::{self, Debug, Formatter};
use std::future::Future;
use std::pin::Pin;

type Handler<B, E> = Box<dyn FnMut(Request<B>) -> HandlerReturn<B, E> + Send + Sync + 'static>;
type HandlerReturn<B, E> = Box<dyn Future<Output = Result<Response<B>, E>> + Send + 'static>;

pub struct Route<B, E> {
    pub(crate) path: String,
    regex: Regex,
    path_params: Vec<String>,
    // Make it an option so that when a router is used to scope in another router,
    // It can be extracted out by 'opt.take()' without taking the whole router's ownership.
    pub(crate) handler: Option<Handler<B, E>>,
    pub(crate) methods: Vec<Method>,
}

impl<B: HttpBody + Send + Sync + Unpin + 'static, E: std::error::Error + Send + Sync + Unpin + 'static> Route<B, E> {
    pub(crate) fn new_with_boxed_handler<P: Into<String>>(
        path: P,
        methods: Vec<Method>,
        handler: Handler<B, E>,
    ) -> crate::Result<Route<B, E>> {
        let path = path.into();
        let (re, params) = generate_exact_match_regex(path.as_str())
            .context("Could not create an exact match regex for the route path")?;

        Ok(Route {
            path,
            regex: re,
            path_params: params,
            handler: Some(handler),
            methods,
        })
    }

    pub(crate) fn new<P, H, R>(path: P, methods: Vec<Method>, mut handler: H) -> crate::Result<Route<B, E>>
    where
        P: Into<String>,
        H: FnMut(Request<B>) -> R + Send + Sync + 'static,
        R: Future<Output = Result<Response<B>, E>> + Send + 'static,
    {
        let handler: Handler<B, E> = Box::new(move |req: Request<B>| Box::new(handler(req)));
        Route::new_with_boxed_handler(path, methods, handler)
    }

    pub(crate) fn is_match(&self, target_path: &str, method: &Method) -> bool {
        self.regex.is_match(target_path) && self.methods.contains(method)
    }

    pub(crate) async fn process(&mut self, target_path: &str, mut req: Request<B>) -> crate::Result<Response<B>> {
        self.push_req_meta(target_path, &mut req);

        let handler = self
            .handler
            .as_mut()
            .expect("A router can not be used after mounting into another router");

        Pin::from(handler(req)).await.wrap()
    }

    fn push_req_meta(&self, target_path: &str, req: &mut Request<B>) {
        self.update_req_meta(req, self.generate_req_meta(target_path));
    }

    fn update_req_meta(&self, req: &mut Request<B>, req_meta: RequestMeta) {
        helpers::update_req_meta_in_extensions(req.extensions_mut(), req_meta);
    }

    fn generate_req_meta(&self, target_path: &str) -> RequestMeta {
        let path_params_list = &self.path_params;
        let ln = path_params_list.len();

        let mut path_params = PathParams::with_capacity(ln);

        if ln > 0 {
            if let Some(caps) = self.regex.captures(target_path) {
                for idx in 0..ln {
                    if let Some(g) = caps.get(idx + 1) {
                        path_params.set(path_params_list[idx].clone(), String::from(g.as_str()));
                    }
                }
            }
        }

        RequestMeta::with_path_params(path_params)
    }
}

impl<B, E> Debug for Route<B, E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{ path: {:?}, regex: {:?}, path_params: {:?}, methods: {:?} }}",
            self.path, self.regex, self.path_params, self.methods
        )
    }
}
