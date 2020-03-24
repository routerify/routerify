extern crate routerify;
use http::HeaderValue;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server};
use lazy_static::lazy_static;
use routerify::prelude::*;
use routerify::{Middleware, Router};
use std::convert::Infallible;
use std::net::SocketAddr;

lazy_static! {
  static ref API_ROUTER: Router = Router::builder()
    .middleware(Middleware::pre(middleware_logger))
    // .middleware(Middleware::post(post_middleware))
    // .get("/users/:username/view/:attr", handle_api_users)
    .get("/users/:username/", handle_api_users)
    .all(|req| async { Ok(Response::new(Body::from("Hey2"))) })
    .build()
    .unwrap();
}

lazy_static! {
  static ref ROUTER: Router = Router::builder()
    .middleware(Middleware::pre(middleware_logger))
    .middleware(Middleware::post(post_middleware))
    .get_or_head("/", handle_home)
    // .add("/", vec![Method::GET, Method::POST, Method::HEAD], handle_home)
    .get("/about", handle_about)
    .router("/api", &*API_ROUTER)
    .all(|req| async { Ok(Response::new(Body::from("Hey"))) })
    .build()
    .unwrap();
}

async fn handle_api_users(req: Request<Body>) -> routerify::Result<Response<Body>> {
  let val = req.extensions().get::<String>();
  println!("{:?}", val);

  let params = req.params().unwrap();
  println!("{:?}", params);
  Ok(Response::new(Body::from("Fetch an user data")))
}

async fn middleware_logger(mut req: Request<Body>) -> routerify::Result<Request<Body>> {
  req.extensions_mut().insert(String::from("abc"));
  // println!("Visited: {} {}", req.method(), req.uri());
  Ok(req)
}

async fn post_middleware(mut res: Response<Body>) -> routerify::Result<Response<Body>> {
  res.headers_mut().append("X-ROUTERIFY", HeaderValue::from_static("NEO"));
  Ok(res)
}

async fn handle_home(req: Request<Body>) -> routerify::Result<Response<Body>> {
  // let val = req.extensions().get::<String>();
  // println!("{:?}", val);

  Ok(Response::new(Body::from("Hello Home")))
}

async fn handle_about(_req: Request<Body>) -> routerify::Result<Response<Body>> {
  Ok(Response::new(Body::from("Hello About")))
}

#[tokio::main]
async fn main() {
  let req_service = make_service_fn(|_conn| async {
    Ok::<_, Infallible>(service_fn(|req: Request<Body>| async {
      Ok::<Response<Body>, Infallible>(routerify::handle_request(&*ROUTER, req).await)
    }))
  });

  let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
  let server = Server::bind(&addr).serve(req_service);

  println!("App is serving on: {}", server.local_addr());
  if let Err(e) = server.await {
    eprintln!("Server Error: {}", e);
  }
}
