use self::support::{into_text, serve};
use hyper::{Body, Client, Request, Response, StatusCode};
use routerify::prelude::RequestExt;
use routerify::{Middleware, RequestInfo, RouteError, Router};
use std::io;
use std::sync::{Arc, Mutex};

mod support;

#[tokio::test]
async fn can_perform_simple_get_request() {
    const RESPONSE_TEXT: &str = "Hello world";
    let router: Router<Body, routerify::Error> = Router::builder()
        .get("/", |_| async move { Ok(Response::new(RESPONSE_TEXT.into())) })
        .err_handler(|_: RouteError| async move { todo!() })
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
        .err_handler(|_: RouteError| async move { todo!() })
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
    #[derive(Debug, Clone, PartialEq)]
    struct Id2(u32);

    let before = |req: Request<Body>| async move {
        req.set_context(Id(42));
        let (parts, body) = req.into_parts();
        parts.set_context(Id2(42));
        Ok(Request::from_parts(parts, body))
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

        let (parts, _) = req.into_parts();

        // Check `id2` from `before()`.
        let id2 = parts.context::<Id2>().unwrap();
        assert_eq!(id2, Id2(42));

        // Update the Id2 value in the context.
        parts.set_context(Id2(1));

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

        // Check updated `id2` from `index()`.
        let id2 = req_info.context::<Id2>().unwrap();
        assert_eq!(id2, Id2(1));

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

        // Check updated `id2` from `index()`.
        let id2 = req_info.context::<Id2>().unwrap();
        assert_eq!(id2, Id2(1));

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
            let (parts, _) = req.into_parts();
            let first = parts.param("first").unwrap();
            let second = parts.param("second").unwrap();
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


#[tokio::test]
async fn can_extract_extension_path_params_1() {
    const RESPONSE_TEXT: &str = "Hello world";
    let router: Router<Body, routerify::Error> = Router::builder()
        .get("/api/:id.json", |req| async move {
            let id = req.param("id").unwrap();
            assert_eq!(id, "40");
            let (parts, _) = req.into_parts();
            let id = parts.param("id").unwrap();
            assert_eq!(id, "40");
            Ok(Response::new(RESPONSE_TEXT.into()))
        })
        .build()
        .unwrap();
    let serve = serve(router).await;
    let resp = Client::new()
        .request(
            Request::builder()
                .method("GET")
                .uri(format!("http://{}/api/40.json", serve.addr()))
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
async fn can_extract_extension_path_params_2() {
    const RESPONSE_TEXT: &str = "Hello world";
    let router: Router<Body, routerify::Error> = Router::builder()
        .get("/api/:fileName", |req| async move {
            let file_name = req.param("fileName").unwrap();
            assert_eq!(file_name, "data.json");
            let (parts, _) = req.into_parts();
            let file_name = parts.param("fileName").unwrap();
            assert_eq!(file_name, "data.json");
            Ok(Response::new(RESPONSE_TEXT.into()))
        })
        .build()
        .unwrap();
    let serve = serve(router).await;
    let resp = Client::new()
        .request(
            Request::builder()
                .method("GET")
                .uri(format!("http://api/data.json", serve.addr()))
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
async fn do_not_execute_scoped_middleware_for_unscoped_path() {
    let api_router: Router<Body, routerify::Error> = Router::builder()
        .middleware(Middleware::pre(|_| async { panic!("should not be executed") }))
        .middleware(Middleware::post(|_| async { panic!("should not be executed") }))
        .get("/api/todo", |_| async { Ok(Response::new("".into())) })
        .build()
        .unwrap();

    let router: Router<Body, routerify::Error> = Router::builder()
        .get("/", |_| async { Ok(Response::new("".into())) })
        .scope("/api", api_router)
        .get("/api/login", |_| async { Ok(Response::new("".into())) })
        .build()
        .unwrap();

    let serve = serve(router).await;
    let _ = Client::new()
        .request(
            Request::builder()
                .method("GET")
                .uri(format!("http://{}/api/login", serve.addr()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    serve.shutdown();
}

#[tokio::test]
async fn execute_scoped_middleware_when_no_unscoped_match() {
    use std::sync::atomic::{AtomicBool, Ordering::SeqCst};
    use std::sync::Arc;

    struct ExecPre(AtomicBool);
    struct ExecPost(AtomicBool);

    let executed_pre = Arc::new(ExecPre(AtomicBool::new(false)));
    let executed_post = Arc::new(ExecPost(AtomicBool::new(false)));

    // Record the execution of pre and post middleware.
    let api_router: Router<Body, routerify::Error> = Router::builder()
        .middleware(Middleware::pre(|req| async {
            let pre = req.data::<Arc<ExecPre>>().unwrap();
            pre.0.store(true, SeqCst);
            Ok(req)
        }))
        .middleware(Middleware::pre(|req| async {
            let post = req.data::<Arc<ExecPost>>().unwrap();
            post.0.store(true, SeqCst);
            Ok(req)
        }))
        .get("/api/todo", |_| async { Ok(Response::new("".into())) })
        .build()
        .unwrap();

    let router: Router<Body, routerify::Error> = Router::builder()
        .data(executed_pre.clone())
        .data(executed_post.clone())
        .get("/", |_| async { Ok(Response::new("".into())) })
        .scope("/api", api_router)
        .get("/api/login", |_| async { Ok(Response::new("".into())) })
        .build()
        .unwrap();

    let serve = serve(router).await;
    let _ = Client::new()
        .request(
            Request::builder()
                .method("GET")
                .uri(format!("http://{}/api/nomatch", serve.addr()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(executed_pre.0.load(SeqCst));
    assert!(executed_post.0.load(SeqCst));

    serve.shutdown();
}

#[tokio::test]
async fn can_handle_custom_errors() {
    #[derive(Debug)]
    enum ApiError {
        Generic(String),
    }
    impl std::error::Error for ApiError {}
    impl std::fmt::Display for ApiError {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            match self {
                ApiError::Generic(s) => write!(f, "Generic: {}", s),
            }
        }
    }

    const RESPONSE_TEXT: &str = "Something went wrong!";
    let router: Router<Body, ApiError> = Router::builder()
        .get("/", |_| async move { Err(ApiError::Generic(RESPONSE_TEXT.into())) })
        .err_handler(|err: RouteError| async move {
            let api_err = err.downcast::<ApiError>().unwrap();
            match api_err.as_ref() {
                ApiError::Generic(s) => Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::from(s.to_string()))
                    .unwrap(),
            }
        })
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

    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let resp = into_text(resp.into_body()).await;
    assert_eq!(resp, RESPONSE_TEXT.to_owned());
    serve.shutdown();
}

#[tokio::test]
async fn can_handle_pre_middleware_errors() {
    struct State {}
    #[derive(Clone)]
    struct Ctx(i32);

    let state = State {};

    // If pre middleware fails, then `data` and `req.context` should
    // propagate to the error handler and post middleware. The route
    // handler should not be executed.
    let router: Router<Body, routerify::Error> = Router::builder()
        .data(state)
        .middleware(Middleware::pre(|req| async move {
            req.set_context(Ctx(42));
            Err(routerify::Error::new("Error!"))
        }))
        .err_handler_with_info(|err, req_info| async move {
            let _ctx = req_info.context::<Ctx>().expect("No Ctx");
            let _state = req_info.data::<State>().expect("No state");
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(err.to_string()))
                .unwrap()
        })
        .middleware(Middleware::post_with_info(|resp, req_info| async move {
            let _ctx = req_info.context::<Ctx>().expect("No Ctx");
            let _state = req_info.data::<State>().expect("No state");
            Ok(resp)
        }))
        .get("/", |_| async { panic!("should not be executed") })
        .build()
        .unwrap();

    let serve = serve(router).await;
    let _ = Client::new()
        .request(
            Request::builder()
                .method("GET")
                .uri(format!("http://{}", serve.addr()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    serve.shutdown();
}
