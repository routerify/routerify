use crate::helpers;
use crate::prelude::*;
use crate::router::Router;
use crate::service::BoxedFutureResult;
use crate::types::{RequestInfo, RequestMeta};
use hyper::{body::HttpBody, service::Service, Request, Response};
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::sync::Mutex;

pub struct RequestService<B, E> {
    pub(crate) router: Arc<Mutex<Router<B, E>>>,
    pub(crate) remote_addr: SocketAddr,
}

impl<B, E> Service<Request<hyper::Body>> for RequestService<B, E>
where
    B: HttpBody + Send + Sync + 'static,
    E: std::error::Error + Send + Sync + 'static,
{
    type Response = Response<B>;
    type Error = crate::Error;
    type Future = Pin<BoxedFutureResult<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, mut req: Request<hyper::Body>) -> Self::Future {
        helpers::update_req_meta_in_extensions(req.extensions_mut(), RequestMeta::with_remote_addr(self.remote_addr));

        let router = self.router.clone();
        Box::pin(async move {
            let mut target_path = helpers::percent_decode_request_path(req.uri().path())
                .context("Couldn't percent decode request path")?;

            if target_path.as_bytes()[target_path.len() - 1] != b'/' {
                target_path.push('/');
            }

            let mut router = router.lock().await;
            let should_gen_req_info = router
                .should_gen_req_info
                .expect("The `should_gen_req_info` flag in Router is not initialized");
            let req_info = if should_gen_req_info {
                Some(RequestInfo::new_from_req(&req))
            } else {
                None
            };

            match router.process(target_path.as_str(), req, req_info.clone()).await {
                Ok(resp) => Ok(resp),
                Err(err) => {
                    if let Some(ref mut err_handler) = router.err_handler {
                        Ok(err_handler.execute(err, req_info).await)
                    } else {
                        Err(err)
                    }
                }
            }
        })
    }
}
