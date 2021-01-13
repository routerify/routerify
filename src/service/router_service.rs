use crate::router::Router;
use crate::service::request_service::{RequestService, RequestServiceBuilder};
use hyper::{body::HttpBody, server::conn::AddrStream, service::Service};
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
    builder: RequestServiceBuilder<B, E>,
}

impl<B: HttpBody + Send + Sync + 'static, E: Into<Box<dyn std::error::Error + Send + Sync>> + 'static>
    RouterService<B, E>
{
    /// Creates a new service with the provided router and it's ready to be used with the hyper [`serve`](https://docs.rs/hyper/0.13.5/hyper/server/struct.Builder.html#method.serve)
    /// method.
    pub fn new(router: Router<B, E>) -> crate::Result<RouterService<B, E>> {
        let builder = RequestServiceBuilder::new(router)?;
        Ok(RouterService { builder })
    }
}

impl<B: HttpBody + Send + Sync + 'static, E: Into<Box<dyn std::error::Error + Send + Sync>> + 'static>
    Service<&AddrStream> for RouterService<B, E>
{
    type Response = RequestService<B, E>;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, conn: &AddrStream) -> Self::Future {
        let req_service = self.builder.build(conn.remote_addr());

        let fut = async move { Ok(req_service) };

        Box::pin(fut)
    }
}
