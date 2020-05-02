use crate::constants;
use crate::middleware::{Middleware, PostMiddleware, PreMiddleware};
use crate::route::Route;
use crate::router::ErrHandler;
use crate::router::Router;
use hyper::{body::HttpBody, Method, Request, Response};
use std::future::Future;

/// Builder for the [Router](./struct.Router.html) type.
///
/// This `RouterBuilder<B, E>` type accepts two type parameters: `B` and `E`.
///
/// * The `B` represents the response body type which will be used by route handlers and the middlewares and this body type must implement
///   the [HttpBody](https://docs.rs/hyper/0.13.5/hyper/body/trait.HttpBody.html) trait. For an instance, `B` could be [hyper::Body](https://docs.rs/hyper/0.13.5/hyper/body/struct.Body.html)
///   type.
/// * The `E` represents any error type which will be used by route handlers and the middlewares. This error type must implement the [std::error::Error](https://doc.rust-lang.org/std/error/trait.Error.html).
///
/// # Examples
///
/// ```no_run
/// use routerify::{Router, Middleware};
/// use hyper::{Response, Request, Body};
///
/// async fn home_handler(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
///     Ok(Response::new(Body::from("home")))
/// }
///
/// async fn upload_handler(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
///     Ok(Response::new(Body::from("upload")))
/// }
///
/// async fn some_pre_middleware_handler(req: Request<Body>) -> Result<Request<Body>, hyper::Error> {
///     Ok(req)
/// }
///
/// # fn run() -> Router<Body, hyper::Error> {
/// // Use Router::builder() method to create a new RouterBuilder instance.
/// // We will use hyper::Body as response body type and hyper::Error as error type.
/// let router: Router<Body, hyper::Error> = Router::builder()
///     .get("/", home_handler)
///     .post("/upload", upload_handler)
///     .middleware(Middleware::pre(some_pre_middleware_handler))
///     .build()
///     .unwrap();
/// # router
/// # }
/// # run();
/// ```
pub struct RouterBuilder<B, E> {
    inner: crate::Result<BuilderInner<B, E>>,
}

struct BuilderInner<B, E> {
    pre_middlewares: Vec<PreMiddleware<E>>,
    routes: Vec<Route<B, E>>,
    post_middlewares: Vec<PostMiddleware<B, E>>,
    err_handler: Option<ErrHandler<B>>,
}

impl<B: HttpBody + Send + Sync + Unpin + 'static, E: std::error::Error + Send + Sync + Unpin + 'static>
    RouterBuilder<B, E>
{
    /// Creates a new `RouterBuilder` instance with default options.
    pub fn new() -> RouterBuilder<B, E> {
        RouterBuilder::default()
    }

    /// Creates a new [Router](./struct.Router.html) instance from the added configuration.
    pub fn build(self) -> crate::Result<Router<B, E>> {
        self.inner.map(|inner| Router {
            pre_middlewares: inner.pre_middlewares,
            routes: inner.routes,
            post_middlewares: inner.post_middlewares,
            err_handler: inner.err_handler,
        })
    }

    fn and_then<F>(self, func: F) -> Self
    where
        F: FnOnce(BuilderInner<B, E>) -> crate::Result<BuilderInner<B, E>>,
    {
        RouterBuilder {
            inner: self.inner.and_then(func),
        }
    }
}

