use crate::types::path_params::PathParams;
use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub(crate) struct RequestMeta {
    path_params: Option<PathParams>,
    remote_addr: Option<SocketAddr>,
}

impl RequestMeta {
    pub fn with_path_params(path_params: PathParams) -> RequestMeta {
        RequestMeta {
            path_params: Some(path_params),
            remote_addr: None,
        }
    }

    pub fn with_remote_addr(remote_addr: SocketAddr) -> RequestMeta {
        RequestMeta {
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

    pub fn extend(&mut self, other_req_meta: RequestMeta) {
        if let Some(other_ra) = other_req_meta.remote_addr {
            self.remote_addr = Some(other_ra)
        }

        if let Some(other_pm) = other_req_meta.path_params {
            if let Some(ref mut existing_pm) = self.path_params {
                existing_pm.extend(other_pm);
            } else {
                self.path_params = Some(other_pm);
            }
        }
    }
}
