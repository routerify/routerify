<div align="center">
  <a href="https://github.com/routerify">
    <img width="200" height="200" src="https://avatars0.githubusercontent.com/u/64579326?s=200&v=4">
  </a>
  <br />
  <br />
</div>

[![Github Actions Status](https://github.com/routerify/routerify/workflows/Test/badge.svg)](https://github.com/routerify/routerify/actions)
[![crates.io](https://img.shields.io/crates/v/routerify.svg)](https://crates.io/crates/routerify)
[![Documentation](https://docs.rs/routerify/badge.svg)](https://docs.rs/routerify)
[![Contributors](https://img.shields.io/github/contributors/routerify/routerify.svg)](https://github.com/orgs/routerify/people)
[![MIT](https://img.shields.io/crates/l/routerify.svg)](./LICENSE)

# Routerify

`Routerify` provides a lightweight, idiomatic, composable and modular router implementation with middleware support for the Rust HTTP library [hyper](https://hyper.rs/).

Routerify's core features:

- 🌀 Design complex routing using [scopes](https://github.com/routerify/routerify/blob/master/examples/scoped_router.rs) and [middlewares](https://github.com/routerify/routerify/blob/master/examples/middleware.rs)
- 🚀 Fast route matching using [`RegexSet`](https://docs.rs/regex/1.4.3/regex/struct.RegexSet.html)
- 🍺 Route handlers may return any [HttpBody](https://docs.rs/hyper/0.14.4/hyper/body/trait.HttpBody.html)
- ❗ Flexible [error handling](https://github.com/routerify/routerify/blob/master/examples/error_handling_with_request_info.rs) strategy
- 💁 [`WebSocket` support](https://github.com/routerify/routerify-websocket) out of the box.
- 🔥 Route handlers and middleware [may share state](https://github.com/routerify/routerify/blob/master/examples/share_data_and_state.rs)
- 🍗 [Extensive documentation](https://docs.rs/routerify/) and [examples](https://github.com/routerify/routerify/tree/master/examples)


To generate a quick server app using [Routerify](https://github.com/routerify/routerify) and [hyper](https://hyper.rs/), please check out [hyper-routerify-server-template](https://github.com/routerify/hyper-routerify-server-template).

*Compiler support: requires rustc 1.48+*

## Benchmarks

| Framework      | Language    | Requests/sec |
|----------------|-------------|--------------|
| [hyper v0.14](https://github.com/hyperium/hyper) | Rust 1.50.0 | 144,583 |
| [routerify v2.0.0](https://github.com/routerify/routerify) with [hyper v0.14](https://github.com/hyperium/hyper) | Rust 1.50.0 | 144,621 |
| [actix-web v3](https://github.com/actix/actix-web) | Rust 1.50.0 | 131,292 |
| [warp v0.3](https://github.com/seanmonstar/warp) | Rust 1.50.0 | 145,362 |
| [go-httprouter, branch master](https://github.com/julienschmidt/httprouter) | Go 1.16 | 130,662 |
| [Rocket, branch master](https://github.com/SergioBenitez/Rocket) | Rust 1.50.0 | 130,045 |

For more info, please visit [Benchmarks](https://github.com/routerify/routerify-benchmark).

## Install

Add this to your `Cargo.toml` file:

```toml
[dependencies]
routerify = "2"
hyper = "0.14"
tokio = { version = "1", features = ["full"] }
```

## Basic Example

A simple example using `Routerify` with `hyper` would look like the following:

```rust
use hyper::{Body, Request, Response, Server, StatusCode};
// Import the routerify prelude traits.
use routerify::prelude::*;
use routerify::{Middleware, Router, RouterService, RequestInfo};
use std::{convert::Infallible, net::SocketAddr};

// Define an app state to share it across the route handlers and middlewares.
struct State(u64);

// A handler for "/" page.
async fn home_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    // Access the app state.
    let state = req.data::<State>().unwrap();
    println!("State value: {}", state.0);

    Ok(Response::new(Body::from("Home page")))
}

// A handler for "/users/:userId" page.
async fn user_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let user_id = req.param("userId").unwrap();
    Ok(Response::new(Body::from(format!("Hello {}", user_id))))
}

// A middleware which logs an http request.
async fn logger(req: Request<Body>) -> Result<Request<Body>, Infallible> {
    println!("{} {} {}", req.remote_addr(), req.method(), req.uri().path());
    Ok(req)
}

// Define an error handler function which will accept the `routerify::Error`
// and the request information and generates an appropriate response.
async fn error_handler(err: routerify::RouteError, _: RequestInfo) -> Response<Body> {
    eprintln!("{}", err);
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Body::from(format!("Something went wrong: {}", err)))
        .unwrap()
}

// Create a `Router<Body, Infallible>` for response body type `hyper::Body`
// and for handler error type `Infallible`.
fn router() -> Router<Body, Infallible> {
    // Create a router and specify the logger middleware and the handlers.
    // Here, "Middleware::pre" means we're adding a pre middleware which will be executed
    // before any route handlers.
    Router::builder()
        // Specify the state data which will be available to every route handlers,
        // error handler and middlewares.
        .data(State(100))
        .middleware(Middleware::pre(logger))
        .get("/", home_handler)
        .get("/users/:userId", user_handler)
        .err_handler_with_info(error_handler)
        .build()
        .unwrap()
}

#[tokio::main]
async fn main() {
    let router = router();

    // Create a Service from the router above to handle incoming requests.
    let service = RouterService::new(router).unwrap();

    // The address on which the server will be listening.
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    // Create a server by passing the created service to `.serve` method.
    let server = Server::bind(&addr).serve(service);

    println!("App is running on: {}", addr);
    if let Err(err) = server.await {
        eprintln!("Server error: {}", err);
   }
}
```

## Documentation

Please visit: [Docs](https://docs.rs/routerify) for an exhaustive documentation.

## Examples

The [examples](https://github.com/routerify/routerify/tree/master/examples).

## Contributing

Your PRs and suggestions are always welcome.
