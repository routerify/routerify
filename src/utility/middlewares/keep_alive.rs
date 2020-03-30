use crate::Middleware;
use hyper::header::{self, HeaderValue};
use hyper::{Body, Response};

pub fn enable_keep_alive() -> Middleware {
    Middleware::post(enable_keep_alive_middleware_handler)
}

async fn enable_keep_alive_middleware_handler(mut res: Response<Body>) -> crate::Result<Response<Body>> {
    let headers = res.headers_mut();
    headers.insert(header::CONNECTION, HeaderValue::from_static("keep-alive"));
    Ok(res)
}
