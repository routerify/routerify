use self::support::{into_text, serve};
use hyper::{Body, Client, Request, Response, StatusCode};
use routerify::prelude::RequestExt;
use routerify::{Middleware, RequestInfo, Router};
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
async fn can_perform_simple_get_request_boxed_error() {
    const RESPONSE_TEXT: &str = "Hello world";
    type BoxedError = Box<dyn std::error::Error + Sync + Send + 'static>;
    let router: Router<Body, BoxedError> = Router::builder()
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

#[tokio::test]
async fn can_propagate_request_context() {
    use std::io;
    #[derive(Debug, Clone, PartialEq)]
    struct Id(u32);

    let before = |req: Request<Body>| async move {
        req.set_context(Id(42));
        Ok(req)
    };

    let index = |req: Request<Body>| async move {
        // Check `id` from `before()`.
        let id = req.context::<Id>().unwrap();
        assert_eq!(id, Id(42));

        // Check that non-existent context value is None.
        let none = req.context::<u64>();
        assert!(none.is_none());

        // Add a String value to the context.
        req.set_context("index".to_string());

        // Trigger this error in order to invoke
        // the error handler.
        Err(io::Error::new(io::ErrorKind::AddrInUse, "bogus error"))
    };

    let error_handler = |_err, req_info: RequestInfo| async move {
        // Check `id` from `before()`.
        let id = req_info.context::<Id>().unwrap();
        assert_eq!(id, Id(42));

        // Check String from `index()`.
        let name = req_info.context::<String>().unwrap();
        assert_eq!(name, "index");

        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from("Something went wrong"))
            .unwrap()
    };

    let after = |res, req_info: RequestInfo| async move {
        // Check `id` from `before()`.
        let id = req_info.context::<Id>().unwrap();
        assert_eq!(id, Id(42));

        // Check String from `index()`.
        let name = req_info.context::<String>().unwrap();
        assert_eq!(name, "index");

        Ok(res)
    };

    let router: Router<Body, std::io::Error> = Router::builder()
        .middleware(Middleware::pre(before))
        .middleware(Middleware::post_with_info(after))
        .err_handler_with_info(error_handler)
        .get("/", index)
        .build()
        .unwrap();
    let serve = serve(router).await;
    let _ = Client::new()
        .request(
            Request::builder()
                .method("GET")
                .uri(format!("http://{}/", serve.addr()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    serve.shutdown();
}

#[tokio::test]
async fn can_extract_path_params() {
    const RESPONSE_TEXT: &str = "Hello world";
    let router: Router<Body, routerify::Error> = Router::builder()
        .get("/api/:first/plus/:second", |req| async move {
            let first = req.param("first").unwrap();
            let second = req.param("second").unwrap();
            assert_eq!(first, "40");
            assert_eq!(second, "2");
            Ok(Response::new(RESPONSE_TEXT.into()))
        })
        .build()
        .unwrap();
    let serve = serve(router).await;
    let resp = Client::new()
        .request(
            Request::builder()
                .method("GET")
                .uri(format!("http://{}/api/40/plus/2", serve.addr()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let resp = into_text(resp.into_body()).await;
    assert_eq!(resp, RESPONSE_TEXT.to_owned());
    serve.shutdown();
}
