// use spin_sdk::http::{conversions::IntoHeaders, IncomingRequest, Method, Scheme};
use pavex::http::{uri::Scheme, HeaderMap, Method};
use pavex::request::RequestHead;

/// A convenience type that's provided to the Leptos context containing info about the incoming Request
/// TODO: This may be able to be eliminated in favor of the native types
#[derive(Debug, Default, Clone)]
pub struct RequestParts {
    method: Method,
    scheme: Option<Scheme>,
    headers: HeaderMap,
}
impl RequestParts {
    pub fn new() -> Self {
        Self {
            method: Method::default(),
            headers: HeaderMap::default(),
            scheme: None,
        }
    }

    pub fn new_from_req(req: &RequestHead) -> Self {
        Self {
            method: req.method.clone(),
            scheme: req.target.scheme().cloned(),
            headers: req.headers.clone(),
        }
    }
    /// Get the Headers for the Request
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }
    /// Get the Method for the Request
    pub fn method(&self) -> &Method {
        &self.method
    }
    /// Get the Scheme for the Request
    pub fn scheme(&self) -> &Option<Scheme> {
        &self.scheme
    }
}
