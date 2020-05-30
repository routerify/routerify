use hyper::{Body, Request, Response, Server};
// Import the routerify prelude traits.
use routerify::prelude::*;
use routerify::{Router, RouterService};
use std::io;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

mod users {
    use super::*;

    struct State {
        count: Arc<Mutex<u8>>,
    }

    async fn list(req: Request<Body>) -> Result<Response<Body>, io::Error> {
        let count = req.data::<State>().unwrap().count.lock().unwrap();
        Ok(Response::new(Body::from(format!("Suppliers: {}", count))))
    }

    pub fn router() -> Router<Body, io::Error> {
        let state = State {
            count: Arc::new(Mutex::new(20)),
        };
        Router::builder().data(state).get("/", list).build().unwrap()
    }
}

mod offers {
    use super::*;

    struct State {
        count: Arc<Mutex<u8>>,
    }

    async fn list(req: Request<Body>) -> Result<Response<Body>, io::Error> {
        let count = req.data::<State>().unwrap().count.lock().unwrap();

        println!("I can also access parent state: {:?}", req.data::<String>().unwrap());

        Ok(Response::new(Body::from(format!("Suppliers: {}", count))))
    }

    pub fn router() -> Router<Body, io::Error> {
        let state = State {
            count: Arc::new(Mutex::new(100)),
        };
        Router::builder().data(state).get("/", list).build().unwrap()
    }
}

#[tokio::main]
async fn main() {
    let scopes = Router::builder()
        .data("Parent State data".to_owned())
        .scope("/offers", offers::router())
        .scope("/users", users::router())
        .build()
        .unwrap();

    let router = Router::builder().scope("/v1", scopes).build().unwrap();

    let service = RouterService::new(router).unwrap();
    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
    let server = Server::bind(&addr).serve(service);
    println!("App is running on: {}", addr);
    if let Err(err) = server.await {
        eprintln!("Server error: {}", err);
    }
}
