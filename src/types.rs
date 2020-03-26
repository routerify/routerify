use std::collections::HashMap;
use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub struct RequestData {
  path_params: Option<PathParams>,
  remote_addr: Option<SocketAddr>,
}

impl RequestData {
  pub fn with_path_params(path_params: PathParams) -> Self {
    RequestData {
      path_params: Some(path_params),
      remote_addr: None,
    }
  }

  pub fn with_remote_addr(remote_addr: SocketAddr) -> Self {
    RequestData {
      path_params: None,
      remote_addr: Some(remote_addr),
    }
  }

  pub fn path_params(&self) -> Option<&PathParams> {
    self.path_params.as_ref()
  }

  pub fn remote_addr(&self) -> Option<&SocketAddr> {
    self.remote_addr.as_ref()
  }

  pub fn extend(&mut self, other_req_data: RequestData) {
    if let Some(other_ra) = other_req_data.remote_addr {
      self.remote_addr = Some(other_ra)
    }

    if let Some(other_pm) = other_req_data.path_params {
      if let Some(ref mut existing_pm) = self.path_params {
        existing_pm.extend(other_pm);
      } else {
        self.path_params = Some(other_pm);
      }
    }
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
