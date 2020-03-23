use crate::middleware::{PostMiddleware, PreMiddleware};
use crate::prelude::*;
use crate::route::Route;
use hyper::{Body, Request, Response, StatusCode};

pub use self::builder::Builder as RouterBuilder;

mod builder;

pub struct Router {
  pre_middlewares: Vec<PreMiddleware>,
  routes: Vec<Route>,
  post_middlewares: Vec<PostMiddleware>,
}

impl Router {
  pub fn builder() -> RouterBuilder {
    builder::Builder::new()
  }

  pub async fn process(&self, target_path: &str, req: Request<Body>) -> crate::Result<Response<Body>> {
    let Router {
      ref pre_middlewares,
      ref routes,
      ref post_middlewares,
    } = self;

    let mut transformed_req = req;
    for pre_middleware in pre_middlewares.iter() {
      transformed_req = pre_middleware
        .process(transformed_req)
        .await
        .context("One of the pre middlewares couldn't process the request")?;
    }

    let mut resp: Option<Response<Body>> = None;
    for route in routes.iter() {
      let matched = route.is_match(target_path, transformed_req.method());
      if matched {
        resp = Some(
          route
            .process(target_path, transformed_req)
            .await
            .context("One of the routes couldn't process the request")?,
        );
        break;
      }
    }

    if let None = resp {
      return Ok(
        Response::builder()
          .status(StatusCode::NOT_FOUND)
          .body(Body::empty())
          .unwrap(),
      );
    }

    let mut transformed_res = resp.unwrap();
    for post_middleware in post_middlewares.iter() {
      transformed_res = post_middleware
        .process(transformed_res)
        .await
        .context("One of the post middlewares couldn't process the request")?;
    }

    Ok(transformed_res)
  }
}
