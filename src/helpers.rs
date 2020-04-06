use crate::types::RequestData;
use crate::utility::handlers;
use crate::Router;
use http::Extensions;
use hyper::{Body, Request, Response};
use std::future::Future;
use std::net::SocketAddr;

pub async fn handle_request_with_err<H, R>(
    router: &'static Router,
    mut req: Request<Body>,
    remote_addr: Option<SocketAddr>,
    error_handler: H,
) -> Response<Body>
where
    H: Fn(crate::Error) -> R + Send + Sync + 'static,
    R: Future<Output = Response<Body>> + Send + 'static,
{
    if let Some(remote_addr) = remote_addr {
        update_req_data_in_extensions(req.extensions_mut(), RequestData::with_remote_addr(remote_addr));
    }

    let target_path = String::from(req.uri().path());
    let resp = router.process(target_path.as_str(), req).await;

    match resp {
        Ok(resp) => resp,
        Err(err) => error_handler(err).await,
    }
}

pub async fn handle_request(
    router: &'static Router,
    req: Request<Body>,
    remote_addr: Option<SocketAddr>,
) -> Response<Body> {
    handle_request_with_err(router, req, remote_addr, handlers::default_error_handler).await
}

pub(crate) fn update_req_data_in_extensions(ext: &mut Extensions, new_req_data: RequestData) {
    if let Some(existing_req_data) = ext.get_mut::<RequestData>() {
        existing_req_data.extend(new_req_data);
    } else {
        ext.insert(new_req_data);
    }
}
