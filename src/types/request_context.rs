use crate::data_map::DataMap;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub(crate) struct RequestContext {
    // Strictly speaking, there should be no need to protect
    // the datamap because the context is per request,
    // thus no concurrent access.
    // However, the context must be mutable and shared
    // via req_info to be accessible from post middleware
    // and error handler. Which is only possible with
    // wrapping it in Arc and locking.
    inner: Arc<Mutex<DataMap>>,
}

impl RequestContext {
    pub(crate) fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(DataMap::new())),
        }
    }

    pub(crate) fn set<T: Send + Sync + Clone + 'static>(&self, val: T) {
        self.inner.lock().unwrap().insert(val);
    }

    pub(crate) fn get<T: Send + Sync + Clone + 'static>(&self) -> Option<T> {
        self.inner.lock().unwrap().get::<T>().map(|val| val.clone())
    }
}
