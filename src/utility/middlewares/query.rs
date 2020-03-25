use crate::Middleware;
use hyper::{Body, Request};
use std::collections::HashMap;
use std::ops::Deref;
use url::Url;

#[derive(Debug, Clone)]
pub struct Query(HashMap<String, String>);

impl Deref for Query {
  type Target = HashMap<String, String>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

pub fn query_parser() -> Middleware {
  Middleware::pre(query_parser_middleware_handler)
}

async fn query_parser_middleware_handler(mut req: Request<Body>) -> crate::Result<Request<Body>> {
  let mut q = Query(HashMap::new());

  if let Some(path_and_query) = req.uri().path_and_query() {
    if let Ok(url) = format!("routerify:{}", path_and_query).parse::<Url>() {
      q = Query(url.query_pairs().into_owned().collect());
    }
  }

  req.extensions_mut().insert(q);

  Ok(req)
}
