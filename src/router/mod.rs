use crate::constants;
use crate::data_map::ScopedDataMap;
use crate::middleware::{PostMiddleware, PreMiddleware};
use crate::route::Route;
use crate::types::RequestInfo;
use crate::Error;
use crate::RouteError;
use hyper::{
    body::HttpBody,
    header::{self, HeaderValue},
    Method, Request, Response, StatusCode,
};
use regex::RegexSet;
use std::any::Any;
use std::fmt::{self, Debug, Formatter};
use std::future::Future;
use std::pin::Pin;

pub use self::builder::RouterBuilder;

mod builder;

pub(crate) type ErrHandlerWithoutInfo<B> =
    Box<dyn Fn(RouteError) -> ErrHandlerWithoutInfoReturn<B> + Send + Sync + 'static>;
pub(crate) type ErrHandlerWithoutInfoReturn<B> = Box<dyn Future<Output = Response<B>> + Send + 'static>;

pub(crate) type ErrHandlerWithInfo<B> =
    Box<dyn Fn(RouteError, RequestInfo) -> ErrHandlerWithInfoReturn<B> + Send + Sync + 'static>;
pub(crate) type ErrHandlerWithInfoReturn<B> = Box<dyn Future<Output = Response<B>> + Send + 'static>;

/// Represents a modular, lightweight and mountable router type.
///
/// A router consists of some routes, some pre-middlewares and some post-middlewares.
///
/// This `Router<B, E>` type accepts two type parameters: `B` and `E`.
///
/// * The `B` represents the response body type which will be used by route handlers and the middlewares and this body type must implement
///   the [HttpBody](https://docs.rs/hyper/0.14.4/hyper/body/trait.HttpBody.html) trait. For an instance, `B` could be [hyper::Body](https://docs.rs/hyper/0.14.4/hyper/body/struct.Body.html)
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

impl<B: HttpBody + Send + Sync + 'static> ErrHandler<B> {
    pub(crate) async fn execute(&self, err: RouteError, req_info: Option<RequestInfo>) -> Response<B> {
        match self {
            ErrHandler::WithoutInfo(ref err_handler) => Pin::from(err_handler(err)).await,
            ErrHandler::WithInfo(ref err_handler) => {
                Pin::from(err_handler(err, req_info.expect("No RequestInfo is provided"))).await
            }
        }
    }
}

impl<B: HttpBody + Send + Sync + 'static, E: Into<Box<dyn std::error::Error + Send + Sync>> + 'static> Router<B, E> {
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

        self.regex_set =
            Some(RegexSet::new(regex_iter).map_err(|e| Error::new(format!("Couldn't create router RegexSet: {}", e)))?);

