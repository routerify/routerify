use hyper::{Body, Request, Response, Server};
use routerify::prelude::*;
use routerify::{Middleware, Router, RouterService};
use std::{convert::Infallible, net::SocketAddr};

async fn home_handler(_: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new(Body::from("Home page")))
}

async fn about_handler(_: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new(Body::from("About page")))
}

async fn logger(req: Request<Body>) -> Result<Request<Body>, Infallible> {
    println!("{} {} {}", req.remote_addr(), req.method(), req.uri().path());
    Ok(req)
}

fn router() -> Router<Body, Infallible> {
    Router::builder()
        .middleware(Middleware::pre(logger))
        .get("/", home_handler)
        .get("/about", about_handler)
        .build()
        .unwrap()
}

#[tokio::main]
async fn main() {
    let router = router();

    let service = RouterService::new(router).unwrap();

    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));

    let server = Server::bind(&addr).serve(service);

    println!("App is running on: {}", addr);
    if let Err(err) = server.await {
        eprintln!("Server error: {}", err);
    }
}
