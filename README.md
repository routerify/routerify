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

The `Routerify` provides a lightweight and modular router implementation with middleware support for the Rust HTTP library [hyper.rs](https://hyper.rs/).

There are a lot of web server frameworks for Rust applications out there and [hyper.rs](https://hyper.rs/) being comparably very fast and ready for production use
is one of them, and it provides only low level API. It doesn't provide any complex routing feature. So, `Routerify` extends the [hyper.rs](https://hyper.rs/) library
by providing that missing feature without compromising any performance.

The `Routerify` offers the following features:

- 📡 Allows defining complex routing logic.
- 🔨 Provides middleware support.
- 🌀 Supports Route Parameters.
- 🚀 Fast as it's using [`RegexSet`](https://docs.rs/regex/1.3.7/regex/struct.RegexSet.html) to match routes. 
- 🍺 It supports any response body type as long as it implements the [HttpBody](https://docs.rs/hyper/0.13.5/hyper/body/trait.HttpBody.html) trait.
- ❗ Provides a flexible error handling strategy.
- 🍗 Exhaustive [examples](https://github.com/routerify/routerify/tree/master/examples) and well documented.

To generate a quick server app using [Routerify](https://github.com/routerify/routerify) and [hyper.rs](https://hyper.rs/), please check out [hyper-routerify-server-template](https://github.com/routerify/hyper-routerify-server-template).

## Benchmarks

| Framework      | Language    | Requests/sec |
|----------------|-------------|--------------|
| [hyper v0.13](https://github.com/hyperium/hyper) | Rust 1.43.0 | 112,557 |
| [routerify v1.1](https://github.com/routerify/routerify) with [hyper v0.13](https://github.com/hyperium/hyper) | Rust 1.43.0 | 112,320 |
| [gotham v0.4.0](https://github.com/gotham-rs/gotham) | Rust 1.43.0 | 100,097 |
| [actix-web v2](https://github.com/actix/actix-web) | Rust 1.43.0 | 96,397 |
| [warp v0.2](https://github.com/seanmonstar/warp) | Rust 1.43.0 | 81,912 |
| [go-httprouter, branch master](https://github.com/julienschmidt/httprouter) | Go 1.13.7 | 74,958 |
| [Rocket, branch async](https://github.com/SergioBenitez/Rocket) | Rust 1.43.0 | 2,041 ? |

For more info, please visit [Benchmarks](https://github.com/routerify/routerify-benchmark).

## Install

Add this to your `Cargo.toml` file:

```toml
[dependencies]
routerify = "1.0"
```

## Basic Example

A simple example using `Routerify` with [hyper.rs](https://hyper.rs/) would look like the following:

```rust
use hyper::{Body, Request, Response, Server};
// Import the routerify prelude traits.
use routerify::prelude::*;
use routerify::{Middleware, Router, RouterService};
use std::{convert::Infallible, net::SocketAddr};

// A handler for "/:name" page.
async fn home_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let name = req.param("name").unwrap();
    Ok(Response::new(Body::from(format!("Hello {}", name))))
}

// A handler for "/about" page.
async fn about_handler(_: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new(Body::from("About page")))
}

// A middleware which logs an http request.
async fn logger(req: Request<Body>) -> Result<Request<Body>, Infallible> {
    println!("{} {} {}", req.remote_addr(), req.method(), req.uri().path());
    Ok(req)
}

// Create a `Router<Body, Infallible>` for response body type `hyper::Body` and for handler error type `Infallible`.
fn router() -> Router<Body, Infallible> {
    // Create a router and specify the logger middleware and the handlers.
    // Here, "Middleware::pre" means we're adding a pre middleware which will be executed
    // before any route handlers.
    Router::builder()
        .middleware(Middleware::pre(logger))
        .get("/:name", home_handler)
        .get("/about", about_handler)
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

The common [examples](https://github.com/routerify/routerify/tree/master/examples).

## Contributing

Your PRs and suggestions are always welcome.