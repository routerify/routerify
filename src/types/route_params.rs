use std::collections::HashMap;

/// Represents a map of the route parameters using the name of the parameter specified in the path as their respective keys.
///
/// Please refer to the [Route Parameters](./index.html#route-parameters) section for more info.
///
/// **Note:** This type shouldn't be created directly. It will be populated into the `req` object of the route handler and
/// can be accessed as `req.params()`.
#[derive(Debug, Clone)]
pub struct RouteParams(HashMap<String, String>);

impl RouteParams {
    /// Creates an empty route parameters map.
    pub fn new() -> RouteParams {
        RouteParams(HashMap::new())
    }

    /// Creates an empty route parameters map with the specified capacity.
    pub fn with_capacity(capacity: usize) -> RouteParams {
        RouteParams(HashMap::with_capacity(capacity))
    }

    /// Sets a new parameter entry with the specified key and the value.
    pub fn set<N: Into<String>, V: Into<String>>(&mut self, param_name: N, param_val: V) {
        self.0.insert(param_name.into(), param_val.into());
    }

    /// Returns the route parameter value mapped with the specified key.
    ///
    /// # Examples
    ///
    /// ```
    /// use routerify::{Router, RouteParams};
    /// use routerify::ext::RequestExt;
    /// use hyper::{Response, Body};
    /// # use std::convert::Infallible;
    ///
    /// # fn run() -> Router<Body, Infallible> {
    /// let router = Router::builder()
    ///     .get("/users/:userName/books/:bookName", |req| async move {
    ///         let params: &RouteParams = req.params();
    ///         
    ///         let user_name = params.get("userName").unwrap();
    ///         let book_name = params.get("bookName").unwrap();
    ///
    ///         Ok(Response::new(Body::from(format!("Username: {}, Book Name: {}", user_name, book_name))))
    ///      })
    ///      .build()
    ///      .unwrap();
    /// # router
    /// # }
    /// # run();
    /// ```
    pub fn get<N: Into<String>>(&self, param_name: N) -> Option<&String> {
        self.0.get(&param_name.into())
    }

    /// Checks if a route parameter exists.
    ///
    /// # Examples
    ///
    /// ```
    /// use routerify::{Router, RouteParams};
    /// use routerify::ext::RequestExt;
    /// use hyper::{Response, Body};
    /// # use std::convert::Infallible;
    ///
    /// # fn run() -> Router<Body, Infallible> {
    /// let router = Router::builder()
    ///     .get("/users/:userName", |req| async move {
    ///         let params: &RouteParams = req.params();
    ///         
    ///         if params.has("userName") {
    ///             Ok(Response::new(Body::from(params.get("userName").unwrap().to_string())))
    ///         } else {
    ///             Ok(Response::new(Body::from("username is not provided")))
    ///         }
    ///      })
    ///      .build()
    ///      .unwrap();
    /// # router
    /// # }
    /// # run();
    /// ```
    pub fn has<N: Into<String>>(&self, param_name: N) -> bool {
        self.0.contains_key(&param_name.into())
    }

    /// Returns the length of the route parameters.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns an [`Iterator`](https://doc.rust-lang.org/std/iter/trait.Iterator.html) over the parameter names.
    pub fn params_names(&self) -> impl Iterator<Item = &String> {
        self.0.keys()
    }

    /// Returns an [`Iterator`](https://doc.rust-lang.org/std/iter/trait.Iterator.html) over the parameter entries
    /// as `(parameter_name: &String, parameter_value:  &String)`.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &String)> {
        self.0.iter()
    }

    /// Extends the current parameters map with other one.
    pub fn extend(&mut self, other_route_params: RouteParams) {
        other_route_params.0.into_iter().for_each(|(key, val)| {
            self.set(key, val);
        })
    }
}
