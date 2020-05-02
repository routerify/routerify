use hyper::{Request, Response, Server};
use routerify::{Router, RouterService};
use std::{convert::Infallible, net::SocketAddr};
use tokio::prelude::*;
// Import the stream_body crate which provides an alternative version of hyper::Body with
// more features. For more info visit: https://github.com/rousan/stream-body
use stream_body::StreamBody;

// A handler for "/" page.
async fn home_handler(_: Request<hyper::Body>) -> Result<Response<StreamBody>, Infallible> {
    let (mut writer, body) = StreamBody::channel();

    tokio::spawn(async move {
        writer.write_all("Hello world ".as_bytes()).await.unwrap();
        writer.write_all("Hello world again".as_bytes()).await.unwrap();
    });

    Ok(Response::new(body))
}

// Define a router method which will return `Router<StreamBody, Infallible>` type
// and we're using `StreamBody` as response body type and `Infallible` as handler error type.
fn router() -> Router<StreamBody, Infallible> {
    Router::builder()
        .get("/", home_handler)
        // Add options handler.
        .options(
            "/*",
            |_req| async move { Ok(Response::new(StreamBody::from("Options"))) },
        )
        // Add 404 page handler.
        .any(|_req| async move { Ok(Response::new(StreamBody::from("Not Found"))) })
        // Add an error handler.
        .err_handler(|err| async move { Response::new(StreamBody::from(format!("Error: {}", err))) })
        .build()
        .unwrap()
}

#[tokio::main]
async fn main() {
    let router = router();

    // Create a Service from the router above to handle incoming requests.
    let service = RouterService::new(router);

    // The address on which the server will be listening.
    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));

    // Create a server by passing the created service to `.serve` method.
    let server = Server::bind(&addr).serve(service);

    println!("App is running on: {}", addr);
    if let Err(err) = server.await {
        eprintln!("Server error: {}", err);
    }
}
