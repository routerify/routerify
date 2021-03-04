# Examples of using Routerify

These examples show of how to do common tasks using `routerify`.

Please visit: [Docs](https://docs.rs/routerify) for an exhaustive documentation.

Run an example:

```sh
 cargo run --example example_name
```

* [`simple_example`](simple_example.rs) - A basic example using `Routerify`.

* [`error_handling`](error_handling.rs) - Shows how to handle any error in `Routerify`.

* [`error_handling_with_request_info`](error_handling_with_request_info.rs) - Shows how to handle any error in `Routerify` based on the request information e.g. headers, method, uri etc.

* [`handle_404_pages`](handle_404_pages.rs) - An example on how to handle any non-existent pages.

* [`middleware`](middleware.rs) - Shows how to use and define a pre middleware and a post middleware.

* [`share_data_and_state`](share_data_and_state.rs) - Shows how to share app data and state across route handlers, middlewares and the error handler.

* [`route_parameters`](route_parameters.rs) - An example on how to use route parameters and how to extract them.

* [`scoped_router`](scoped_router.rs) - Shows how to write modular routing logic by mounting a router on another router.

* [`request_duration`](request_duration.rs) - Shows how to measure the duration of a request using per request context and middleware.
