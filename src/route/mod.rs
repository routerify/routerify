use crate::prelude::*;
use crate::router::Router;
use crate::{PathParams, RequestData};
use futures::future::{BoxFuture, FutureExt};
use hyper::upgrade::Upgraded;
use hyper::{Body, Method, Request, Response};
use regex::Regex;
use std::future::Future;
use std::pin::Pin;

mod regex_generator;

type BoxedRouteHandler = Box<dyn Fn(Request<Body>) -> BoxedRouteResponse + Send + Sync + 'static>;
type BoxedRouteResponse = Box<dyn Future<Output = crate::Result<Response<Body>>> + Send + Sync + 'static>;
type BoxedWsHandler = Box<dyn Fn(Upgraded) -> BoxedWsResponse + Send + Sync + 'static>;
type BoxedWsResponse = Box<dyn Future<Output = crate::Result<()>> + Send + Sync + 'static>;

pub struct Route {
  path: String,
  method: Option<Method>,
  route_handler: Option<BoxedRouteHandler>,
  router: Option<&'static Router>,
  ws_handler: Option<BoxedWsHandler>,
  regex: Regex,
  path_params: Vec<String>,
}

impl Route {
  pub fn from_route_handler<P, H, R>(path: P, method: Method, handler: H) -> crate::Result<Route>
  where
    P: Into<String>,
    H: Fn(Request<Body>) -> R + Send + Sync + 'static,
    R: Future<Output = crate::Result<Response<Body>>> + Send + Sync + 'static,
  {
    let path = path.into();
    let (re, params) = Route::gen_exact_match_regex(path.as_str())?;

    let handler: BoxedRouteHandler = Box::new(move |req: Request<Body>| Box::new(handler(req)));
    Ok(Route {
      path,
      method: Some(method),
      route_handler: Some(handler),
      router: None,
      ws_handler: None,
      regex: re,
      path_params: params,
    })
  }

  pub fn from_router_handler<P>(path: P, router: &'static Router) -> crate::Result<Route>
  where
    P: Into<String>,
  {
    let path = path.into();
    let (re, params) = Self::gen_prefix_match_regex(path.as_str())?;

    Ok(Route {
      path,
      method: None,
      route_handler: None,
      router: Some(router),
      ws_handler: None,
      regex: re,
      path_params: params,
    })
  }

  pub fn from_ws_handler<P, H, R>(path: P, handler: H) -> crate::Result<Route>
  where
    P: Into<String>,
    H: Fn(Upgraded) -> R + Send + Sync + 'static,
    R: Future<Output = crate::Result<()>> + Send + Sync + 'static,
  {
    let path = path.into();
    let (re, params) = Self::gen_exact_match_regex(path.as_str())?;

    let handler: BoxedWsHandler = Box::new(move |upgraded: Upgraded| Box::new(handler(upgraded)));
    Ok(Route {
      path: path.into(),
      method: None,
      route_handler: None,
      router: None,
      ws_handler: Some(handler),
      regex: re,
      path_params: params,
    })
  }

  fn gen_exact_match_regex(path: &str) -> crate::Result<(Regex, Vec<String>)> {
    regex_generator::generate_exact_match_regex(path)
      .context("Could not create an exact match regex for the route path")
  }

  fn gen_prefix_match_regex(path: &str) -> crate::Result<(Regex, Vec<String>)> {
    regex_generator::generate_prefix_match_regex(path)
      .context("Could not create a prefix match regex for the route path")
  }

  pub fn is_match(&self, target_path: &str, method: &Method) -> bool {
    if let (Some(_), Some(ref m)) = (&self.route_handler, &self.method) {
      return method == m && self.regex.is_match(target_path);
    } else if let Some(_) = self.router {
      return self.regex.is_match(target_path);
    } else if let Some(_) = self.ws_handler {
      return self.regex.is_match(target_path);
    }

    false
  }

  pub fn path(&self) -> &String {
    &self.path
  }

  pub async fn process(&self, target_path: &str, req: Request<Body>) -> crate::Result<Response<Body>> {
    if let (Some(ref handler), Some(_)) = (&self.route_handler, &self.method) {
      return self.process_route_req(target_path, req, handler).await;
    } else if let Some(router) = self.router {
      return self.process_router_req(router, target_path, req).await;
    } else if let Some(_) = self.ws_handler {
      return self.process_ws_req(req).await;
    }

    Err(crate::Error::new(
      "Couldn't handle the request as there is no matching handler",
    ))
  }

  async fn process_route_req(
    &self,
    target_path: &str,
    mut req: Request<Body>,
    handler: &BoxedRouteHandler,
  ) -> crate::Result<Response<Body>> {
    let path_params_list = &self.path_params;
    let ln = path_params_list.len();

    let path_params = if ln > 0 {
      (0..ln)
        .zip(self.regex.captures_iter(target_path))
        .fold(PathParams::with_capacity(ln), |mut acc, (idx, caps)| {
          let whole = caps.get(0).unwrap();
          acc.set(path_params_list[idx].clone(), String::from(whole.as_str()));
          acc
        })
    } else {
      PathParams::new()
    };

    let req_data = RequestData::new(path_params);
    req.extensions_mut().insert(req_data);
    Pin::from(handler(req)).await
  }

  fn process_router_req(
    &self,
    router: &'static Router,
    target_path: &str,
    req: Request<Body>,
  ) -> BoxFuture<'static, crate::Result<Response<Body>>> {
    let target_path: String = self.regex.replace(target_path, "").into();
    async move { router.process(target_path.as_str(), req).await }.boxed()
  }

  async fn process_ws_req(&self, _req: Request<Body>) -> crate::Result<Response<Body>> {
    todo!("Websocket support is not yet added");
  }
}
