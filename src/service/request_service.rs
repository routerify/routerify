use crate::helpers;
use crate::router::Router;
use crate::types::RequestMeta;
use hyper::{body::HttpBody, service::Service, Request, Response};
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct RequestService<B, E> {
    pub(crate) router: *mut Router<B, E>,
    pub(crate) remote_addr: SocketAddr,
}

unsafe impl<B: HttpBody + Send + Sync + Unpin + 'static, E: std::error::Error + Send + Sync + Unpin + 'static> Send
    for RequestService<B, E>
{
}
unsafe impl<B: HttpBody + Send + Sync + Unpin + 'static, E: std::error::Error + Send + Sync + Unpin + 'static> Sync
    for RequestService<B, E>
{
}

impl<B: HttpBody + Send + Sync + Unpin + 'static, E: std::error::Error + Send + Sync + Unpin + 'static>
    Service<Request<hyper::Body>> for RequestService<B, E>
{
    type Response = Response<B>;
    type Error = crate::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, mut req: Request<hyper::Body>) -> Self::Future {
        helpers::update_req_meta_in_extensions(req.extensions_mut(), RequestMeta::with_remote_addr(self.remote_addr));

        let router = unsafe { &mut *self.router };

        let fut = async move {
            match router.process(req).await {
                Ok(resp) => crate::Result::Ok(resp),
                Err(err) => {
                    if let Some(ref mut err_handler) = router.err_handler {
                        crate::Result::Ok(Pin::from(err_handler(err)).await)
                    } else {
                        crate::Result::Err(err)
                    }
                }
            }
        };

        Box::pin(fut)
    }
}
