use crate::constants;
use crate::helpers;
use crate::middleware::PostMiddleware;
use crate::route::Route;
use crate::router::ErrHandler;
use crate::router::Router;
use crate::service::request_service::RequestService;
use crate::types::RequestMeta;
use hyper::{
    body::HttpBody, header, header::HeaderValue, server::conn::AddrStream, service::Service, Method, Request, Response,
    StatusCode,
};
use std::any::Any;
use std::convert::Infallible;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct RouterService<B, E> {
    router: Router<B, E>,
}

impl<B: HttpBody + Send + Sync + Unpin + 'static, E: std::error::Error + Send + Sync + Unpin + 'static>
    RouterService<B, E>
{
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
            |req| async move {
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
