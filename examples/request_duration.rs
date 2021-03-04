use hyper::{Body, Request, Response, Server};
// Import the routerify prelude traits.
use routerify::prelude::*;
use routerify::{Router, RouterService, Middleware, RequestInfo};
use std::net::SocketAddr;
use std::convert::Infallible;

async fn before(req: Request<Body>) -> Result<Request<Body>, Infallible> {
    req.set_context(tokio::time::Instant::now());
    Ok(req)
}

async fn hello(_: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new(Body::from("Home page")))
}

async fn after(res: Response<Body>, req_info: RequestInfo) -> Result<Response<Body>, Infallible> {
    let started = req_info.context::<tokio::time::Instant>().unwrap();
    let duration = started.elapsed();
    println!("duration {:?}", duration);
    Ok(res)
}

fn router() -> Router<Body, Infallible> {
    Router::builder()
        .get("/", hello)
        .middleware(Middleware::pre(before))
        .middleware(Middleware::post_with_info(after))
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
