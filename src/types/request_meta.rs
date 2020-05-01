use crate::types::route_params::RouteParams;
use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub(crate) struct RequestMeta {
    route_params: Option<RouteParams>,
    remote_addr: Option<SocketAddr>,
}

impl RequestMeta {
    pub fn with_route_params(route_params: RouteParams) -> RequestMeta {
        RequestMeta {
            route_params: Some(route_params),
            remote_addr: None,
        }
    }

    pub fn with_remote_addr(remote_addr: SocketAddr) -> RequestMeta {
        RequestMeta {
            route_params: None,
            remote_addr: Some(remote_addr),
        }
    }

    pub fn route_params(&self) -> Option<&RouteParams> {
        self.route_params.as_ref()
    }

    pub fn remote_addr(&self) -> Option<&SocketAddr> {
        self.remote_addr.as_ref()
    }

    pub fn extend(&mut self, other_req_meta: RequestMeta) {
        if let Some(other_ra) = other_req_meta.remote_addr {
            self.remote_addr = Some(other_ra)
        }

        if let Some(other_pm) = other_req_meta.route_params {
            if let Some(ref mut existing_pm) = self.route_params {
                existing_pm.extend(other_pm);
            } else {
                self.route_params = Some(other_pm);
            }
        }
    }
}
