use hyper::{header, Body, Request, Response, StatusCode};

pub async fn default_404_handler(_: Request<Body>) -> crate::Result<Response<Body>> {
    Ok(Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header(header::CONTENT_TYPE, "text/plain")
        .body(Body::from(StatusCode::NOT_FOUND.canonical_reason().unwrap()))
        .expect("Couldn't create the default 404 response"))
}

pub async fn default_options_handler(_: Request<Body>) -> crate::Result<Response<Body>> {
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/plain")
        .body(Body::from(StatusCode::OK.canonical_reason().unwrap()))
        .expect("Couldn't create the default OPTIONS response"))
}

pub async fn default_error_handler(err: crate::Error) -> Response<Body> {
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .header(header::CONTENT_TYPE, "text/plain")
        .body(Body::from(format!(
            "{}: {}",
            StatusCode::INTERNAL_SERVER_ERROR.canonical_reason().unwrap(),
            err
        )))
        .expect("Couldn't create a response while handling the server error")
}
