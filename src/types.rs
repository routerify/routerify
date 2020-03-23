use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct RequestData {
  path_params: PathParams,
}

impl RequestData {
  pub fn new(path_params: PathParams) -> Self {
    RequestData { path_params }
  }

  pub fn path_params(&self) -> &PathParams {
    &self.path_params
  }
}

#[derive(Debug, Clone)]
pub struct PathParams(HashMap<String, String>);

impl PathParams {
  pub fn new() -> Self {
    PathParams(HashMap::new())
  }

  pub fn with_capacity(capacity: usize) -> Self {
    PathParams(HashMap::with_capacity(capacity))
  }

  pub fn set<K: Into<String>, V: Into<String>>(&mut self, key: K, val: V) {
    self.0.insert(key.into(), val.into());
  }

  pub fn get(&self, key: &String) -> Option<&String> {
    self.0.get(key)
  }

  pub fn has(&self, key: &String) -> bool {
    self.0.contains_key(key)
  }
}
