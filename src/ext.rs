use crate::types::{PathParams, RequestData};
use crate::utility::middlewares::Query;
use hyper::{Body, Request};
use std::net::SocketAddr;

pub trait RequestExt {
  fn params(&self) -> Option<&PathParams>;
  fn remote_addr(&self) -> Option<&SocketAddr>;
  fn query(&self) -> Option<&Query>;
}

impl RequestExt for Request<Body> {
  fn params(&self) -> Option<&PathParams> {
    let ext = self.extensions();

    if let Some(data) = ext.get::<RequestData>() {
      data.path_params()
    } else {
      None
    }
  }

  fn remote_addr(&self) -> Option<&SocketAddr> {
    let ext = self.extensions();

    if let Some(data) = ext.get::<RequestData>() {
      data.remote_addr()
    } else {
      None
    }
  }
  fn query(&self) -> Option<&Query> {
    self.extensions().get::<Query>()
  }
}
