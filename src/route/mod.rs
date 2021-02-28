use crate::helpers;
use crate::regex_generator::generate_exact_match_regex;
use crate::types::{RequestMeta, RouteParams};
use crate::Error;
use hyper::{body::HttpBody, Method, Request, Response};
use regex::Regex;
use std::fmt::{self, Debug, Formatter};
use std::future::Future;
use std::pin::Pin;

type Handler<B, E> = Box<dyn Fn(Request<hyper::Body>) -> HandlerReturn<B, E> + Send + Sync + 'static>;
type HandlerReturn<B, E> = Box<dyn Future<Output = Result<Response<B>, E>> + Send + 'static>;

/// Represents a single route.
///
/// A route consists of a path, http method type(s) and a handler. It shouldn't be created directly, use [RouterBuilder](./struct.RouterBuilder.html) methods
/// to create a route.
///
/// This `Route<B, E>` type accepts two type parameters: `B` and `E`.
///
/// * The `B` represents the response body type which will be used by route handlers and the middlewares and this body type must implement
///   the [HttpBody](https://docs.rs/hyper/0.13.5/hyper/body/trait.HttpBody.html) trait. For an instance, `B` could be [hyper::Body](https://docs.rs/hyper/0.13.5/hyper/body/struct.Body.html)
///   type.
/// * The `E` represents any error type which will be used by route handlers and the middlewares. This error type must implement the [std::error::Error](https://doc.rust-lang.org/std/error/trait.Error.html).
///
/// # Examples
///
/// ```
/// use routerify::Router;
/// use hyper::{Response, Request, Body};
///
/// async fn home_handler(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
///     Ok(Response::new(Body::from("home")))
/// }
///
/// # fn run() -> Router<Body, hyper::Error> {
/// let router = Router::builder()
///     // Create a route on "/" path for `GET` method.
///     .get("/", home_handler)
///     .build()
///     .unwrap();
/// # router
/// # }
/// # run();
/// ```
pub struct Route<B, E> {
    pub(crate) path: String,
    pub(crate) regex: Regex,
    route_params: Vec<String>,
    // Make it an option so that when a router is used to scope in another router,
    // It can be extracted out by 'opt.take()' without taking the whole router's ownership.
    pub(crate) handler: Option<Handler<B, E>>,
    pub(crate) methods: Vec<Method>,
}

impl<
        B: HttpBody + Send + Sync + 'static,
        E: Into<Box<dyn std::error::Error + Send + Sync>> + 'static,
    > Route<B, E>
{
    pub(crate) fn new_with_boxed_handler<P: Into<String>>(
        path: P,
        methods: Vec<Method>,
        handler: Handler<B, E>,
    ) -> crate::Result<Route<B, E>> {
        let path = path.into();
        let (re, params) = generate_exact_match_regex(path.as_str())?;

        Ok(Route {
            path,
            regex: re,
            route_params: params,
            handler: Some(handler),
            methods,
        })
    }

    pub(crate) fn new<P, H, R>(path: P, methods: Vec<Method>, handler: H) -> crate::Result<Route<B, E>>
    where
        P: Into<String>,
        H: Fn(Request<hyper::Body>) -> R + Send + Sync + 'static,
        R: Future<Output = Result<Response<B>, E>> + Send + 'static,
    {
        let handler: Handler<B, E> = Box::new(move |req: Request<hyper::Body>| Box::new(handler(req)));
        Route::new_with_boxed_handler(path, methods, handler)
    }

    pub(crate) fn is_match_method(&self, method: &Method) -> bool {
        self.methods.contains(method)
    }

    pub(crate) async fn process(
        &self,
        target_path: &str,
        mut req: Request<hyper::Body>,
    ) -> crate::Result<Response<B>> {
        self.push_req_meta(target_path, &mut req);

        let handler = self
            .handler
            .as_ref()
            .expect("A router can not be used after mounting into another router");

        Pin::from(handler(req))
            .await
            .map_err(|e| Error::HandleRequest(e.into(), target_path.into()))
    }

    fn push_req_meta(&self, target_path: &str, req: &mut Request<hyper::Body>) {
        self.update_req_meta(req, self.generate_req_meta(target_path));
    }

    fn update_req_meta(&self, req: &mut Request<hyper::Body>, req_meta: RequestMeta) {
        helpers::update_req_meta_in_extensions(req.extensions_mut(), req_meta);
    }

    fn generate_req_meta(&self, target_path: &str) -> RequestMeta {
        let route_params_list = &self.route_params;
        let ln = route_params_list.len();

        let mut route_params = RouteParams::with_capacity(ln);

        if ln > 0 {
            if let Some(caps) = self.regex.captures(target_path) {
                let mut iter = caps.iter();
                for param in route_params_list {
                    if let Some(Some(g)) = iter.next() {
                        route_params.set(param.clone(), g.as_str());
                    }
                }
            }
        }

        RequestMeta::with_route_params(route_params)
    }
}

impl<B, E> Debug for Route<B, E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{ path: {:?}, regex: {:?}, route_params: {:?}, methods: {:?} }}",
            self.path, self.regex, self.route_params, self.methods
        )
    }
}
