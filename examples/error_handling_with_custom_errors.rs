use hyper::{Body, Request, Response, Server, StatusCode};
use routerify::{Router, RouterService};
use std::fmt;
use std::net::SocketAddr;

// Define a custom error enum to model a possible API service error.
#[derive(Debug)]
enum ApiError {
    #[allow(dead_code)]
    Unauthorized,
    Generic(String),
}

impl std::error::Error for ApiError {}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ApiError::Unauthorized => write!(f, "Unauthorized"),
            ApiError::Generic(s) => write!(f, "Generic: {}", s),
        }
    }
}

// Router, handlers and middleware must use the same error type.
// In this case it's `ApiError`.

// A handler for "/" page.
async fn home_handler(_: Request<Body>) -> Result<Response<Body>, ApiError> {
    // Simulate failure by returning `ApiError::Generic` variant.
    Err(ApiError::Generic("Something went wrong!".into()))
}

// Define an error handler function which will accept the `routerify::HandleError`
// and generates an appropriate response.
async fn error_handler(err: routerify::HandleError) -> Response<Body> {
    // Because `routerify::HandleError` is a boxed error, it must be
    // downcasted first. Unwrap for simplicity.
    let api_err = err.downcast::<ApiError>().unwrap();

    // Now that we've got the actual error, we can handle it
    // appropriately.
    match api_err.as_ref() {
        ApiError::Unauthorized => Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(Body::empty())
            .unwrap(),
        ApiError::Generic(s) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from(s.to_string()))
            .unwrap(),
    }
}

fn router() -> Router<Body, ApiError> {
    // Create a router and specify the the handlers.
    Router::builder()
        .get("/", home_handler)
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
