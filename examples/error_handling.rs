use hyper::{Body, Request, Response, Server, StatusCode};
use routerify::{Router, RouterService};
use std::net::SocketAddr;

// A handler for "/" page.
async fn home_handler(_: Request<Body>) -> Result<Response<Body>, routerify::Error> {
    Err(routerify::Error::new("Some errors"))
}

// A handler for "/about" page.
async fn about_handler(_: Request<Body>) -> Result<Response<Body>, routerify::Error> {
    Ok(Response::new(Body::from("About page")))
}

// Define an error handler function which will accept the `routerify::HandleError`
// and generates an appropriate response.
async fn error_handler(err: routerify::HandleError) -> Response<Body> {
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Body::from(err.to_string()))
        .unwrap()
}

fn router() -> Router<Body, routerify::Error> {
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