        Ok(())
    }

    pub(crate) fn init_req_info_gen(&mut self) {
        if let Some(ErrHandler::WithInfo(_)) = self.err_handler {
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

    pub(crate) fn init_x_powered_by_middleware(&mut self) {
        let x_powered_by_post_middleware = PostMiddleware::new("/*", |mut res| async move {
            res.headers_mut().insert(
                constants::HEADER_NAME_X_POWERED_BY,
                HeaderValue::from_static(constants::HEADER_VALUE_X_POWERED_BY),
            );
            Ok(res)
        })
        .unwrap();

        self.post_middlewares.insert(0, x_powered_by_post_middleware);
    }

    // pub(crate) fn init_keep_alive_middleware(&mut self) {
    //     let keep_alive_post_middleware = PostMiddleware::new("/*", |mut res| async move {
    //         res.headers_mut()
    //             .insert(header::CONNECTION, HeaderValue::from_static("keep-alive"));
    //         Ok(res)
    //     })
    //     .unwrap();

    //     self.post_middlewares.push(keep_alive_post_middleware);
    // }

    pub(crate) fn init_global_options_route(&mut self) {
        let options_method = vec![Method::OPTIONS];
        let found = self
            .routes
            .iter()
            .any(|route| route.path == "/*" && route.methods.as_slice() == options_method.as_slice());

        if found {
            return;
        }

        if let Some(router) = self.downcast_to_hyper_body_type() {
            let options_route: Route<hyper::Body, E> = Route::new("/*", options_method, |_req| async move {
                Ok(Response::builder()
                    .status(StatusCode::NO_CONTENT)
                    .body(hyper::Body::empty())
                    .expect("Couldn't create the default OPTIONS response"))
            })
            .unwrap();

            router.routes.push(options_route);
        } else {
            eprintln!(
                "Warning: No global `options method` route added. It is recommended to send response to any `options` request.\n\
                Please add one by calling `.options(\"/*\", handler)` method of the root router builder.\n"
            );
        }
    }

    pub(crate) fn init_default_404_route(&mut self) {
        let found = self
            .routes
            .iter()
            .any(|route| route.path == "/*" && route.methods.as_slice() == &constants::ALL_POSSIBLE_HTTP_METHODS[..]);

        if found {
            return;
        }

        if let Some(router) = self.downcast_to_hyper_body_type() {
            let default_404_route: Route<hyper::Body, E> =
                Route::new("/*", constants::ALL_POSSIBLE_HTTP_METHODS.to_vec(), |_req| async move {
                    Ok(Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .header(header::CONTENT_TYPE, "text/plain")
                        .body(hyper::Body::from(StatusCode::NOT_FOUND.canonical_reason().unwrap()))
                        .expect("Couldn't create the default 404 response"))
                })
                .unwrap();
            router.routes.push(default_404_route);
        } else {
            eprintln!(
                "Warning: No default 404 route added. It is recommended to send 404 response to any non-existent route.\n\
                Please add one by calling `.any(handler)` method of the root router builder.\n"
            );
        }
    }

    pub(crate) fn init_err_handler(&mut self) {
        let found = self.err_handler.is_some();

        if found {
            return;
        }

        if let Some(router) = self.downcast_to_hyper_body_type() {
            let handler: ErrHandler<hyper::Body> = ErrHandler::WithoutInfo(Box::new(move |err: RouteError| {
                Box::new(async move {
                    Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .header(header::CONTENT_TYPE, "text/plain")
                        .body(hyper::Body::from(format!(
                            "{}: {}",
                            StatusCode::INTERNAL_SERVER_ERROR.canonical_reason().unwrap(),
                            err
                        )))
                        .expect("Couldn't create a response while handling the server error")
                })
            }));
            router.err_handler = Some(handler);
        } else {
            eprintln!(
                "Warning: No error handler added. It is recommended to add one to see what went wrong if any route or middleware fails.\n\
                Please add one by calling `.err_handler(handler)` method of the root router builder.\n"
            );
        }
    }

    fn downcast_to_hyper_body_type(&mut self) -> Option<&mut Router<hyper::Body, E>> {
        let any_obj: &mut dyn Any = self;
        any_obj.downcast_mut::<Router<hyper::Body, E>>()
    }

    /// Return a [RouterBuilder](./struct.RouterBuilder.html) instance to build a `Router`.
    pub fn builder() -> RouterBuilder<B, E> {
        builder::RouterBuilder::new()
    }

    pub(crate) async fn process(
        &self,
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

        let mut route_scope_depth = None;
        for idx in &matched_route_idxs {
            let route = &self.routes[*idx];
            // Middleware should be executed even if there's no route, e.g.
            // logging. Before doing the depth check make sure that there's
            // an actual route match, not a catch-all "/*".
            if route.is_match_method(req.method()) && route.path != "/*" {
                route_scope_depth = Some(route.scope_depth);
                break;
            }
        }

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

        let res_pre = self
            .execute_pre_middleware(req, matched_pre_middleware_idxs, route_scope_depth, req_info.clone())
            .await?;

        // If pre middlewares succeed then execute the route handler.
        // If a pre middleware fails and is able to generate error response
        // (because Router.err_handler is set), then skip directly to post
        // middleware.
        let mut resp = None;
        match res_pre {
            Ok(transformed_req) => {
                for idx in matched_route_idxs {
                    let route = &self.routes[idx];

                    if route.is_match_method(transformed_req.method()) {
                        let route_resp_res = route.process(target_path, transformed_req).await;

                        let route_resp = match route_resp_res {
                            Ok(route_resp) => route_resp,
                            Err(err) => {
                                if let Some(ref err_handler) = self.err_handler {
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
            }
            Err(err_response) => {
                resp = Some(err_response);
            }
        };

        if resp.is_none() {
            let e = "No handlers added to handle non-existent routes. Tips: Please add an '.any' route at the bottom to handle any routes.";
            return Err(crate::Error::new(e).into());
        }

        let mut transformed_res = resp.unwrap();
        for idx in matched_post_middleware_idxs {
            let post_middleware = &self.post_middlewares[idx];
            // Do not execute middleware with the same prefix but from a deeper scope.
            if route_scope_depth.is_none() || post_middleware.scope_depth <= route_scope_depth.unwrap() {
                match post_middleware.process(transformed_res, req_info.clone()).await {
                    Ok(res_resp) => {
                        transformed_res = res_resp;
                    }
                    Err(err) => {
                        if let Some(ref err_handler) = self.err_handler {
                            return Ok(err_handler.execute(err, req_info.clone()).await);
                        } else {
                            return Err(err);
                        }
                    }
                }
            }
        }

        Ok(transformed_res)
    }

    async fn execute_pre_middleware(
        &self,
        req: Request<hyper::Body>,
        matched_pre_middleware_idxs: Vec<usize>,
        route_scope_depth: Option<u32>,
        req_info: Option<RequestInfo>,
    ) -> crate::Result<Result<Request<hyper::Body>, Response<B>>> {
        let mut transformed_req = req;
        for idx in matched_pre_middleware_idxs {
            let pre_middleware = &self.pre_middlewares[idx];
            // Do not execute middleware with the same prefix but from a deeper scope.
            if route_scope_depth.is_none() || pre_middleware.scope_depth <= route_scope_depth.unwrap() {
                match pre_middleware.process(transformed_req).await {
                    Ok(res_req) => {
                        transformed_req = res_req;
                    }
                    Err(err) => {
                        if let Some(ref err_handler) = self.err_handler {
                            return Ok(Err(err_handler.execute(err, req_info).await));
                        } else {
                            return Err(err);
                        }
                    }
                }
            }
        }
        Ok(Ok(transformed_req))
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
