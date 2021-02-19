use crate::data_map::ScopedDataMap;
use crate::middleware::{PostMiddleware, PreMiddleware};
use crate::route::Route;
use crate::types::RequestInfo;
use crate::Error;
use hyper::{body::HttpBody, Request, Response};
use regex::RegexSet;
use std::fmt::{self, Debug, Formatter};
use std::future::Future;
use std::pin::Pin;

pub use self::builder::RouterBuilder;

mod builder;

pub(crate) type ErrHandlerWithoutInfo<B> =
    Box<dyn FnMut(crate::Error) -> ErrHandlerWithoutInfoReturn<B> + Send + Sync + 'static>;
pub(crate) type ErrHandlerWithoutInfoReturn<B> = Box<dyn Future<Output = Response<B>> + Send + 'static>;

pub(crate) type ErrHandlerWithInfo<B> =
    Box<dyn FnMut(crate::Error, RequestInfo) -> ErrHandlerWithInfoReturn<B> + Send + Sync + 'static>;
pub(crate) type ErrHandlerWithInfoReturn<B> = Box<dyn Future<Output = Response<B>> + Send + 'static>;

/// Represents a modular, lightweight and mountable router type.
///
/// A router consists of some routes, some pre-middlewares and some post-middlewares.
///
/// This `Router<B, E>` type accepts two type parameters: `B` and `E`.
///
/// * The `B` represents the response body type which will be used by route handlers and the middlewares and this body type must implement
///   the [HttpBody](https://docs.rs/hyper/0.13.5/hyper/body/trait.HttpBody.html) trait. For an instance, `B` could be [hyper::Body](https://docs.rs/hyper/0.13.5/hyper/body/struct.Body.html)
///   type.
/// * The `E` represents any error type which will be used by route handlers and the middlewares. This error type must implement the [std::error::Error](https://doc.rust-lang.org/std/error/trait.Error.html).
///
/// A `Router` can be created using the `Router::builder()` method.
///
/// # Examples
///
/// ```
/// use routerify::Router;
/// use hyper::{Response, Request, Body};
///
/// // A handler for "/about" page.
/// // We will use hyper::Body as response body type and hyper::Error as error type.
/// async fn about_handler(_: Request<Body>) -> Result<Response<Body>, hyper::Error> {
///     Ok(Response::new(Body::from("About page")))
/// }
///
/// # fn run() -> Router<Body, hyper::Error> {
/// // Create a router with hyper::Body as response body type and hyper::Error as error type.
/// let router: Router<Body, hyper::Error> = Router::builder()
///     .get("/about", about_handler)
///     .build()
///     .unwrap();
/// # router
/// # }
/// # run();
/// ```
pub struct Router<B, E> {
    pub(crate) pre_middlewares: Vec<PreMiddleware<E>>,
    pub(crate) routes: Vec<Route<B, E>>,
    pub(crate) post_middlewares: Vec<PostMiddleware<B, E>>,
    pub(crate) scoped_data_maps: Vec<ScopedDataMap>,

    // This handler should be added only on root Router.
    // Any error handler attached to scoped router will be ignored.
    pub(crate) err_handler: Option<ErrHandler<B>>,

    // We'll initialize it from the RouterService via Router::init_regex_set() method.
    regex_set: Option<RegexSet>,

    // We'll initialize it from the RouterService via Router::init_req_info_gen() method.
    pub(crate) should_gen_req_info: Option<bool>,
}

pub(crate) enum ErrHandler<B> {
    WithoutInfo(ErrHandlerWithoutInfo<B>),
    WithInfo(ErrHandlerWithInfo<B>),
}

impl<B: HttpBody + Send + Sync + Unpin + 'static> ErrHandler<B> {
    pub(crate) async fn execute(&mut self, err: crate::Error, req_info: Option<RequestInfo>) -> Response<B> {
        match self {
            ErrHandler::WithoutInfo(ref mut err_handler) => Pin::from(err_handler(err)).await,
            ErrHandler::WithInfo(ref mut err_handler) => {
                Pin::from(err_handler(err, req_info.expect("No RequestInfo is provided"))).await
            }
        }
    }
}

