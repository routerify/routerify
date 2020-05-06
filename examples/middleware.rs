use hyper::{header::HeaderValue, Body, Request, Response, Server};
// Import the routerify prelude traits.
use routerify::prelude::*;
use routerify::{Middleware, Router, RouterService};
use std::io;
use std::net::SocketAddr;

// A handler for "/" page.
async fn home_handler(_: Request<Body>) -> Result<Response<Body>, io::Error> {
    Ok(Response::new(Body::from("Home page")))
}

// A handler for "/about" page.
async fn about_handler(_: Request<Body>) -> Result<Response<Body>, io::Error> {
    Ok(Response::new(Body::from("About page")))
}

// Define a pre middleware handler which will be executed on every request and
// logs some meta.
async fn logger_middleware(req: Request<Body>) -> Result<Request<Body>, io::Error> {
    println!("{} {} {}", req.remote_addr(), req.method(), req.uri().path());
    Ok(req)
}

// Define a post middleware handler which will be executed on every request and
// adds a header to the response.
async fn my_custom_header_adder_middleware(mut res: Response<Body>) -> Result<Response<Body>, io::Error> {
    res.headers_mut()
        .insert("x-custom-header", HeaderValue::from_static("some value"));
    Ok(res)
}

fn router() -> Router<Body, io::Error> {
    // Create a router and specify the the handlers.
    Router::builder()
        // Create a pre middleware using `Middleware::pre()` method
        // and attach it to the router.
        .middleware(Middleware::pre(logger_middleware))
        // Create a post middleware using `Middleware::post()` method
        // and attach it to the router.
        .middleware(Middleware::post(my_custom_header_adder_middleware))
        .get("/", home_handler)
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
    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));

    // Create a server by passing the created service to `.serve` method.
    let server = Server::bind(&addr).serve(service);

    println!("App is running on: {}", addr);
    if let Err(err) = server.await {
        eprintln!("Server error: {}", err);
    }
}
