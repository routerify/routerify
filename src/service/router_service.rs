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
///     let service = RouterService::new(router);
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
pub struct RouterService<B, E> {
    router: Router<B, E>,
}

impl<B: HttpBody + Send + Sync + Unpin + 'static, E: std::error::Error + Send + Sync + Unpin + 'static>
    RouterService<B, E>
{
    /// Creates a new service with the provided router and it's ready to be used with the hyper [`serve`](https://docs.rs/hyper/0.13.5/hyper/server/struct.Builder.html#method.serve)
    /// method.
    pub fn new(mut router: Router<B, E>) -> RouterService<B, E> {
        let x_powered_by_post_middleware = PostMiddleware::new("/*", |mut res| async move {
            res.headers_mut().insert(
                constants::X_POWERED_BY_HEADER_NAME,
                HeaderValue::from_static(constants::X_POWERED_BY_HEADER_VAL),
            );
            Ok(res)
        })
        .unwrap();
        router.post_middlewares.push(x_powered_by_post_middleware);

        let keep_alive_post_middleware = PostMiddleware::new("/*", |mut res| async move {
            res.headers_mut()
                .insert(header::CONNECTION, HeaderValue::from_static("keep-alive"));
            Ok(res)
        })
        .unwrap();
        router.post_middlewares.push(keep_alive_post_middleware);

        let any_obj: &mut dyn Any = &mut router;
        if let Some(router) = any_obj.downcast_mut::<Router<hyper::Body, E>>() {
            Self::init_router_for_hyper_body(router);
        }

        if let None = router.err_handler {
            eprintln!("No error handler added. Please add one by calling `root_router_builder.err_handler(handler)`");
        }

        RouterService { router }
    }

    fn init_router_for_hyper_body(router: &mut Router<hyper::Body, E>) {
        let options_route: Route<hyper::Body, E> = Route::new("/*", vec![Method::OPTIONS], |_req| async move {
            Ok(Response::builder()
                .status(StatusCode::NO_CONTENT)
                .body(hyper::Body::empty())
                .expect("Couldn't create the default OPTIONS response"))
        })
        .unwrap();
        router.routes.push(options_route);

        let default_404_route: Route<hyper::Body, E> = Route::new(
            "/*",
            vec![
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::PATCH,
                Method::DELETE,
                Method::CONNECT,
                Method::HEAD,
                Method::OPTIONS,
                Method::TRACE,
            ],
            |_req| async move {
                Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .header(header::CONTENT_TYPE, "text/plain")
                    .body(hyper::Body::from(StatusCode::NOT_FOUND.canonical_reason().unwrap()))
                    .expect("Couldn't create the default 404 response"))
            },
        )
        .unwrap();
        router.routes.push(default_404_route);

        if let None = router.err_handler {
            let handler: ErrHandler<hyper::Body> = Box::new(move |err: crate::Error| {
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
            });
            router.err_handler = Some(handler);
        }
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