impl<
        B: HttpBody + Send + Sync + Unpin + 'static,
        E: Into<Box<dyn std::error::Error + Send + Sync>> + Unpin + 'static,
    > Router<B, E>
{
    pub(crate) fn new(
        pre_middlewares: Vec<PreMiddleware<E>>,
        routes: Vec<Route<B, E>>,
        post_middlewares: Vec<PostMiddleware<B, E>>,
        scoped_data_maps: Vec<ScopedDataMap>,
        err_handler: Option<ErrHandler<B>>,
    ) -> Self {
        Router {
            pre_middlewares,
            routes,
            post_middlewares,
            scoped_data_maps,
            err_handler,
            regex_set: None,
            should_gen_req_info: None,
        }
    }

    pub(crate) fn init_regex_set(&mut self) -> crate::Result<()> {
        let regex_iter = self
            .pre_middlewares
            .iter()
            .map(|m| m.regex.as_str())
            .chain(self.routes.iter().map(|r| r.regex.as_str()))
            .chain(self.post_middlewares.iter().map(|m| m.regex.as_str()))
            .chain(self.scoped_data_maps.iter().map(|d| d.regex.as_str()));

        self.regex_set = Some(RegexSet::new(regex_iter).map_err(Error::CreateRouterRegexSet)?);

        Ok(())
    }

    pub(crate) fn init_req_info_gen(&mut self) {
        if let Some(ErrHandler::WithInfo (_) ) = self.err_handler {
            self.should_gen_req_info = Some(true);
            return;
        }

        for post_middleware in self.post_middlewares.iter() {
            if post_middleware.should_require_req_meta() {
                self.should_gen_req_info = Some(true);
                return;
            }
        }

        self.should_gen_req_info = Some(false);
    }

    /// Return a [RouterBuilder](./struct.RouterBuilder.html) instance to build a `Router`.
    pub fn builder() -> RouterBuilder<B, E> {
        builder::RouterBuilder::new()
    }

    pub(crate) async fn process(
        &mut self,
        target_path: &str,
        mut req: Request<hyper::Body>,
        mut req_info: Option<RequestInfo>,
    ) -> crate::Result<Response<B>> {
        let (
            matched_pre_middleware_idxs,
            matched_route_idxs,
            matched_post_middleware_idxs,
            matched_scoped_data_map_idxs,
        ) = self.match_regex_set(target_path);

        let shared_data_maps = matched_scoped_data_map_idxs
            .into_iter()
            .map(|idx| self.scoped_data_maps[idx].clone_data_map())
            .collect::<Vec<_>>();

        if let Some(ref mut req_info) = req_info {
            if !shared_data_maps.is_empty() {
                req_info.shared_data_maps.replace(shared_data_maps.clone());
            }
        }

        let ext = req.extensions_mut();
        ext.insert(shared_data_maps);

        let mut transformed_req = req;
        for idx in matched_pre_middleware_idxs {
            let pre_middleware = &mut self.pre_middlewares[idx];

            transformed_req = pre_middleware.process(transformed_req).await?;
        }

        let mut resp = None;
        for idx in matched_route_idxs {
            let route = &mut self.routes[idx];

            if route.is_match_method(transformed_req.method()) {
                let route_resp_res = route.process(target_path, transformed_req).await;

                let route_resp = match route_resp_res {
                    Ok(route_resp) => route_resp,
                    Err(err) => {
                        if let Some(ref mut err_handler) = self.err_handler {
                            err_handler.execute(err, req_info.clone()).await
                        } else {
                            return Err(err);
                        }
                    }
                };

                resp = Some(route_resp);
                break;
            }
        }

        if resp.is_none() {
            return Err(Error::HandleNonExistentRoute);
        }

        let mut transformed_res = resp.unwrap();
        for idx in matched_post_middleware_idxs {
            let post_middleware = &mut self.post_middlewares[idx];
            transformed_res = post_middleware.process(transformed_res, req_info.clone()).await?;
        }

        Ok(transformed_res)
    }

    fn match_regex_set(&self, target_path: &str) -> (Vec<usize>, Vec<usize>, Vec<usize>, Vec<usize>) {
        let matches = self
            .regex_set
            .as_ref()
            .expect("The 'regex_set' field in Router is not initialized")
            .matches(target_path)
            .into_iter();

        let pre_middlewares_len = self.pre_middlewares.len();
        let routes_len = self.routes.len();
        let post_middlewares_len = self.post_middlewares.len();
        let scoped_data_maps_len = self.scoped_data_maps.len();

        let mut matched_pre_middleware_idxs = Vec::new();
        let mut matched_route_idxs = Vec::new();
        let mut matched_post_middleware_idxs = Vec::new();
        let mut matched_scoped_data_map_idxs = Vec::new();

        for idx in matches {
            if idx < pre_middlewares_len {
                matched_pre_middleware_idxs.push(idx);
            } else if idx >= pre_middlewares_len && idx < (pre_middlewares_len + routes_len) {
                matched_route_idxs.push(idx - pre_middlewares_len);
            } else if idx >= (pre_middlewares_len + routes_len)
                && idx < (pre_middlewares_len + routes_len + post_middlewares_len)
            {
                matched_post_middleware_idxs.push(idx - pre_middlewares_len - routes_len);
            } else if idx >= (pre_middlewares_len + routes_len + post_middlewares_len)
                && idx < (pre_middlewares_len + routes_len + post_middlewares_len + scoped_data_maps_len)
            {
                matched_scoped_data_map_idxs.push(idx - pre_middlewares_len - routes_len - post_middlewares_len);
            }
        }

        (
            matched_pre_middleware_idxs,
            matched_route_idxs,
            matched_post_middleware_idxs,
            matched_scoped_data_map_idxs,
        )
    }
}

impl<B, E> Debug for Router<B, E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{ Pre-Middlewares: {:?}, Routes: {:?}, Post-Middlewares: {:?}, ScopedDataMaps: {:?}, ErrHandler: {:?}, ShouldGenReqInfo: {:?} }}",
            self.pre_middlewares,
            self.routes,
            self.post_middlewares,
            self.scoped_data_maps,
            self.err_handler.is_some(),
            self.should_gen_req_info
        )
    }
}
