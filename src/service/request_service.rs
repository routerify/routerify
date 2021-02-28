use crate::helpers;
use crate::router::Router;
use crate::types::{RequestContext, RequestInfo, RequestMeta};
use hyper::{body::HttpBody, service::Service, Request, Response};
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::sync::Arc;

pub struct RequestService<B, E> {
    pub(crate) router: Arc<Router<B, E>>,
    pub(crate) remote_addr: SocketAddr,
}

impl<
        B: HttpBody + Send + Sync + 'static,
        E: Into<Box<dyn std::error::Error + Send + Sync>> + 'static,
    > Service<Request<hyper::Body>> for RequestService<B, E>
{
    type Response = Response<B>;
    type Error = crate::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, mut req: Request<hyper::Body>) -> Self::Future {
        let router = self.router.clone();
        let remote_addr = self.remote_addr;

        let fut = async move {
            helpers::update_req_meta_in_extensions(req.extensions_mut(), RequestMeta::with_remote_addr(remote_addr));

            let mut target_path = helpers::percent_decode_request_path(req.uri().path())?;

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

            match router.process(target_path.as_str(), req, req_info.clone()).await {
                Ok(resp) => crate::Result::Ok(resp),
                Err(err) => {
                    if let Some(ref err_handler) = router.err_handler {
                        crate::Result::Ok(err_handler.execute(err, req_info.clone()).await)
                    } else {
                        crate::Result::Err(err)
                    }
                }
            }
        };

        Box::pin(fut)
    }
}
