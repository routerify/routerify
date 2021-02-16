use self::support::{into_text, serve};
use hyper::{Body, Client, Request, Response};
use routerify::prelude::RequestExt;
use routerify::Router;
use std::io;
use std::sync::{Arc, Mutex};

mod support;

#[tokio::test]
async fn can_perform_simple_get_request() {
    const RESPONSE_TEXT: &str = "Hello world";
    let router: Router<Body, routerify::Error> = Router::builder()
        .get("/", |_| async move { Ok(Response::new(RESPONSE_TEXT.into())) })
        .build()
        .unwrap();
    let serve = serve(router).await;
    let resp = Client::new()
        .request(
            Request::builder()
                .method("GET")
                .uri(format!("http://{}/", serve.addr()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let resp = into_text(resp.into_body()).await;
    assert_eq!(resp, RESPONSE_TEXT.to_owned());
    serve.shutdown();
}

#[tokio::test]
async fn can_respond_with_data_from_scope_state() {
    // Creating two modules containing separate state and routes which expose that state directly...
    mod service1 {
        use super::*;
        struct State {
            count: Arc<Mutex<u8>>,
        }
        async fn list(req: Request<Body>) -> Result<Response<Body>, io::Error> {
            let count = req.data::<State>().unwrap().count.lock().unwrap();
            Ok(Response::new(Body::from(format!("{}", count))))
        }
        pub fn router() -> Router<Body, io::Error> {
            let state = State {
                count: Arc::new(Mutex::new(1)),
            };
            Router::builder().data(state).get("/", list).build().unwrap()
        }
    }

    mod service2 {
        use super::*;
        struct State {
            count: Arc<Mutex<u8>>,
        }
        async fn list(req: Request<Body>) -> Result<Response<Body>, io::Error> {
            let count = req.data::<State>().unwrap().count.lock().unwrap();
            Ok(Response::new(Body::from(format!("{}", count))))
        }
        pub fn router() -> Router<Body, io::Error> {
            let state = State {
                count: Arc::new(Mutex::new(2)),
            };
            Router::builder().data(state).get("/", list).build().unwrap()
        }
    }

    let router = Router::builder()
        .scope(
            "/v1",
            Router::builder()
                .scope("/service1", service1::router())
                .scope("/service2", service2::router())
                .build()
                .unwrap(),
        )
        .build()
        .unwrap();
    let serve = serve(router).await;

    // Ensure response contains service1's unique data.
    let resp = Client::new()
        .request(serve.new_request("GET", "/v1/service1").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(200, resp.status().as_u16());
    assert_eq!("1", into_text(resp.into_body()).await);

    // Ensure response contains service2's unique data.
    let resp = Client::new()
        .request(serve.new_request("GET", "/v1/service2").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(200, resp.status().as_u16());
    assert_eq!(into_text(resp.into_body()).await, "2");

    serve.shutdown();
}
