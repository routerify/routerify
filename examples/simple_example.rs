use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use routerify::prelude::*;
use routerify::{Middleware, Router, RouterService};
use std::{convert::Infallible, net::SocketAddr};

// A handler for "/" page.
async fn home(_: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new(Body::from("Home page")))
}

// A handler for "/about" page.
async fn about(_: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new(Body::from("About page")))
}

// A middleware which logs an http request.
async fn logger(req: Request<Body>) -> Result<Request<Body>, Infallible> {
    println!("{} {} {}", req.remote_addr(), req.method(), req.uri().path());
    Ok(req)
}
#[tokio::main]
async fn main() {
    // Create a router and specify the logger middleware and the handlers.
    // Here, "Middleware::pre" means we're adding a pre-middleware which will accept
    // a request and transforms it to a new request.
    let router = Router::builder()
        .middleware(Middleware::pre(logger))
        .get("/", home)
        .get("/about", about)
        .build()
        .unwrap();

    // Create a Service from the router above to handle all the incoming requests.
    let service = RouterService::new(router);

    // The address on which the server will be listening.
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    // Create the server by passing the created service to .`serve` method.
    let server = Server::bind(&addr).serve(service);

    println!("App is running on: {}", addr);
    if let Err(err) = server.await {
        eprintln!("Server error: {}", err);
    }
}