impl<B: HttpBody + Send + Sync + Unpin + 'static, E: std::error::Error + Send + Sync + Unpin + 'static>
    RouterBuilder<B, E>
{
    /// Adds a new route with `GET` method and the handler at the specified path.
    ///
    /// # Examples
    ///
    /// ```
    /// use routerify::Router;
    /// use hyper::{Response, Request, Body};
    ///
    /// async fn home_handler(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    ///     Ok(Response::new(Body::from("home")))
    /// }
    ///
    /// # fn run() -> Router<Body, hyper::Error> {
    /// let router = Router::builder()
    ///     .get("/", home_handler)
    ///     .build()
    ///     .unwrap();
    /// # router
    /// # }
    /// # run();
    /// ```
    pub fn get<P, H, R>(self, path: P, handler: H) -> Self
    where
        P: Into<String>,
        H: FnMut(Request<hyper::Body>) -> R + Send + Sync + 'static,
        R: Future<Output = Result<Response<B>, E>> + Send + 'static,
    {
        self.add(path, vec![Method::GET], handler)
    }

    /// Adds a new route with `GET` and `HEAD` methods and the handler at the specified path.
    ///
    /// # Examples
    ///
    /// ```
    /// use routerify::Router;
    /// use hyper::{Response, Request, Body};
    ///
    /// async fn home_handler(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    ///     Ok(Response::new(Body::from("home")))
    /// }
    ///
    /// # fn run() -> Router<Body, hyper::Error> {
    /// let router = Router::builder()
    ///     .get_or_head("/", home_handler)
    ///     .build()
    ///     .unwrap();
    /// # router
    /// # }
    /// # run();
    /// ```
    pub fn get_or_head<P, H, R>(self, path: P, handler: H) -> Self
    where
        P: Into<String>,
        H: FnMut(Request<hyper::Body>) -> R + Send + Sync + 'static,
        R: Future<Output = Result<Response<B>, E>> + Send + 'static,
    {
        self.add(path, vec![Method::GET, Method::HEAD], handler)
    }

    /// Adds a new route with `POST` method and the handler at the specified path.
    ///
    /// # Examples
    ///
    /// ```
    /// use routerify::Router;
    /// use hyper::{Response, Request, Body};
    ///
    /// async fn file_upload_handler(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    ///     Ok(Response::new(Body::from("File uploader")))
    /// }
    ///
    /// # fn run() -> Router<Body, hyper::Error> {
    /// let router = Router::builder()
    ///     .post("/upload", file_upload_handler)
    ///     .build()
    ///     .unwrap();
    /// # router
    /// # }
    /// # run();
    /// ```
    pub fn post<P, H, R>(self, path: P, handler: H) -> Self
    where
        P: Into<String>,
        H: FnMut(Request<hyper::Body>) -> R + Send + Sync + 'static,
        R: Future<Output = Result<Response<B>, E>> + Send + 'static,
    {
        self.add(path, vec![Method::POST], handler)
    }

    /// Adds a new route with `PUT` method and the handler at the specified path.
    ///
    /// # Examples
    ///
    /// ```
    /// use routerify::Router;
    /// use hyper::{Response, Request, Body};
    ///
    /// async fn file_upload_handler(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    ///     Ok(Response::new(Body::from("File uploader")))
    /// }
    ///
    /// # fn run() -> Router<Body, hyper::Error> {
    /// let router = Router::builder()
    ///     .put("/upload", file_upload_handler)
    ///     .build()
    ///     .unwrap();
    /// # router
    /// # }
    /// # run();
    /// ```
    pub fn put<P, H, R>(self, path: P, handler: H) -> Self
    where
        P: Into<String>,
        H: FnMut(Request<hyper::Body>) -> R + Send + Sync + 'static,
        R: Future<Output = Result<Response<B>, E>> + Send + 'static,
    {
        self.add(path, vec![Method::PUT], handler)
    }

    /// Adds a new route with `DELETE` method and the handler at the specified path.
    ///
    /// # Examples
    ///
    /// ```
    /// use routerify::Router;
    /// use hyper::{Response, Request, Body};
    ///
    /// async fn delete_file_handler(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    ///     Ok(Response::new(Body::from("Delete file")))
    /// }
    ///
    /// # fn run() -> Router<Body, hyper::Error> {
    /// let router = Router::builder()
    ///     .delete("/delete-file", delete_file_handler)
    ///     .build()
    ///     .unwrap();
    /// # router
    /// # }
    /// # run();
    /// ```
    pub fn delete<P, H, R>(self, path: P, handler: H) -> Self
    where
        P: Into<String>,
        H: FnMut(Request<hyper::Body>) -> R + Send + Sync + 'static,
        R: Future<Output = Result<Response<B>, E>> + Send + 'static,
    {
        self.add(path, vec![Method::DELETE], handler)
    }

    /// Adds a new route with `HEAD` method and the handler at the specified path.
    ///
    /// # Examples
    ///
    /// ```
    /// use routerify::Router;
    /// use hyper::{Response, Request, Body};
    ///
    /// async fn a_head_handler(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    ///     Ok(Response::new(Body::empty()))
    /// }
    ///
    /// # fn run() -> Router<Body, hyper::Error> {
    /// let router = Router::builder()
    ///     .head("/fetch-data", a_head_handler)
    ///     .build()
    ///     .unwrap();
    /// # router
    /// # }
    /// # run();
    /// ```
    pub fn head<P, H, R>(self, path: P, handler: H) -> Self
    where
        P: Into<String>,
        H: FnMut(Request<hyper::Body>) -> R + Send + Sync + 'static,
        R: Future<Output = Result<Response<B>, E>> + Send + 'static,
    {
        self.add(path, vec![Method::HEAD], handler)
    }

    /// Adds a new route with `TRACE` method and the handler at the specified path.
    ///
    /// # Examples
    ///
    /// ```
    /// use routerify::Router;
    /// use hyper::{Response, Request, Body};
    ///
    /// async fn trace_handler(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    ///     Ok(Response::new(Body::empty()))
    /// }
    ///
    /// # fn run() -> Router<Body, hyper::Error> {
    /// let router = Router::builder()
    ///     .trace("/abc", trace_handler)
    ///     .build()
    ///     .unwrap();
    /// # router
    /// # }
    /// # run();
    /// ```
    pub fn trace<P, H, R>(self, path: P, handler: H) -> Self
    where
        P: Into<String>,
        H: FnMut(Request<hyper::Body>) -> R + Send + Sync + 'static,
        R: Future<Output = Result<Response<B>, E>> + Send + 'static,
    {
        self.add(path, vec![Method::TRACE], handler)
    }

    /// Adds a new route with `CONNECT` method and the handler at the specified path.
    ///
    /// # Examples
    ///
    /// ```
    /// use routerify::Router;
    /// use hyper::{Response, Request, Body};
    ///
    /// async fn connect_handler(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    ///     Ok(Response::new(Body::empty()))
    /// }
    ///
    /// # fn run() -> Router<Body, hyper::Error> {
    /// let router = Router::builder()
    ///     .connect("/abc", connect_handler)
    ///     .build()
    ///     .unwrap();
    /// # router
    /// # }
    /// # run();
    /// ```
    pub fn connect<P, H, R>(self, path: P, handler: H) -> Self
    where
        P: Into<String>,
        H: FnMut(Request<hyper::Body>) -> R + Send + Sync + 'static,
        R: Future<Output = Result<Response<B>, E>> + Send + 'static,
    {
        self.add(path, vec![Method::CONNECT], handler)
    }

    /// Adds a new route with `PATCH` method and the handler at the specified path.
    ///
    /// # Examples
    ///
    /// ```
    /// use routerify::Router;
    /// use hyper::{Response, Request, Body};
    ///
    /// async fn update_data_handler(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    ///     Ok(Response::new(Body::from("Data updater")))
    /// }
    ///
    /// # fn run() -> Router<Body, hyper::Error> {
    /// let router = Router::builder()
    ///     .patch("/update-data", update_data_handler)
    ///     .build()
    ///     .unwrap();
    /// # router
    /// # }
    /// # run();
    /// ```
    pub fn patch<P, H, R>(self, path: P, handler: H) -> Self
    where
        P: Into<String>,
        H: FnMut(Request<hyper::Body>) -> R + Send + Sync + 'static,
        R: Future<Output = Result<Response<B>, E>> + Send + 'static,
    {
        self.add(path, vec![Method::PATCH], handler)
    }

    /// Adds a new route with `OPTIONS` method and the handler at the specified path.
    ///
    /// # Examples
    ///
    /// ```
    /// use routerify::Router;
    /// use hyper::{Response, Request, Body};
    ///
    /// async fn options_handler(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    ///     Ok(Response::new(Body::empty()))
    /// }
    ///
    /// # fn run() -> Router<Body, hyper::Error> {
    /// let router = Router::builder()
    ///     .options("/abc", options_handler)
    ///     .build()
    ///     .unwrap();
    /// # router
    /// # }
    /// # run();
    /// ```
    pub fn options<P, H, R>(self, path: P, handler: H) -> Self
    where
        P: Into<String>,
        H: FnMut(Request<hyper::Body>) -> R + Send + Sync + 'static,
        R: Future<Output = Result<Response<B>, E>> + Send + 'static,
    {
        self.add(path, vec![Method::OPTIONS], handler)
    }

    /// Adds a new route with any method type and the handler at the `/*` path. It will accept any kind of request. It can be used to send
    /// response for any non-existing routes i.e. for `404` pages.
    ///
    /// # Examples
    ///
    /// ```
    /// use routerify::Router;
    /// use hyper::{Response, Request, Body, StatusCode};
    ///
    /// async fn home_handler(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    ///     Ok(Response::new(Body::from("home")))
    /// }
    ///
    /// async fn handler_404(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    ///     Ok(
    ///         Response::builder()
    ///          .status(StatusCode::NOT_FOUND)
    ///          .body(Body::from("NOT FOUND"))
    ///          .unwrap()
    ///      )
    /// }
    ///
    /// # fn run() -> Router<Body, hyper::Error> {
    /// let router = Router::builder()
    ///     .get("/home", home_handler)
    ///     .any(handler_404)
    ///     .build()
    ///     .unwrap();
    /// # router
    /// # }
    /// # run();
    /// ```
    pub fn any<H, R>(self, handler: H) -> Self
    where
        H: FnMut(Request<hyper::Body>) -> R + Send + Sync + 'static,
        R: Future<Output = Result<Response<B>, E>> + Send + 'static,
    {
        self.add("/*", constants::ALL_POSSIBLE_HTTP_METHODS.to_vec(), handler)
    }

    /// Adds a new route with the specified method(s) and the handler at the specified path. It can be used to define routes with multiple method types.
    ///
    /// # Examples
    ///
    /// ```
    /// use routerify::Router;
    /// use hyper::{Response, Request, Body, StatusCode, Method};
    ///
    /// async fn cart_checkout_handler(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    ///     Ok(Response::new(Body::from("You shopping cart is being checking out")))
    /// }
    ///
    /// # fn run() -> Router<Body, hyper::Error> {
    /// let router = Router::builder()
    ///     .add("/checkout", vec![Method::GET, Method::POST], cart_checkout_handler)
    ///     .build()
    ///     .unwrap();
    /// # router
    /// # }
    /// # run();
    /// ```
    pub fn add<P, H, R>(self, path: P, methods: Vec<Method>, handler: H) -> Self
    where
        P: Into<String>,
        H: FnMut(Request<hyper::Body>) -> R + Send + Sync + 'static,
        R: Future<Output = Result<Response<B>, E>> + Send + 'static,
    {
        self.and_then(move |mut inner| {
            let route = Route::new(path, methods, handler)?;
            inner.routes.push(route);
            crate::Result::Ok(inner)
        })
    }

    /// It mounts a router onto another router. It can be very useful when you want to write modular routing logic.
    ///
    /// # Examples
    ///
    /// ```
    /// use routerify::Router;
    /// use hyper::{Response, Request, Body};
    ///
    /// mod api {
    ///     use routerify::Router;
    ///     use hyper::{Response, Request, Body};
    ///
    ///     pub fn router() -> Router<Body, hyper::Error> {
    ///         Router::builder()
    ///          .get("/users", |req| async move { Ok(Response::new(Body::from("User list"))) })
    ///          .get("/books", |req| async move { Ok(Response::new(Body::from("Book list"))) })
    ///          .build()
    ///          .unwrap()
    ///     }
    /// }
    ///
    /// # fn run() -> Router<Body, hyper::Error> {
    /// let router = Router::builder()
    ///     // Now, mount the api router at `/api` path.
    ///     .scope("/api", api::router())
    ///     .build()
    ///     .unwrap();
    /// # router
    /// # }
    /// # run();
    /// ```
    ///
    /// Now, the app can handle requests on: `/api/users` and `/api/books` paths.
    pub fn scope<P>(self, path: P, mut router: Router<B, E>) -> Self
    where
        P: Into<String>,
    {
        let mut path = path.into();

        if path.ends_with("/") {
            path = (&path[..path.len() - 1]).to_string();
        }

        let mut builder = self;

        for pre_middleware in router.pre_middlewares.iter_mut() {
            let new_pre_middleware = PreMiddleware::new_with_boxed_handler(
                format!("{}{}", path.as_str(), pre_middleware.path.as_str()),
                pre_middleware
                    .handler
                    .take()
                    .expect("No handler found in one of the pre-middlewares"),
            );
            builder = builder.and_then(move |mut inner| {
                inner.pre_middlewares.push(new_pre_middleware?);
                crate::Result::Ok(inner)
            });
        }

        for route in router.routes.iter_mut() {
            let new_route = Route::new_with_boxed_handler(
                format!("{}{}", path.as_str(), route.path.as_str()),
                route.methods.clone(),
                route.handler.take().expect("No handler found in one of the routes"),
            );
            builder = builder.and_then(move |mut inner| {
                inner.routes.push(new_route?);
                crate::Result::Ok(inner)
            });
        }

        for post_middleware in router.post_middlewares.iter_mut() {
            let new_post_middleware = PostMiddleware::new_with_boxed_handler(
                format!("{}{}", path.as_str(), post_middleware.path.as_str()),
                post_middleware
                    .handler
                    .take()
                    .expect("No handler found in one of the post-middlewares"),
            );
            builder = builder.and_then(move |mut inner| {
                inner.post_middlewares.push(new_post_middleware?);
                crate::Result::Ok(inner)
            });
        }

        builder
    }
}

