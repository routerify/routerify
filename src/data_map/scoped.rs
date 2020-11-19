use crate::data_map::{DataMap, SharedDataMap};
use crate::regex_generator::generate_exact_match_regex;
use regex::Regex;
use std::fmt::{self, Debug, Formatter};
use std::sync::Arc;

pub(crate) struct ScopedDataMap {
    pub(crate) path: String,
    pub(crate) regex: Regex,
    // Make it an option so that when a router is used to scope in another router,
    // It can be extracted out by 'opt.take()' without taking the whole router's ownership.
    pub(crate) data_map: Option<Arc<DataMap>>,
}

impl ScopedDataMap {
    pub fn new<P: Into<String>>(path: P, data_map: Arc<DataMap>) -> crate::Result<ScopedDataMap> {
        let path = path.into();
        let (re, _) = generate_exact_match_regex(path.as_str())?;

        Ok(ScopedDataMap {
            path,
            regex: re,
            data_map: Some(data_map),
        })
    }

    pub fn clone_data_map(&self) -> SharedDataMap {
        SharedDataMap::new(
            self.data_map
                .as_ref()
                .expect("The data map MUST NOT be `None` in this case")
                .clone(),
        )
    }
}

impl Debug for ScopedDataMap {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{{ path: {:?}, regex: {:?} }}", self.path, self.regex)
    }
}
