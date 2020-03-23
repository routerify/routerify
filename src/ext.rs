use crate::types::{PathParams, RequestData};
use hyper::{Body, Request};

pub trait RequestExt {
  fn params(&self) -> Option<&PathParams>;
}

impl RequestExt for Request<Body> {
  fn params(&self) -> Option<&PathParams> {
    let ext = self.extensions();

    if let Some(data) = ext.get::<RequestData>() {
      Some(data.path_params())
    } else {
      None
    }
  }
}