impl<B: HttpBody + Send + Sync + Unpin + 'static, E: std::error::Error + Send + Sync + Unpin + 'static>
    RouterBuilder<B, E>
{
    /// Adds a single middleware. A pre middleware can be created by [`Middleware::pre`](./enum.Middleware.html#method.pre) method and a post
    /// middleware can be created by [`Middleware::post`](./enum.Middleware.html#method.post) method.
    ///
    /// # Examples
    ///
    /// ```
    /// use routerify::{Router, Middleware};
    /// use hyper::{Response, Request, Body};
    /// use std::convert::Infallible;
    ///
    /// # fn run() -> Router<Body, Infallible> {
    /// let router = Router::builder()
    ///      // Create and attach a pre middleware.
    ///      .middleware(Middleware::pre(|req| async move { /* Do some operations */ Ok(req) }))
    ///      // Create and attach a post middleware.
    ///      .middleware(Middleware::post(|res| async move { /* Do some operations */ Ok(res) }))
    ///      .build()
    ///      .unwrap();
    /// # router
    /// # }
    /// # run();
    /// ```
    pub fn middleware(self, m: Middleware<B, E>) -> Self {
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

    /// Adds a handler to handle any error raised by the routes or any middlewares. Please refer to [Error Handling](./index.html#error-handling) section
    /// for more info.
    pub fn err_handler<H, R>(self, mut handler: H) -> Self
    where
        H: FnMut(crate::Error) -> R + Send + Sync + 'static,
        R: Future<Output = Response<B>> + Send + 'static,
    {
        let handler: ErrHandler<B> = Box::new(move |err: crate::Error| Box::new(handler(err)));

        self.and_then(move |mut inner| {
            inner.err_handler = Some(handler);
            crate::Result::Ok(inner)
        })
    }
}

impl<B: HttpBody + Send + Sync + Unpin + 'static, E: std::error::Error + Send + Sync + Unpin + 'static> Default
    for RouterBuilder<B, E>
{
    fn default() -> RouterBuilder<B, E> {
        RouterBuilder {
            inner: Ok(BuilderInner {
                pre_middlewares: Vec::new(),
                routes: Vec::new(),
                post_middlewares: Vec::new(),
                err_handler: None,
            }),
        }
    }
}
