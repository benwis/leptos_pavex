use std::sync::{Arc, RwLock};
use pavex::http::{StatusCode, HeaderMap, HeaderValue, HeaderName};


#[derive(Clone, Debug, Default)]
pub struct ResponseOptions(Arc<RwLock<ResponseParts>>);

impl ResponseOptions {

    /// A simpler way to overwrite the contents of `ResponseOptions` with a new `ResponseParts`.
    pub fn overwrite(&self, parts: ResponseParts) {
        let mut writable = self.0.write();
        *writable = parts
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
    pub(crate) status: Option<StatusCode>,
    headers: HeaderMap,
}

impl Default for ResponseParts {
    fn default() -> Self {
        let mut headers = HeaderMap::new();
        headers
            .append(
                "content-type",
                HeaderValue::from("text/html"),
            )
            .expect("Failed to append headers");
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