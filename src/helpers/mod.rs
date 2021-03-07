use crate::types::RequestMeta;
use http::Extensions;
use percent_encoding::percent_decode_str;

pub(crate) fn update_req_meta_in_extensions(ext: &mut Extensions, new_req_meta: RequestMeta) {
    if let Some(existing_req_meta) = ext.get_mut::<RequestMeta>() {
        existing_req_meta.extend(new_req_meta);
    } else {
        ext.insert(new_req_meta);
    }
}

pub(crate) fn percent_decode_request_path(val: &str) -> crate::Result<String> {
    percent_decode_str(val)
        .decode_utf8()
        .map_err(Into::into)
        .map(|val| val.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_percent_decode_request_path() {
        let val = "/Alice%20John/do something";
        assert_eq!(
            percent_decode_request_path(val).unwrap(),
            "/Alice John/do something".to_owned()
        );

        let val = "Alice%20John";
        assert_eq!(percent_decode_request_path(val).unwrap(), "Alice John".to_owned());

        let val = "Go<>crazy";
        assert_eq!(percent_decode_request_path(val).unwrap(), "Go<>crazy".to_owned());

        let val = "go%crazy";
        assert_eq!(percent_decode_request_path(val).unwrap(), "go%crazy".to_owned());
    }
}
