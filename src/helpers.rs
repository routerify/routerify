use crate::Router;
use hyper::{Body, Request, Response, StatusCode};
use std::future::Future;

pub async fn handle_request_err<H, R>(router: &'static Router, req: Request<Body>, error_handler: H) -> Response<Body>
where
  H: Fn(crate::Error) -> R + Send + Sync + 'static,
  R: Future<Output = Response<Body>> + Send + Sync + 'static,
{
  let target_path = String::from(req.uri().path());
  let resp = router.process(target_path.as_str(), req).await;

  match resp {
    Ok(resp) => resp,
    Err(err) => error_handler(err).await,
  }
}

pub async fn handle_request(router: &'static Router, req: Request<Body>) -> Response<Body> {
  handle_request_err(router, req, default_http_error_handler).await
}

async fn default_http_error_handler(err: crate::Error) -> Response<Body> {
  let msg = err.to_string();

  Response::builder()
    .status(StatusCode::INTERNAL_SERVER_ERROR)
    .header("Content-Type", "application/json")
    .body(Body::from(format!("{{ message: {} }}", msg)))
    .unwrap()
}
