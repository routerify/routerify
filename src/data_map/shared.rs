use crate::data_map::DataMap;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub(crate) struct SharedDataMap {
    pub(crate) inner: Arc<DataMap>,
}

impl SharedDataMap {
    pub fn new(data_map: Arc<DataMap>) -> SharedDataMap {
        SharedDataMap { inner: data_map }
    }
}
