use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct RouteParams(HashMap<String, String>);

impl RouteParams {
    pub fn new() -> RouteParams {
        RouteParams(HashMap::new())
    }

    pub fn with_capacity(capacity: usize) -> RouteParams {
        RouteParams(HashMap::with_capacity(capacity))
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

    pub fn extend(&mut self, other_route_params: RouteParams) {
        other_route_params.0.into_iter().for_each(|(key, val)| {
            self.set(key, val);
        })
    }
}
