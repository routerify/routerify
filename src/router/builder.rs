use crate::middleware::{Middleware, PostMiddleware, PreMiddleware};
use crate::route::Route;
use crate::router::Router;
use hyper::upgrade::Upgraded;
use hyper::{Body, Method, Request, Response};
use std::future::Future;

struct BuilderInner {
  pre_middlewares: Vec<PreMiddleware>,
  routes: Vec<Route>,
  post_middlewares: Vec<PostMiddleware>,
}

pub struct Builder {
  inner: crate::Result<BuilderInner>,
}

impl Builder {
  pub fn new() -> Self {
    Builder::default()
  }

  pub fn build(self) -> crate::Result<Router> {
    self.inner.map(|inner| Router {
      pre_middlewares: inner.pre_middlewares,
      routes: inner.routes,
      post_middlewares: inner.post_middlewares,
    })
  }

  fn and_then<F>(self, func: F) -> Self
  where
    F: FnOnce(BuilderInner) -> crate::Result<BuilderInner>,
  {
    Builder {
      inner: self.inner.and_then(func),
    }
  }
}

impl Builder {
  pub fn get<P, H, R>(self, path: P, handler: H) -> Self
  where
    P: Into<String>,
    H: Fn(Request<Body>) -> R + Send + Sync + 'static,
    R: Future<Output = crate::Result<Response<Body>>> + Send + Sync + 'static,
  {
    self.push_method_route(path, Method::GET, handler)
  }

  pub fn post<P, H, R>(self, path: P, handler: H) -> Self
  where
    P: Into<String>,
    H: Fn(Request<Body>) -> R + Send + Sync + 'static,
    R: Future<Output = crate::Result<Response<Body>>> + Send + Sync + 'static,
  {
    self.push_method_route(path, Method::POST, handler)
  }

  pub fn put<P, H, R>(self, path: P, handler: H) -> Self
  where
    P: Into<String>,
    H: Fn(Request<Body>) -> R + Send + Sync + 'static,
    R: Future<Output = crate::Result<Response<Body>>> + Send + Sync + 'static,
  {
    self.push_method_route(path, Method::PUT, handler)
  }

  pub fn delete<P, H, R>(self, path: P, handler: H) -> Self
  where
    P: Into<String>,
    H: Fn(Request<Body>) -> R + Send + Sync + 'static,
    R: Future<Output = crate::Result<Response<Body>>> + Send + Sync + 'static,
  {
    self.push_method_route(path, Method::DELETE, handler)
  }

  pub fn head<P, H, R>(self, path: P, handler: H) -> Self
  where
    P: Into<String>,
    H: Fn(Request<Body>) -> R + Send + Sync + 'static,
    R: Future<Output = crate::Result<Response<Body>>> + Send + Sync + 'static,
  {
    self.push_method_route(path, Method::HEAD, handler)
  }

  pub fn trace<P, H, R>(self, path: P, handler: H) -> Self
  where
    P: Into<String>,
    H: Fn(Request<Body>) -> R + Send + Sync + 'static,
    R: Future<Output = crate::Result<Response<Body>>> + Send + Sync + 'static,
  {
    self.push_method_route(path, Method::TRACE, handler)
  }

  pub fn connect<P, H, R>(self, path: P, handler: H) -> Self
  where
    P: Into<String>,
    H: Fn(Request<Body>) -> R + Send + Sync + 'static,
    R: Future<Output = crate::Result<Response<Body>>> + Send + Sync + 'static,
  {
    self.push_method_route(path, Method::CONNECT, handler)
  }

  pub fn patch<P, H, R>(self, path: P, handler: H) -> Self
  where
    P: Into<String>,
    H: Fn(Request<Body>) -> R + Send + Sync + 'static,
    R: Future<Output = crate::Result<Response<Body>>> + Send + Sync + 'static,
  {
    self.push_method_route(path, Method::PATCH, handler)
  }

  fn push_method_route<P, H, R>(self, path: P, method: Method, handler: H) -> Self
  where
    P: Into<String>,
    H: Fn(Request<Body>) -> R + Send + Sync + 'static,
    R: Future<Output = crate::Result<Response<Body>>> + Send + Sync + 'static,
  {
    self.and_then(move |mut inner| {
      let route = Route::from_route_handler(path, method, handler)?;
      inner.routes.push(route);
      crate::Result::Ok(inner)
    })
  }

  pub fn router<P>(self, path: P, router: &'static Router) -> Self
  where
    P: Into<String>,
  {
    self.and_then(move |mut inner| {
      let route = Route::from_router_handler(path, router)?;
      inner.routes.push(route);
      crate::Result::Ok(inner)
    })
  }

  pub fn upgrade<P, H, R>(self, path: P, handler: H) -> Self
  where
    P: Into<String>,
    H: Fn(Upgraded) -> R + Send + Sync + 'static,
    R: Future<Output = crate::Result<()>> + Send + Sync + 'static,
  {
    self.and_then(move |mut inner| {
      let route = Route::from_ws_handler(path, handler)?;
      inner.routes.push(route);
      crate::Result::Ok(inner)
    })
  }
}

impl Builder {
  pub fn middleware(self, m: Middleware) -> Self {
    self.and_then(move |mut inner| {
      match m {
        Middleware::Pre(middleware) => {
          inner.pre_middlewares.push(middleware);
        }
        Middleware::Post(middleware) => {
          inner.post_middlewares.push(middleware);
        }
      }
      crate::Result::Ok(inner)
    })
  }
}

impl Default for Builder {
  fn default() -> Self {
    Builder {
      inner: Ok(BuilderInner {
        post_middlewares: Vec::new(),
        pre_middlewares: Vec::new(),
        routes: Vec::new(),
      }),
    }
  }
}
