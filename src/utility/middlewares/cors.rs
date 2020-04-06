use crate::Middleware;
use hyper::header::{self, HeaderValue};
use hyper::{Body, Response};

pub fn cors_enable_all() -> Middleware {
    Middleware::post(cors_enable_all_middleware_handler)
}

async fn cors_enable_all_middleware_handler(mut res: Response<Body>) -> crate::Result<Response<Body>> {
    let headers = res.headers_mut();
    headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*"));
    headers.insert(header::ACCESS_CONTROL_ALLOW_METHODS, HeaderValue::from_static("*"));
    headers.insert(header::ACCESS_CONTROL_ALLOW_HEADERS, HeaderValue::from_static("*"));

    Ok(res)
}
