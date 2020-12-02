use crate::constants;
use crate::middleware::PostMiddleware;
use crate::route::Route;
use crate::router::ErrHandler;
use crate::router::Router;
use crate::service::request_service::RequestService;
use hyper::{
    body::HttpBody,
    header::{self, HeaderValue},
    server::conn::AddrStream,
    service::Service,
    Method, Response, StatusCode,
};
use std::any::Any;
use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// A [`Service`](https://docs.rs/hyper/0.13.5/hyper/service/trait.Service.html) to process incoming requests.
///
/// This `RouterService<B, E>` type accepts two type parameters: `B` and `E`.
///
/// * The `B` represents the response body type which will be used by route handlers and the middlewares and this body type must implement
///   the [HttpBody](https://docs.rs/hyper/0.13.5/hyper/body/trait.HttpBody.html) trait. For an instance, `B` could be [hyper::Body](https://docs.rs/hyper/0.13.5/hyper/body/struct.Body.html)
///   type.
/// * The `E` represents any error type which will be used by route handlers and the middlewares. This error type must implement the [std::error::Error](https://doc.rust-lang.org/std/error/trait.Error.html).
///
/// # Examples
///
/// ```no_run
/// use hyper::{Body, Request, Response, Server};
/// use routerify::{Router, RouterService};
/// use std::convert::Infallible;
/// use std::net::SocketAddr;
///
/// // A handler for "/" page.
/// async fn home(_: Request<Body>) -> Result<Response<Body>, Infallible> {
///     Ok(Response::new(Body::from("Home page")))
/// }
///
/// fn router() -> Router<Body, Infallible> {
///     Router::builder()
///         .get("/", home)
///         .build()
///         .unwrap()
/// }
///
/// #[tokio::main]
/// async fn main() {
///     let router = router();
///
///     // Create a Service from the router above to handle incoming requests.
///     let service = RouterService::new(router).unwrap();
///
///     let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
///
///     // Create a server by passing the created service to `.serve` method.
///     let server = Server::bind(&addr).serve(service);
///
///     println!("App is running on: {}", addr);
///     if let Err(err) = server.await {
///         eprintln!("Server error: {}", err);
///    }
/// }
/// ```
#[derive(Debug)]
pub struct RouterService<B, E> {
    router: Router<B, E>,
}

impl<B: HttpBody + Send + Sync + Unpin + 'static, E: std::error::Error + Send + Sync + Unpin + 'static>
    RouterService<B, E>
{
    /// Creates a new service with the provided router and it's ready to be used with the hyper [`serve`](https://docs.rs/hyper/0.13.5/hyper/server/struct.Builder.html#method.serve)
    /// method.
    pub fn new(mut router: Router<B, E>) -> crate::Result<RouterService<B, E>> {
        Self::init_router_with_x_powered_by_middleware(&mut router);
        // Self::init_router_with_keep_alive_middleware(&mut router);

        Self::init_router_with_global_options_route(&mut router);
        Self::init_router_with_default_404_route(&mut router);

        Self::init_router_with_err_handler(&mut router);

        router.init_regex_set()?;
        router.init_req_info_gen()?;

        Ok(RouterService { router })
    }

    fn init_router_with_x_powered_by_middleware(router: &mut Router<B, E>) {
        let x_powered_by_post_middleware = PostMiddleware::new("/*", |mut res| async move {
            res.headers_mut().insert(
                constants::HEADER_NAME_X_POWERED_BY,
                HeaderValue::from_static(constants::HEADER_VALUE_X_POWERED_BY),
            );
            Ok(res)
        })
        .unwrap();

        router.post_middlewares.insert(0,x_powered_by_post_middleware);
    }

    // fn init_router_with_keep_alive_middleware(router: &mut Router<B, E>) {
    //     let keep_alive_post_middleware = PostMiddleware::new("/*", |mut res| async move {
    //         res.headers_mut()
    //             .insert(header::CONNECTION, HeaderValue::from_static("keep-alive"));
    //         Ok(res)
    //     })
    //     .unwrap();

    //     router.post_middlewares.push(keep_alive_post_middleware);
    // }

    fn init_router_with_global_options_route(router: &mut Router<B, E>) {
        let options_method = vec![Method::OPTIONS];
        let found = router
            .routes
            .iter()
            .any(|route| route.path == "/*" && route.methods.as_slice() == options_method.as_slice());

        if found {
            return;
        }

        if let Some(router) = Self::downcast_router_to_hyper_body_type(router) {
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

    fn init_router_with_default_404_route(router: &mut Router<B, E>) {
        let found = router
            .routes
            .iter()
            .any(|route| route.path == "/*" && route.methods.as_slice() == &constants::ALL_POSSIBLE_HTTP_METHODS[..]);

        if found {
            return;
        }

        if let Some(router) = Self::downcast_router_to_hyper_body_type(router) {
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

    fn init_router_with_err_handler(router: &mut Router<B, E>) {
        let found = router.err_handler.is_some();

        if found {
            return;
        }

        if let Some(router) = Self::downcast_router_to_hyper_body_type(router) {
            let handler: ErrHandler<hyper::Body> = ErrHandler::WithoutInfo(Box::new(move |err: crate::Error| {
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

    fn downcast_router_to_hyper_body_type(router: &mut Router<B, E>) -> Option<&mut Router<hyper::Body, E>> {
        let any_obj: &mut dyn Any = router;
        any_obj.downcast_mut::<Router<hyper::Body, E>>()
    }
}

impl<B: HttpBody + Send + Sync + Unpin + 'static, E: std::error::Error + Send + Sync + Unpin + 'static>
    Service<&AddrStream> for RouterService<B, E>
{
    type Response = RequestService<B, E>;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, conn: &AddrStream) -> Self::Future {
        let remote_addr = conn.remote_addr();

        let req_service = RequestService {
            router: &mut self.router,
            remote_addr,
        };

        let fut = async move { Ok(req_service) };

        Box::pin(fut)
    }
}
