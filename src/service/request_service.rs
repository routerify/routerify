use crate::helpers;
use crate::router::Router;
use crate::types::{RequestContext, RequestInfo, RequestMeta};
use crate::Error;
use hyper::{body::HttpBody, service::Service, Request, Response};
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

pub struct RequestService<B, E> {
    pub(crate) router: Arc<Router<B, E>>,
    pub(crate) remote_addr: SocketAddr,
}

impl<B: HttpBody + Send + Sync + 'static, E: Into<Box<dyn std::error::Error + Send + Sync>> + 'static>
    Service<Request<hyper::Body>> for RequestService<B, E>
{
    type Response = Response<B>;
    type Error = crate::RouteError;
    #[allow(clippy::type_complexity)]
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, mut req: Request<hyper::Body>) -> Self::Future {
        let router = self.router.clone();
        let remote_addr = self.remote_addr;

        let fut = async move {
            helpers::update_req_meta_in_extensions(req.extensions_mut(), RequestMeta::with_remote_addr(remote_addr));

            let mut target_path = helpers::percent_decode_request_path(req.uri().path())
                .map_err(|e| Error::new(format!("Couldn't percent decode request path: {}", e)))?;

            if target_path.is_empty() || target_path.as_bytes()[target_path.len() - 1] != b'/' {
                target_path.push('/');
            }

            let mut req_info = None;
            let should_gen_req_info = router
                .should_gen_req_info
                .expect("The `should_gen_req_info` flag in Router is not initialized");

            let context = RequestContext::new();

            if should_gen_req_info {
                req_info = Some(RequestInfo::new_from_req(&req, context.clone()));
            }

            req.extensions_mut().insert(context);

            router.process(target_path.as_str(), req, req_info.clone()).await
        };

        Box::pin(fut)
    }
}

#[derive(Debug)]
pub struct RequestServiceBuilder<B, E> {
    router: Arc<Router<B, E>>,
}

impl<B, E> Clone for RequestServiceBuilder<B, E> {
    fn clone(&self) -> Self {
        Self { router: self.router.clone() }
    }
}

impl<B: HttpBody + Send + Sync + 'static, E: Into<Box<dyn std::error::Error + Send + Sync>> + 'static>
    RequestServiceBuilder<B, E>
{
    pub fn new(mut router: Router<B, E>) -> crate::Result<Self> {
        router.init_x_powered_by_middleware();
        // router.init_keep_alive_middleware();

        router.init_global_options_route();
        router.init_default_404_route();

        router.init_err_handler();

        router.init_regex_set()?;
        router.init_req_info_gen();
        Ok(Self {
            router: Arc::from(router),
        })
    }

    pub fn build(&self, remote_addr: SocketAddr) -> RequestService<B, E> {
        RequestService {
            router: self.router.clone(),
            remote_addr,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Error, RequestServiceBuilder, RouteError, Router};
    use futures::future::poll_fn;
    use http::Method;
    use hyper::service::Service;
    use hyper::{Body, Request, Response};
    use std::net::SocketAddr;
    use std::str::FromStr;
    use std::task::Poll;

    #[tokio::test]
    async fn should_route_request() {
        const RESPONSE_TEXT: &str = "Hello world!";
        let remote_addr = SocketAddr::from_str("0.0.0.0:8080").unwrap();
        let router: Router<hyper::body::Body, Error> = Router::builder()
            .get("/", |_| async move { Ok(Response::new(Body::from(RESPONSE_TEXT))) })
            .build()
            .unwrap();
        let req = Request::builder()
            .method(Method::GET)
            .uri("/")
            .body(hyper::Body::empty())
            .unwrap();
        let builder = RequestServiceBuilder::new(router).unwrap();
        let mut service = builder.build(remote_addr);
        poll_fn(|ctx| -> Poll<Result<(), RouteError>> { service.poll_ready(ctx) })
            .await
            .expect("request service is not ready");
        let resp: Response<hyper::body::Body> = service.call(req).await.unwrap();
        let body = resp.into_body();
        let body = String::from_utf8(hyper::body::to_bytes(body).await.unwrap().to_vec()).unwrap();
        assert_eq!(RESPONSE_TEXT, body)
    }
}
