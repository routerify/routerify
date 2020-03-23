use hyper::{Body, Response};
use std::future::Future;
use std::pin::Pin;

type BoxedPostHandler = Box<dyn Fn(Response<Body>) -> BoxedTransformedResponse + Send + Sync + 'static>;
type BoxedTransformedResponse = Box<dyn Future<Output = crate::Result<Response<Body>>> + Send + Sync + 'static>;

pub struct PostMiddleware {
  handler: BoxedPostHandler,
}

impl PostMiddleware {
  pub fn new<H, R>(handler: H) -> Self
  where
    H: Fn(Response<Body>) -> R + Send + Sync + 'static,
    R: Future<Output = crate::Result<Response<Body>>> + Send + Sync + 'static,
  {
    let handler: BoxedPostHandler = Box::new(move |req: Response<Body>| Box::new(handler(req)));
    PostMiddleware { handler }
  }

  pub async fn process(&self, res: Response<Body>) -> crate::Result<Response<Body>> {
    Pin::from((self.handler)(res)).await
  }
}
