use std::sync::Arc;
use pavex::http::{StatusCode, HeaderMap, HeaderValue, HeaderName};
use parking_lot::RwLock;

#[derive(Clone, Debug, Default)]
pub struct ResponseOptions(pub Arc<RwLock<ResponseParts>>);

impl ResponseOptions {

    /// A simpler way to overwrite the contents of `ResponseOptions` with a new `ResponseParts`.
    pub fn overwrite(&self, parts: ResponseParts) {
        let mut writable = self.0.write();
        *writable = parts
    }
    /// Get the status of the returned Response.
    pub fn status(&self) -> Option<StatusCode> {
        let readable = self.0.read();
        let res_parts = readable;
        res_parts.status
    }
    /// Get the headers of the returned Response.
    pub fn headers(&self) -> HeaderMap {
        let readable = self.0.read();
        let res_parts = readable;
        res_parts.headers.clone()
        }
    /// Set the status of the returned Response.
    pub fn set_status(&self, status: StatusCode) {
        let mut writeable = self.0.write();
        let res_parts = &mut *writeable;
        res_parts.status = Some(status);
    }
    /// Insert a header, overwriting any previous value with the same key.
    pub fn insert_header(&self, key: HeaderName, value: HeaderValue) {
        let mut writeable = self.0.write();
        let res_parts = &mut *writeable;
        res_parts.headers.insert(key, value);
    }
    /// Append a header, leaving any header with the same key intact.
    pub fn append_header(&self, key: HeaderName, value: HeaderValue) {
        let mut writeable = self.0.write();
        let res_parts = &mut *writeable;
        res_parts.headers.append(key, value);
    }
}

#[derive(Debug)]
struct ResponseParts {
    pub status: Option<StatusCode>,
    pub headers: HeaderMap,
}

impl Default for ResponseParts {
    fn default() -> Self {
        let mut headers = HeaderMap::new();
        headers
            .append(
                "content-type",
                HeaderValue::from_str("text/html").unwrap(),
            );
        Self {
            status: Default::default(),
            headers,
        }
    }
}

impl ResponseParts {
    pub fn default_without_headers() -> Self {
        Self {
            status: Default::default(),
            headers: HeaderMap::new(),
        }
    }
    /// Insert a header, overwriting any previous value with the same key
    pub fn insert_header(&mut self, key: HeaderName, value: HeaderValue) {
        self.headers.insert(key, value);
    }
    /// Append a header, leaving any header with the same key intact
    pub fn append_header(&mut self, key: HeaderName, value: HeaderValue) {
        self.headers.append(key, value);
    }
}