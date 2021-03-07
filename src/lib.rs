//! `Routerify` provides a lightweight, idiomatic, composable and modular router implementation with middleware support for the Rust HTTP library [hyper](https://hyper.rs/).
//!
//! Routerify's core features:
//!
//! - 🌀 Design complex routing using [scopes](https://github.com/routerify/routerify/blob/master/examples/scoped_router.rs) and [middlewares](https://github.com/routerify/routerify/blob/master/examples/middleware.rs)
//!
//! - 🚀 Fast route matching using [`RegexSet`](https://docs.rs/regex/1.4.3/regex/struct.RegexSet.html)
//!
//! - 🍺 Route handlers may return any [HttpBody](https://docs.rs/hyper/0.14.4/hyper/body/trait.HttpBody.html)
//!
//! - ❗ Flexible [error handling](https://github.com/routerify/routerify/blob/master/examples/error_handling_with_request_info.rs) strategy
//!
//! - 💁 [`WebSocket` support](https://github.com/routerify/routerify-websocket) out of the box.
//!
//! - 🔥 Route handlers and middleware [may share state](https://github.com/routerify/routerify/blob/master/examples/share_data_and_state.rs)
//!
//! - 🍗 [Extensive documentation](https://docs.rs/routerify/) and [examples](https://github.com/routerify/routerify/tree/master/examples)
//!
//! To generate a quick server app using [Routerify](https://github.com/routerify/routerify) and [hyper](https://hyper.rs/),
//! please check out [hyper-routerify-server-template](https://github.com/routerify/hyper-routerify-server-template).
//!
//!
//! ## Benchmarks
//!
//! | Framework      | Language    | Requests/sec |
//! |----------------|-------------|--------------|
//! | [hyper v0.14](https://github.com/hyperium/hyper) | Rust 1.50.0 | 144,583 |
//! | [routerify v2.0.0-beta-4](https://github.com/routerify/routerify) with [hyper v0.14](https://github.com/hyperium/hyper) | Rust 1.50.0 | 144,621 |
//! | [actix-web v3](https://github.com/actix/actix-web) | Rust 1.50.0 | 131,292 |
//! | [warp v0.3](https://github.com/seanmonstar/warp) | Rust 1.50.0 | 145,362 |
//! | [go-httprouter, branch master](https://github.com/julienschmidt/httprouter) | Go 1.16 | 130,662 |
//! | [Rocket, branch master](https://github.com/SergioBenitez/Rocket) | Rust 1.50.0 | 130,045 |
//!
//! For more info, please visit [Benchmarks](https://github.com/routerify/routerify-benchmark).
//!
//! ## Basic Example
//!
//! A simple example using `Routerify` with `hyper` would look like the following:
//!
//! ```no_run
//! use hyper::{Body, Request, Response, Server, StatusCode};
//! // Import the routerify prelude traits.
//! use routerify::prelude::*;
//! use routerify::{Middleware, Router, RouterService, RequestInfo};
//! use std::{convert::Infallible, net::SocketAddr};
//!
//! // Define an app state to share it across the route handlers and middlewares.
//! struct State(u64);
//!
//! // A handler for "/" page.
//! async fn home_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
//!     // Access the app state.
//!     let state = req.data::<State>().unwrap();
//!     println!("State value: {}", state.0);
//!
//!     Ok(Response::new(Body::from("Home page")))
//! }
//!
//! // A handler for "/users/:userId" page.
//! async fn user_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
//!     let user_id = req.param("userId").unwrap();
//!     Ok(Response::new(Body::from(format!("Hello {}", user_id))))
//! }
//!
//! // A middleware which logs an http request.
//! async fn logger(req: Request<Body>) -> Result<Request<Body>, Infallible> {
//!     println!("{} {} {}", req.remote_addr(), req.method(), req.uri().path());
//!     Ok(req)
//! }
//!
//! // Define an error handler function which will accept the `routerify::Error`
//! // and the request information and generates an appropriate response.
//! async fn error_handler(err: routerify::HandleError, _: RequestInfo) -> Response<Body> {
//!     eprintln!("{}", err);
//!     Response::builder()
//!         .status(StatusCode::INTERNAL_SERVER_ERROR)
//!         .body(Body::from(format!("Something went wrong: {}", err)))
//!         .unwrap()
//! }
//!
//! // Create a `Router<Body, Infallible>` for response body type `hyper::Body`
//! // and for handler error type `Infallible`.
//! fn router() -> Router<Body, Infallible> {
//!     // Create a router and specify the logger middleware and the handlers.
//!     // Here, "Middleware::pre" means we're adding a pre middleware which will be executed
//!     // before any route handlers.
//!     Router::builder()
//!         // Specify the state data which will be available to every route handlers,
//!         // error handler and middlewares.
//!         .data(State(100))
//!         .middleware(Middleware::pre(logger))
//!         .get("/", home_handler)
//!         .get("/users/:userId", user_handler)
//!         .err_handler_with_info(error_handler)
//!         .build()
//!         .unwrap()
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let router = router();
//!
//!     // Create a Service from the router above to handle incoming requests.
//!     let service = RouterService::new(router).unwrap();
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
//!
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
//! The post Middlewares will be executed after all the route handlers process the request and generates a response and it will access that response object and the request info(optional)
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
//!     // Then return the transformed response object to be consumed by the other middlewares.
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
//! #### Post Middleware with Request Info
//!
//! Sometimes, the post middleware requires the request informations e.g. headers, method, uri etc to generate a new response. As an example, it could be used to manage
//! sessions. To register this kind of post middleware, you have to use [`Middleware::post_with_info`](./enum.Middleware.html#method.post_with_info) method as follows:
//!
//! ```
//! use routerify::{Router, Middleware, RequestInfo};
//! use hyper::{Response, Body};
//! use std::convert::Infallible;
//!
//! // The handler for a post middleware which requires request info.
//! // It accepts `res` and `req_info` and it transforms the `res` and passes it to the next middlewares.
//! async fn post_middleware_with_info_handler(res: Response<Body>, req_info: RequestInfo) -> Result<Response<Body>, Infallible> {
//!     let transformed_res = res;
//!
//!     // Do some response transformation based on the request headers, method etc.
//!     let _headers = req_info.headers();
//!
//!     // Then return the transformed response object to be consumed by the other middlewares.
//!     Ok(transformed_res)
//! }
//!
//! # fn run() -> Router<Body, Infallible> {
//! let router = Router::builder()
//!      // Create a post middleware instance by `Middleware::post_with_info` method
//!      // and attach it.
//!      .middleware(Middleware::post_with_info(post_middleware_with_info_handler))
//!      // This middleware can also be attached on a specific path as shown below.
//!      .middleware(Middleware::post_with_info_with_path("/my-path", post_middleware_with_info_handler).unwrap())
//!      .build()
//!      .unwrap();
//! # router
//! # }
//! # run();
//! ```
//!
//! ### The built-in Middleware
//!
//! Here is a list of some middlewares which are published in different crates:
//!
//! - [routerify-cors](https://github.com/routerify/routerify-cors): A post middleware which enables `CORS` to the routes.
//! - [routerify-query](https://github.com/routerify/routerify-query): A pre middleware which parses the request query string.
//!
//! ## Data and State Sharing
//!
//! `Routerify` also allows you to share data or app state across the route handlers, middlewares and the error handler via the [`RouterBuilder`](./struct.RouterBuilder.html) method
//! [`data`](./struct.RouterBuilder.html#method.data). As it provides composable router API, it also allows to have app state/data per each sub-router.
//!
//! Here is an example to share app state:
//!
//! ```
//! # use hyper::{Body, Request, Response, Server, StatusCode};
//! // Import the routerify prelude traits.
//! use routerify::prelude::*;
//! use routerify::{Middleware, Router, RouterService, RequestInfo};
//! # use std::{convert::Infallible, net::SocketAddr};
//!
//! // Define an app state to share it across the route handlers, middlewares
//! // and the error handler.
//! struct State(u64);
//!
//! // A handler for "/" page.
//! async fn home_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
//!     // Access the app state.
//!     let state = req.data::<State>().unwrap();
//!     println!("State value: {}", state.0);
//!
//!     Ok(Response::new(Body::from("Home page")))
//! }
//!
//! // A middleware which logs an http request.
//! async fn logger(req: Request<Body>) -> Result<Request<Body>, Infallible> {
//!     // You can also access the same state from middleware.
//!     let state = req.data::<State>().unwrap();
//!     println!("State value: {}", state.0);
//!
//!     println!("{} {} {}", req.remote_addr(), req.method(), req.uri().path());
//!     Ok(req)
//! }
//!
//! // Define an error handler function which will accept the `routerify::Error`
//! // and the request information and generates an appropriate response.
//! async fn error_handler(err: routerify::HandleError, req_info: RequestInfo) -> Response<Body> {
//!     // You can also access the same state from error handler.
//!     let state = req_info.data::<State>().unwrap();
//!     println!("State value: {}", state.0);
//!
//!     eprintln!("{}", err);
//!     Response::builder()
//!         .status(StatusCode::INTERNAL_SERVER_ERROR)
//!         .body(Body::from(format!("Something went wrong: {}", err)))
//!         .unwrap()
//! }
//!
//! // Create a `Router<Body, Infallible>` for response body type `hyper::Body`
//! // and for handler error type `Infallible`.
//! fn router() -> Router<Body, Infallible> {
//!     Router::builder()
//!         // Specify the state data which will be available to every route handlers,
//!         // error handler and middlewares.
//!         .data(State(100))
//!         .middleware(Middleware::pre(logger))
//!         .get("/", home_handler)
//!         .err_handler_with_info(error_handler)
//!         .build()
//!         .unwrap()
//! }
//!
//! # #[tokio::main]
//! # async fn run() {
//! #    let router = router();
//! #
//! #    // Create a Service from the router above to handle incoming requests.
//! #    let service = RouterService::new(router).unwrap();
//! #
//! #   // The address on which the server will be listening.
//! #   let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
//! #
//! #   // Create a server by passing the created service to `.serve` method.
//! #   let server = Server::bind(&addr).serve(service);
//! #
//! #   println!("App is running on: {}", addr);
//! #   if let Err(err) = server.await {
//! #       eprintln!("Server error: {}", err);
//! #  }
//! # }
//! ```
//!
//! Here is an example to `mutate` app state:
//!
//! ```
//! # use hyper::{Body, Request, Response, Server, StatusCode};
//! // Import the routerify prelude traits.
//! use routerify::prelude::*;
//! use routerify::{Middleware, Router, RouterService, RequestInfo};
//! # use std::{convert::Infallible, net::SocketAddr};
//! use std::sync::Mutex;
//!
//! // Define an app state to share it across the route handlers, middlewares
//! // and the error handler.
//! struct State(u64);
//!
//! // A handler for "/" page.
//! async fn home_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
//!     // Access the app state.
//!     let state = req.data::<Mutex<State>>().unwrap();
//!     let mut lock = state.lock().unwrap();
//!     // Mutate the app state if needed.
//!     lock.0 += 1;
//!
//!     println!("Updated State value: {}", lock.0);
//!
//!     Ok(Response::new(Body::from("Home page")))
//! }
//!
//! // Create a `Router<Body, Infallible>` for response body type `hyper::Body`
//! // and for handler error type `Infallible`.
//! fn router() -> Router<Body, Infallible> {
//!     Router::builder()
//!         // Specify the state data which will be available to every route handlers,
//!         // error handler and middlewares.
//!         .data(Mutex::new(State(100)))
//!         .get("/", home_handler)
//!         .build()
//!         .unwrap()
//! }
//!
//! # #[tokio::main]
//! # async fn run() {
//! #    let router = router();
//! #
//! #    // Create a Service from the router above to handle incoming requests.
//! #    let service = RouterService::new(router).unwrap();
//! #
//! #   // The address on which the server will be listening.
//! #   let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
//! #
//! #   // Create a server by passing the created service to `.serve` method.
//! #   let server = Server::bind(&addr).serve(service);
//! #
//! #   println!("App is running on: {}", addr);
//! #   if let Err(err) = server.await {
//! #       eprintln!("Server error: {}", err);
//! #  }
//! # }
//! ```
//!
//! Here is any example on having app state per each sub-router:
//!
//! ```
//! # use hyper::{Body, Request, Response, Server, StatusCode};
//! # // Import the routerify prelude traits.
//! # use routerify::prelude::*;
//! # use routerify::{Middleware, Router, RouterService, RequestInfo};
//! # use std::{convert::Infallible, net::SocketAddr};
//!
//! mod foo {
//!     # use std::{convert::Infallible, net::SocketAddr};
//!     # use routerify::{Middleware, Router, RouterService, RequestInfo};
//!     # use hyper::{Body, Request, Response, Server, StatusCode};
//!     pub fn router() -> Router<Body, Infallible> {
//!         Router::builder()
//!             // Specify data for this sub-router only.
//!             .data("Data for foo router")
//!             .build()
//!             .unwrap()
//!     }
//! }
//!
//! mod bar {
//!     # use std::{convert::Infallible, net::SocketAddr};
//!     # use routerify::{Middleware, Router, RouterService, RequestInfo};
//!     # use hyper::{Body, Request, Response, Server, StatusCode};
//!     pub fn router() -> Router<Body, Infallible> {
//!         Router::builder()
//!             // Specify data for this sub-router only.
//!             .data("Data for bar router")
//!             .build()
//!             .unwrap()
//!     }
//! }
//!
//! fn router() -> Router<Body, Infallible> {
//!     Router::builder()
//!         // This data will be available to all the child sub-routers.
//!         .data(100_u32)
//!         .scope("/foo", foo::router())
//!         .scope("/bar", bar::router())
//!         .build()
//!         .unwrap()
//! }
//! ```
//!
//! You can also share multiple data as follows:
//!
//! ```
//! # use hyper::{Body, Request, Response, Server, StatusCode};
//! # // Import the routerify prelude traits.
//! # use routerify::prelude::*;
//! # use routerify::{Middleware, Router, RouterService, RequestInfo};
//! # use std::{convert::Infallible, net::SocketAddr};
//! # use std::sync::Mutex;
//! fn router() -> Router<Body, Infallible> {
//!     Router::builder()
//!         // Share multiple data, a single data for each data type.
//!         .data(100_u32)
//!         .data(String::from("Hello world"))
//!         .build()
//!         .unwrap()
//! }
//! ```
//!
//! ### Request context
//!
//! It's possible to share data local to the request across the route handlers and middleware via the
//! [`RequestExt`](./ext/trait.RequestExt.html) methods [`context`](./ext/trait.RequestExt.html#method.context)
//! and [`set_context`](./ext/trait.RequestExt.html#method.set_context). In the error handler it can be accessed
//! via [`RequestInfo`](./struct.RequestInfo.html) method [`context`](./struct.RequestInfo.html#method.context).
//!
//! ## Error Handling
//!
//! Any route or middleware could go wrong and throws an error. `Routerify` tries to add a default error handler in some cases. But, it also
//! allow to attach a custom error handler. The error handler generates a response based on the error and the request info(optional).
//!
//! Here is an basic example:
//!
//! ```
//! use routerify::{Router, Middleware};
//! use routerify::prelude::*;
//! use hyper::{Response, Body, StatusCode};
//!
//! // The error handler will accept the thrown error in routerify::Error type and
//! // it will have to generate a response based on the error.
//! async fn error_handler(err: routerify::HandleError) -> Response<Body> {
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
//!
//! ### Error Handling with Request Info
//!
//! Sometimes, it's needed to to generate response on error based on the request headers, method, uri etc. `Routerify` also provides a method [`err_handler_with_info`](./struct.RouterBuilder.html#method.err_handler_with_info)
//! to register this kind of error handler as follows:
//!
//! ```
//! use routerify::{Router, Middleware, RequestInfo};
//! use routerify::prelude::*;
//! use hyper::{Response, Body, StatusCode};
//!
//! // The error handler will accept the thrown error and the request info and
//! // it will generate a response.
//! async fn error_handler(err: routerify::HandleError, req_info: RequestInfo) -> Response<Body> {
//!     // Now generate response based on the `err` and the `req_info`.
//!     Response::builder()
//!       .status(StatusCode::INTERNAL_SERVER_ERROR)
//!       .body(Body::from("Something went wrong"))
//!       .unwrap()
//! }
//!
//! # fn run() -> Router<Body, hyper::Error> {
//! let router = Router::builder()
//!      .get("/users", |req| async move { Ok(Response::new(Body::from("It might raise an error"))) })
//!      // Now register this error handler.
//!      .err_handler_with_info(error_handler)
//!      .build()
//!      .unwrap();
//! # router
//! # }
//! # run();
//! ```

pub use self::error::{Error, HandleError};
pub use self::middleware::{Middleware, PostMiddleware, PreMiddleware};
pub use self::route::Route;
pub use self::router::{Router, RouterBuilder};
#[doc(hidden)]
pub use self::service::RequestService;
pub use self::service::RequestServiceBuilder;
pub use self::service::RouterService;
pub use self::types::{RequestInfo, RouteParams};

mod constants;
mod data_map;
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
pub type Result<T> = std::result::Result<T, HandleError>;
