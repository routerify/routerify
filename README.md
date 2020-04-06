# Routerify

A lightweight router implementation with middleware support for the rust HTTP library [hyper](https://hyper.rs/).

## Usage

Add this to your Cargo.toml:

```toml
[dependencies]
routerify = "0.1"
```

## Examples

```rust
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use lazy_static::lazy_static;
use routerify::{Middleware, Router};
use std::convert::Infallible;
use std::net::SocketAddr;

lazy_static! {
  static ref API_ROUTER: Router = Router::builder()
    .middleware(Middleware::pre(middleware_logger))
    .get("/users/:username/view", handle_api_users)
    .build()
    .unwrap();
}

lazy_static! {
  static ref ROUTER: Router = Router::builder()
    .middleware(Middleware::pre(middleware_logger))
    .get("/", handle_home)
    .router("/api", &*API_ROUTER)
    .build()
    .unwrap();
}

async fn handle_api_users(_req: Request<Body>) -> routerify::Result<Response<Body>> {
  Ok(Response::new(Body::from("Fetch an user data")))
}

async fn middleware_logger(req: Request<Body>) -> routerify::Result<Request<Body>> {
  println!("Visited: {} {}", req.method(), req.uri());
  Ok(req)
}

async fn handle_home(_req: Request<Body>) -> routerify::Result<Response<Body>> {
  Ok(Response::new(Body::from("Hello Home")))
}

#[tokio::main]
async fn main() {
  let req_service = make_service_fn(|_conn| async {
    Ok::<_, Infallible>(service_fn(|req: Request<Body>| async {
      Ok::<Response<Body>, Infallible>(routerify::handle_request(&*ROUTER, req).await)
    }))
  });

  let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
  let server = Server::bind(&addr)
    .serve(req_service);

  println!("App is serving on: {}", server.local_addr());
  if let Err(e) = server.await {
    eprintln!("Server Error: {}", e);
  }
}
```