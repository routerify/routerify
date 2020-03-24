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

  pub fn extend(&mut self, other_req_data: RequestData) {
    self.path_params.extend(other_req_data.path_params);
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

  pub fn set<N: Into<String>, V: Into<String>>(&mut self, param_name: N, param_val: V) {
    self.0.insert(param_name.into(), param_val.into());
  }

  pub fn get(&self, param_name: &String) -> Option<&String> {
    self.0.get(param_name)
  }

  pub fn has(&self, param_name: &String) -> bool {
    self.0.contains_key(param_name)
  }

  pub fn len(&self) -> usize {
    self.0.len()
  }

  pub fn params_names(&self) -> impl Iterator<Item = &String> {
    self.0.keys()
  }

  pub fn iter(&self) -> impl Iterator<Item = (&String, &String)> {
    self.0.iter()
  }

  pub fn extend(&mut self, other_path_params: PathParams) {
    other_path_params.0.into_iter().for_each(|(key, val)| {
      self.set(key, val);
    })
  }
}
