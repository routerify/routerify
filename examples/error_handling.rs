use hyper::{Body, Request, Response, Server, StatusCode};
use routerify::{Router, RouterService};
use std::error::Error;
use std::io;
use std::net::SocketAddr;

// A handler for "/" page.
async fn home_handler(_: Request<Body>) -> Result<Response<Body>, io::Error> {
    Err(io::Error::new(io::ErrorKind::Other, "Some errors"))
}

// A handler for "/about" page.
async fn about_handler(_: Request<Body>) -> Result<Response<Body>, io::Error> {
    Ok(Response::new(Body::from("About page")))
}

fn format_cause_chain(err: impl Error) -> String {
    let mut lines = vec![format!("error: {}", err)];
    let mut source = err.source();
    while let Some(src) = source {
        lines.push(format!("  caused by: {}", src));
        source = src.source();
    }
    lines.join("\n")
}

// Define an error handler function which will accept the `routerify::Error`
// and generates an appropriate response.
async fn error_handler(err: routerify::Error) -> Response<Body> {
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Body::from(format_cause_chain(err)))
        .unwrap()
}

fn router() -> Router<Body, io::Error> {
    // Create a router and specify the the handlers.
    Router::builder()
        .get("/", home_handler)
        .get("/about", about_handler)
        // Specify the error handler to handle any errors caused by
        // a route or any middleware.
        .err_handler(error_handler)
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
