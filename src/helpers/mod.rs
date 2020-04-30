use crate::types::RequestMeta;
use http::Extensions;

pub(crate) fn update_req_meta_in_extensions(ext: &mut Extensions, new_req_meta: RequestMeta) {
    if let Some(existing_req_meta) = ext.get_mut::<RequestMeta>() {
        existing_req_meta.extend(new_req_meta);
    } else {
        ext.insert(new_req_meta);
    }
}
