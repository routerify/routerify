//! The `Routerify` provides routing logic to the existing Rust HTTP library [hyper.rs](https://hyper.rs/).
//!
//! There are lot of web server framework for Rust applications out there and [hyper.rs](https://hyper.rs/) being comparably very fast and ready for production use
//! is one of them and it provides only low level API. It doesn't provide any complex routing feature. So, `Routerify` extends the [hyper.rs](https://hyper.rs/) library
//! by providing that missing feature.
//!
//! The `Routerify` offers the following features:
//!
//! - 📡 Allows to define complex routing logic.
//!
//! - 🔨 Provides middleware support.
//!
//! - 🌀 Supports Route Parameters.
//!
//! - 🚀 Fast and ready for production use.
//!
//! - 🍺 It supports any body type as long as it implements the [HttpBody](https://docs.rs/hyper/0.13.5/hyper/body/trait.HttpBody.html) trait.
//!
//! - ❗ Provides a flexible [error handling](./index.html#error-handling) strategy.
//!
//! - 🍗 Extensive [examples](https://github.com/routerify/routerify/tree/master/examples) and well documented.
//!
//! ## Basic Example
//!
//! A simple example using `Routerify` with [hyper.rs](https://hyper.rs/) would look like the following:
//!
//! ```no_run
//! use hyper::{Body, Request, Response, Server};
//! use routerify::prelude::*;
//! use routerify::{Middleware, Router, RouterService};
//! use std::{convert::Infallible, net::SocketAddr};
//!
//! // A handler for "/" page.
//! async fn home(_: Request<Body>) -> Result<Response<Body>, Infallible> {
//!     Ok(Response::new(Body::from("Home page")))
//! }
//!
//! // A handler for "/about" page.
//! async fn about(_: Request<Body>) -> Result<Response<Body>, Infallible> {
//!     Ok(Response::new(Body::from("About page")))
//! }
//!
//! // A middleware which logs an http request.
//! async fn logger(req: Request<Body>) -> Result<Request<Body>, Infallible> {
//!     println!("{} {} {}", req.remote_addr(), req.method(), req.uri().path());
//!     Ok(req)
//! }
//!
//! // Create a `Router<Body, Infallible>` for body type `hyper::Body` and for handler error type `Infallible`.
//! fn router() -> Router<Body, Infallible> {
//!     // Create a router and specify the logger middleware and the handlers.
//!     // Here, "Middleware::pre" means we're adding a pre-middleware which will accept
//!     // a request and transforms it to a new request.
//!     Router::builder()
//!         .middleware(Middleware::pre(logger))
//!         .get("/", home)
//!         .get("/about", about)
//!         .build()
//!         .unwrap()
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let router = router();
//!
//!     // Create a Service from the router above to handle incoming requests.
//!     let service = RouterService::new(router);
//!
//!     // The address on which the server will be listening.
//!     let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
//!
//!     // Create a server by passing the created service to `.serve` method.
//!     let server = Server::bind(&addr).serve(service);
//!
//!     println!("App is running on: {}", addr);
//!     if let Err(err) = server.await {
//!         eprintln!("Server error: {}", err);
//!    }
//! }
//! ```
//! ## Routing
//!
//! ### Route Handlers
//!
//! A handler could be a function or a closure to handle a request at a specified route path.
//!
//! Here is a handler with a function:
//!
//! ```
//! use routerify::Router;
//! use hyper::{Response, Request, Body};
//! use std::convert::Infallible;
//!
//! // A handler for "/about" page.
//! async fn about_handler(_: Request<Body>) -> Result<Response<Body>, Infallible> {
//!     Ok(Response::new(Body::from("About page")))
//! }
//!
//! # fn run() -> Router<Body, Infallible> {
//! let router = Router::builder()
//!     .get("/about", about_handler)
//!     .build()
//!     .unwrap();
//! # router
//! # }
//! # run();
//! ```
//!
//! Here is a handler with closure function:
//!
//! ```
//! use routerify::Router;
//! use hyper::{Response, Body};
//! # use std::convert::Infallible;
//!
//! # fn run() -> Router<Body, Infallible> {
//! let router = Router::builder()
//!     .get("/about", |req| async move { Ok(Response::new(Body::from("About page"))) })
//!     .build()
//!     .unwrap();
//! # router
//! # }
//! # run();
//! ```
//!
//! ### Route Paths
//!
//! Route paths, in combination with a request method, define the endpoints at which requests can be made.
//! Route paths can be strings or strings with glob pattern `*`.
//!
//!
//! Here are some examples:
//!
//! This route path will match with exactly "/about":
//!
//! ```
//! use routerify::Router;
//! use hyper::{Response, Body};
//! # use std::convert::Infallible;
//!
//! # fn run() -> Router<Body, Infallible> {
//! let router = Router::builder()
//!     .get("/about", |req| async move { Ok(Response::new(Body::from("About page"))) })
//!     .build()
//!     .unwrap();
//! # router
//! # }
//! # run();
//! ```
//!
//! A route path using the glob `*` pattern:
//!
//! ```
//! use routerify::Router;
//! use hyper::{Response, Body};
//! # use std::convert::Infallible;
//!
//! # fn run() -> Router<Body, Infallible> {
//! let router = Router::builder()
//!     .get("/users/*", |req| async move { Ok(Response::new(Body::from("It will match /users/, /users/any_path"))) })
//!     .build()
//!     .unwrap();
//! # router
//! # }
//! # run();
//! ```
//!
//! #### Handle 404 Pages
//!
//! Here is an example to handle 404 pages.
//!
//! ```
//! use routerify::Router;
//! use hyper::{Response, Body, StatusCode};
//! # use std::convert::Infallible;
//!
//! # fn run() -> Router<Body, Infallible> {
//! let router = Router::builder()
//!     .get("/users", |req| async move { Ok(Response::new(Body::from("User List"))) })
//!     // It fallbacks to the following route for any non-existent routes.
//!     .any(|_req| async move {
//!         Ok(
//!             Response::builder()
//!             .status(StatusCode::NOT_FOUND)
//!             .body(Body::from("NOT FOUND"))
//!             .unwrap()
//!         )
//!     })
//!     .build()
//!     .unwrap();
//! # router
//! # }
//! # run();
//! ```
//!
//! ### Route Parameters
//!
//! Route parameters are named URL segments that are used to capture the values specified at their position in the URL.
//! The captured values can be accessed by `req.params` and `re.param` methods using the name of the route parameter specified in the path.
//!
//! ```txt
//! Route path: /users/:userName/books/:bookName
//! Request URL: http://localhost:3000/users/alice/books/HarryPotter
//! req.params() returns a hashmap: { "userName": "alice", "bookName": "HarryPotter" }
//! ```
//!
//! To define routes with route parameters, simply specify the route parameters in the path of the route as shown below.
//!
//! ```
//! use routerify::Router;
//! // Add routerify prelude traits.
//! use routerify::prelude::*;
//! use hyper::{Response, Body};
//! # use std::convert::Infallible;
//!
//! # fn run() -> Router<Body, Infallible> {
//! let router = Router::builder()
//!     .get("/users/:userName/books/:bookName", |req| async move {
//!         let user_name = req.param("userName").unwrap();
//!         let book_name = req.param("bookName").unwrap();
//!
//!         Ok(Response::new(Body::from(format!("Username: {}, Book Name: {}", user_name, book_name))))
//!      })
//!      .build()
//!      .unwrap();
//! # router
//! # }
//! # run();
//! ```
//!
//! ### Scoping/Mounting Router
//!
//! The `routerify::Router` is a modular, lightweight and mountable router component. A router can be scoped in or mount to a
//! different router.
//!
//! Here is a simple example which creates a Router and it mounts that router at `/api` path with `.scope()` method:
//!
//! ```
//! use routerify::Router;
//! use routerify::prelude::*;
//! use hyper::{Response, Body};
//! use std::convert::Infallible;
//!
//! fn api_router() -> Router<Body, Infallible> {
//!     Router::builder()
//!         .get("/books", |req| async move { Ok(Response::new(Body::from("List of books"))) })
//!         .get("/books/:bookId", |req| async move {
//!             Ok(Response::new(Body::from(format!("Show book: {}", req.param("bookId").unwrap()))))
//!          })
//!         .build()
//!         .unwrap()
//! }
//!
//! # fn run() -> Router<Body, Infallible> {
//! let router = Router::builder()
//!      // Mounts the API router at "/api" path .
//!      .scope("/api", api_router())
//!      .build()
//!      .unwrap();
//! # router
//! # }
//! # run();
//! ```
//! Now, the app can handle requests to `/api/books` as well as to `/api/books/:bookId`.
//!
//! ## Middleware
//!
//! The `Routerify` also supports Middleware functionality. If you are unfamiliar with Middleware, in short, here a middlewar is a function (or could be a closure
//! function) which access the `req` and `res` object and does some changes to them and passes the transformed request and response object to the other middlewares and the actual route handler
//! to process it.
//!
//! A Middleware function can do the following tasks:
//!
//! - Execute any code.
//! - Transform the request and the response object.
//!
//! Here, the `Routerify` categorizes the middlewares into two different types:
//!
//! ### Pre Middleware
//!
//! The pre Middlewares will be executed before any route handlers and it will access the `req` object and it can also do some changes to the request object
//! if required.
//!
//! Here is an example of a pre middleware:
//!
//! ```
//! use routerify::{Router, Middleware};
//! use hyper::{Request, Body};
//! use std::convert::Infallible;
//!
//! // The handler for a pre middleware.
//! // It accepts a `req` and it transforms the `req` and passes it to the next middlewares.
//! async fn my_pre_middleware_handler(req: Request<Body>) -> Result<Request<Body>, Infallible> {
//!     // Do some changes if required.
//!     let transformed_req = req;
//!
//!     // Then return the transformed request object to be consumed by the other middlewares
//!     // and the route handlers.
//!     Ok(transformed_req)
//! }
//!
//! # fn run() -> Router<Body, Infallible> {
//! let router = Router::builder()
//!      // Create a pre middleware instance by `Middleware::pre` method
//!      // and attach it.
//!      .middleware(Middleware::pre(my_pre_middleware_handler))
//!      // A middleware can also be attached on a specific path as shown below.
//!      .middleware(Middleware::pre_with_path("/my-path/log", my_pre_middleware_handler).unwrap())
//!      .build()
//!      .unwrap();
//! # router
//! # }
//! # run();
//! ```
//!
//! Here is a pre middleware which logs the incoming requests:
//!
//! ```
//! use routerify::{Router, Middleware};
//! use routerify::prelude::*;
//! use hyper::{Request, Body};
//! use std::convert::Infallible;
//!
//! async fn logger_middleware_handler(req: Request<Body>) -> Result<Request<Body>, Infallible> {
//!     println!("{} {} {}", req.remote_addr(), req.method(), req.uri().path());
//!     Ok(req)
//! }
//!
//! # fn run() -> Router<Body, Infallible> {
//! let router = Router::builder()
//!      .middleware(Middleware::pre(logger_middleware_handler))
//!      .build()
//!      .unwrap();
//! # router
//! # }
//! # run();
//! ```
//!
//! ### Post Middleware
//!
//! The post Middlewares will be executed after all the route handlers process the request and generates a response and it will access that response object
//! and it can also do some changes to the response if required.
//!
//! Here is an example of a post middleware:
//!
//! ```
//! use routerify::{Router, Middleware};
//! use hyper::{Response, Body};
//! use std::convert::Infallible;
//!
//! // The handler for a post middleware.
//! // It accepts a `res` and it transforms the `res` and passes it to the next middlewares.
//! async fn my_post_middleware_handler(res: Response<Body>) -> Result<Response<Body>, Infallible> {
//!     // Do some changes if required.
//!     let transformed_res = res;
//!
//!     // Then return the transformed response object to be consumed by the other middlewares
//!     // and the route handlers.
//!     Ok(transformed_res)
//! }
//!
//! # fn run() -> Router<Body, Infallible> {
//! let router = Router::builder()
//!      // Create a post middleware instance by `Middleware::post` method
//!      // and attach it.
//!      .middleware(Middleware::post(my_post_middleware_handler))
//!      // A middleware can also be attached on a specific path as shown below.
//!      .middleware(Middleware::post_with_path("/my-path/log", my_post_middleware_handler).unwrap())
//!      .build()
//!      .unwrap();
//! # router
//! # }
//! # run();
//! ```
//!
//! Here is a post middleware which adds a header to the response object:
//!
//! ```
//! use routerify::{Router, Middleware};
//! use routerify::prelude::*;
//! use hyper::{Response, Body, header::HeaderValue};
//! use std::convert::Infallible;
//!
//! async fn my_post_middleware_handler(mut res: Response<Body>) -> Result<Response<Body>, Infallible> {
//!     // Add a header to response object.
//!     res.headers_mut().insert("x-my-custom-header", HeaderValue::from_static("my-value"));
//!
//!     Ok(res)
//! }
//!
//! # fn run() -> Router<Body, Infallible> {
//! let router = Router::builder()
//!      .middleware(Middleware::post(my_post_middleware_handler))
//!      .build()
//!      .unwrap();
//! # router
//! # }
//! # run();
//! ```
//!
//! ### The built-in Middlewars
//!
//! Here is a list of some middlewares which are published in different crates:
//!
//! - [routerify-cors](https://github.com/routerify/routerify-cors): A post middleware which enable `CORS` to the rouets.
//! - [routerify-query](https://github.com/routerify/routerify-query): A pre middleware which parses the request query string.
//!
//! ## Error Handling
//!
//! Any route or middlewares could go wrong and throws error. The `Routerify` tries to add a default error handler in some cases. But, it also
//! allow to attach a custom error handler.
//!
//! Here is an example:
//!
//! ```
//! use routerify::{Router, Middleware};
//! use routerify::prelude::*;
//! use hyper::{Response, Body, StatusCode};
//!
//! // The error handler will accept the thrown error in routerify::Error type and
//! // it will have to generate a response based on the error.
//! async fn error_handler(err: routerify::Error) -> Response<Body> {
//!     Response::builder()
//!       .status(StatusCode::INTERNAL_SERVER_ERROR)
//!       .body(Body::from("Something went wrong"))
//!       .unwrap()
//! }
//!
//! # fn run() -> Router<Body, hyper::Error> {
//! let router = Router::builder()
//!      .get("/users", |req| async move { Ok(Response::new(Body::from("It might raise an error"))) })
//!      // Here attach the custom error handler defined above.
//!      .err_handler(error_handler)
//!      .build()
//!      .unwrap();
//! # router
//! # }
//! # run();
//! ```

pub use self::error::Error;
pub(crate) use self::error::{ErrorExt, ResultExt};
pub use self::middleware::{Middleware, PostMiddleware, PreMiddleware};
pub use self::route::Route;
pub use self::router::{Router, RouterBuilder};
#[doc(hidden)]
pub use self::service::RequestService;
pub use self::service::RouterService;
pub use self::types::RouteParams;

mod constants;
mod error;
pub mod ext;
mod helpers;
mod middleware;
pub mod prelude;
mod regex_generator;
mod route;
mod router;
mod service;
mod types;

/// A Result type often returned from methods that can have routerify errors.
pub type Result<T> = std::result::Result<T, Error>;
