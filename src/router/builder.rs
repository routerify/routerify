use crate::middleware::{Middleware, PostMiddleware, PreMiddleware};
use crate::route::Route;
use crate::router::Router;
use crate::utility::handlers;
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
    self
      .options("*", handlers::default_options_handler)
      .all(handlers::default_404_handler)
      .inner
      .map(|inner| Router {
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
    self.add(path, vec![Method::GET], handler)
  }

  pub fn get_or_head<P, H, R>(self, path: P, handler: H) -> Self
  where
    P: Into<String>,
    H: Fn(Request<Body>) -> R + Send + Sync + 'static,
    R: Future<Output = crate::Result<Response<Body>>> + Send + Sync + 'static,
  {
    self.add(path, vec![Method::GET, Method::HEAD], handler)
  }

  pub fn post<P, H, R>(self, path: P, handler: H) -> Self
  where
    P: Into<String>,
    H: Fn(Request<Body>) -> R + Send + Sync + 'static,
    R: Future<Output = crate::Result<Response<Body>>> + Send + Sync + 'static,
  {
    self.add(path, vec![Method::POST], handler)
  }

  pub fn put<P, H, R>(self, path: P, handler: H) -> Self
  where
    P: Into<String>,
    H: Fn(Request<Body>) -> R + Send + Sync + 'static,
    R: Future<Output = crate::Result<Response<Body>>> + Send + Sync + 'static,
  {
    self.add(path, vec![Method::PUT], handler)
  }

  pub fn delete<P, H, R>(self, path: P, handler: H) -> Self
  where
    P: Into<String>,
    H: Fn(Request<Body>) -> R + Send + Sync + 'static,
    R: Future<Output = crate::Result<Response<Body>>> + Send + Sync + 'static,
  {
    self.add(path, vec![Method::DELETE], handler)
  }

  pub fn head<P, H, R>(self, path: P, handler: H) -> Self
  where
    P: Into<String>,
    H: Fn(Request<Body>) -> R + Send + Sync + 'static,
    R: Future<Output = crate::Result<Response<Body>>> + Send + Sync + 'static,
  {
    self.add(path, vec![Method::HEAD], handler)
  }

  pub fn trace<P, H, R>(self, path: P, handler: H) -> Self
  where
    P: Into<String>,
    H: Fn(Request<Body>) -> R + Send + Sync + 'static,
    R: Future<Output = crate::Result<Response<Body>>> + Send + Sync + 'static,
  {
    self.add(path, vec![Method::TRACE], handler)
  }

  pub fn connect<P, H, R>(self, path: P, handler: H) -> Self
  where
    P: Into<String>,
    H: Fn(Request<Body>) -> R + Send + Sync + 'static,
    R: Future<Output = crate::Result<Response<Body>>> + Send + Sync + 'static,
  {
    self.add(path, vec![Method::CONNECT], handler)
  }

  pub fn patch<P, H, R>(self, path: P, handler: H) -> Self
  where
    P: Into<String>,
    H: Fn(Request<Body>) -> R + Send + Sync + 'static,
    R: Future<Output = crate::Result<Response<Body>>> + Send + Sync + 'static,
  {
    self.add(path, vec![Method::PATCH], handler)
  }

  pub fn options<P, H, R>(self, path: P, handler: H) -> Self
  where
    P: Into<String>,
    H: Fn(Request<Body>) -> R + Send + Sync + 'static,
    R: Future<Output = crate::Result<Response<Body>>> + Send + Sync + 'static,
  {
    self.add(path, vec![Method::OPTIONS], handler)
  }

  pub fn all<H, R>(self, handler: H) -> Self
  where
    H: Fn(Request<Body>) -> R + Send + Sync + 'static,
    R: Future<Output = crate::Result<Response<Body>>> + Send + Sync + 'static,
  {
    self.add("*", Vec::new(), handler)
  }

  pub fn add<P, H, R>(self, path: P, methods: Vec<Method>, handler: H) -> Self
  where
    P: Into<String>,
    H: Fn(Request<Body>) -> R + Send + Sync + 'static,
    R: Future<Output = crate::Result<Response<Body>>> + Send + Sync + 'static,
  {
    self.and_then(move |mut inner| {
      let route = Route::with_normal(path, methods, handler)?;
      inner.routes.push(route);
      crate::Result::Ok(inner)
    })
  }

  pub fn router<P>(self, path: P, router: &'static Router) -> Self
  where
    P: Into<String>,
  {
    self.and_then(move |mut inner| {
      let route = Route::with_router(path, router)?;
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
