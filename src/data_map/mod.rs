use http::Extensions;
use std::sync::Arc;

#[derive(Debug)]
pub(crate) struct DataMap {
    inner: Extensions,
}

impl DataMap {
    pub fn new() -> DataMap {
        DataMap {
            inner: Extensions::new(),
        }
    }

    pub fn insert<T: Send + Sync + 'static>(&mut self, val: T) {
        self.inner.insert(val);
    }

    pub fn get<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.inner.get::<T>()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct SharedDataMap {
    pub(crate) inner: Arc<DataMap>,
}

impl SharedDataMap {
    pub fn new(data_map: Arc<DataMap>) -> SharedDataMap {
        SharedDataMap { inner: data_map }
    }
}
