use crate::data_map::SharedDataMap;
use crate::types::{RequestContext, RequestMeta, RouteParams};
use hyper::Request;
use std::net::SocketAddr;

/// A extension trait which extends the [`hyper::Request`](https://docs.rs/hyper/0.14.4/hyper/struct.Request.html) and [`http::Parts`](https://docs.rs/http/0.2.4/http/request/struct.Parts.html) types with some helpful methods.
pub trait RequestExt {
    /// It returns the route parameters as [RouteParams](../struct.RouteParams.html) type with the name of the parameter specified in the path as their respective keys.
    ///
    /// # Examples
    ///
    /// ```
    /// use routerify::{Router, RouteParams};
    /// use routerify::ext::RequestExt;
    /// use hyper::{Response, Body};
    /// # use std::convert::Infallible;
    ///
    /// # fn run() -> Router<Body, Infallible> {
    /// let router = Router::builder()
    ///     .get("/users/:userName/books/:bookName", |req| async move {
    ///         let params: &RouteParams = req.params();
    ///         let user_name = params.get("userName").unwrap();
    ///         let book_name = params.get("bookName").unwrap();
    ///
    ///         Ok(Response::new(Body::from(format!("Username: {}, Book Name: {}", user_name, book_name))))
    ///      })
    ///      .build()
    ///      .unwrap();
    /// # router
    /// # }
    /// # run();
    /// ```
    fn params(&self) -> &RouteParams;

    /// It returns the route parameter value by the name of the parameter specified in the path.
    ///
    /// # Examples
    ///
    /// ```
    /// use routerify::{Router, RouteParams};
    /// use routerify::ext::RequestExt;
    /// use hyper::{Response, Body};
    /// # use std::convert::Infallible;
    ///
    /// # fn run() -> Router<Body, Infallible> {
    /// let router = Router::builder()
    ///     .get("/users/:userName/books/:bookName", |req| async move {
    ///         let user_name = req.param("userName").unwrap();
    ///         let book_name = req.param("bookName").unwrap();
    ///
    ///         Ok(Response::new(Body::from(format!("Username: {}, Book Name: {}", user_name, book_name))))
    ///      })
    ///      .build()
    ///      .unwrap();
    /// # router
    /// # }
    /// # run();
    /// ```
    fn param<P: Into<String>>(&self, param_name: P) -> Option<&String>;

    /// It returns the remote address of the incoming request.
    ///
    /// # Examples
    ///
    /// ```
    /// use routerify::{Router, RouteParams};
    /// use routerify::ext::RequestExt;
    /// use hyper::{Response, Body};
    /// # use std::convert::Infallible;
    ///
    /// # fn run() -> Router<Body, Infallible> {
    /// let router = Router::builder()
    ///     .get("/hello", |req| async move {
    ///         let remote_addr = req.remote_addr();
    ///
    ///         Ok(Response::new(Body::from(format!("Hello from : {}", remote_addr))))
    ///      })
    ///      .build()
    ///      .unwrap();
    /// # router
    /// # }
    /// # run();
    /// ```
    fn remote_addr(&self) -> SocketAddr;

    /// Access data which was shared by the [`RouterBuilder`](../struct.RouterBuilder.html) method
    /// [`data`](../struct.RouterBuilder.html#method.data).
    ///
    /// Please refer to the [Data and State Sharing](../index.html#data-and-state-sharing) for more info.
    fn data<T: Send + Sync + 'static>(&self) -> Option<&T>;

    /// Access data in the request context.
    fn context<T: Send + Sync + Clone + 'static>(&self) -> Option<T>;

    /// Put data into the request context.
    ///
    /// # Examples
    ///
    /// ```
    /// use routerify::{Router, RouteParams, Middleware};
    /// use routerify::ext::RequestExt;
    /// use hyper::{Response, Request, Body};
    /// # use std::convert::Infallible;
    ///
    /// # fn run() -> Router<Body, Infallible> {
    /// let router = Router::builder()
    ///     .middleware(Middleware::pre(|req: Request<Body>| async move {
    ///         req.set_context("example".to_string());
    ///
    ///         Ok(req)
    ///     }))
    ///     .get("/hello", |req| async move {
    ///         let text = req.context::<String>().unwrap();
    ///
    ///         Ok(Response::new(Body::from(format!("Hello from : {}", text))))
    ///      })
    ///      .build()
    ///      .unwrap();
    /// # router
    /// # }
    /// # run();
    /// ```
    fn set_context<T: Send + Sync + Clone + 'static>(&self, val: T);
}

fn params(ext: &http::Extensions) -> &RouteParams {
    ext.get::<RequestMeta>()
        .and_then(|meta| meta.route_params())
        .expect("Routerify: No RouteParams added while processing request")
}

fn param<P: Into<String>>(ext: &http::Extensions, param_name: P) -> Option<&String> {
    params(ext).get(&param_name.into())
}

fn remote_addr(ext: &http::Extensions) -> SocketAddr {
    ext.get::<RequestMeta>()
        .and_then(|meta| meta.remote_addr())
        .copied()
        .expect("Routerify: No remote address added while processing request")
}

fn data<T: Send + Sync + 'static>(ext: &http::Extensions) -> Option<&T> {
    let shared_data_maps = ext.get::<Vec<SharedDataMap>>();

    if let Some(shared_data_maps) = shared_data_maps {
        for shared_data_map in shared_data_maps.iter() {
            if let Some(data) = shared_data_map.inner.get::<T>() {
                return Some(data);
            }
        }
    }

    None
}

fn context<T: Send + Sync + Clone + 'static>(ext: &http::Extensions) -> Option<T> {
    let ctx = ext.get::<RequestContext>().expect("Context must be present");
    ctx.get::<T>()
}

fn set_context<T: Send + Sync + Clone + 'static>(ext: &http::Extensions, val: T) {
    let ctx = ext.get::<RequestContext>().expect("Context must be present");
    ctx.set(val)
}

impl RequestExt for Request<hyper::body::Incoming> {
    fn params(&self) -> &RouteParams {
        params(self.extensions())
    }

    fn param<P: Into<String>>(&self, param_name: P) -> Option<&String> {
        param(self.extensions(), param_name)
    }

    fn remote_addr(&self) -> SocketAddr {
        remote_addr(self.extensions())
    }

    fn data<T: Send + Sync + 'static>(&self) -> Option<&T> {
        data(self.extensions())
    }

    fn context<T: Send + Sync + Clone + 'static>(&self) -> Option<T> {
        context(self.extensions())
    }

    fn set_context<T: Send + Sync + Clone + 'static>(&self, val: T) {
        set_context(self.extensions(), val)
    }
}

impl RequestExt for http::request::Parts {
    fn params(&self) -> &RouteParams {
        params(&self.extensions)
    }

    fn param<P: Into<String>>(&self, param_name: P) -> Option<&String> {
        param(&self.extensions, param_name)
    }

    fn remote_addr(&self) -> SocketAddr {
        remote_addr(&self.extensions)
    }

    fn data<T: Send + Sync + 'static>(&self) -> Option<&T> {
        data(&self.extensions)
    }

    fn context<T: Send + Sync + Clone + 'static>(&self) -> Option<T> {
        context(&self.extensions)
    }

    fn set_context<T: Send + Sync + Clone + 'static>(&self, val: T) {
        set_context(&self.extensions, val)
    }
}
