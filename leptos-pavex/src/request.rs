use bytes::Bytes;
use futures::{Stream, StreamExt};
use http_body_util::BodyExt;
use leptos::server_fn::{error::ServerFnError, request::Req};
use pavex::request::body::{BodySizeLimit, BufferedBody, RawIncomingBody};
use pavex::request::RequestHead;
use std::borrow::Cow;
/// This is here because the orphan rule does not allow us to implement it on IncomingRequest with
/// the generic error. So we have to wrap it to make it happy
#[derive(Debug)]
pub struct PavexRequest {
    pub head: RequestHead,
    pub body: RawIncomingBody,
}
impl PavexRequest {
    pub fn new_from_req(head: RequestHead, body: RawIncomingBody) -> Self {
        Self { head, body }
    }
}

impl<CustErr> Req<CustErr> for PavexRequest
where
    CustErr: 'static,
{
    fn as_query(&self) -> Option<&str> {
        let target = &self.head.target;
        target.query()
    }

    fn to_content_type(&self) -> Option<Cow<'_, str>> {
        let headers = &self.head.headers;
        headers
            .get("Content-Type")
            .map(|h| String::from_utf8_lossy(h.as_bytes()))
    }

    fn accepts(&self) -> Option<Cow<'_, str>> {
        let headers = &self.head.headers;
        headers
            .get("Acceot")
            .map(|h| String::from_utf8_lossy(h.as_bytes()))
    }

    fn referer(&self) -> Option<Cow<'_, str>> {
        let headers = &self.head.headers;
        headers
            .get("Referer")
            .map(|h| String::from_utf8_lossy(h.as_bytes()))
    }

    async fn try_into_bytes(self) -> Result<Bytes, ServerFnError<CustErr>> {
        let buf = BufferedBody::extract(&self.head, self.body, BodySizeLimit::Disabled)
            .await
            .map_err(|e| ServerFnError::Deserialization(e.to_string()))?;
        Ok(buf.bytes)
    }

    async fn try_into_string(self) -> Result<String, ServerFnError<CustErr>> {
        let buf = BufferedBody::extract(&self.head, self.body, BodySizeLimit::Disabled)
            .await
            .map_err(|e| ServerFnError::Deserialization(e.to_string()))?;
        String::from_utf8(Vec::from(buf.bytes))
            .map_err(|e| ServerFnError::Deserialization(e.to_string()))
    }

    fn try_into_stream(
        self,
    ) -> Result<
        impl Stream<Item = Result<Bytes, ServerFnError>> + Send + 'static,
        ServerFnError<CustErr>,
    > {
        Ok(self
            .body
            .into_data_stream()
            .map(|chunk| chunk.map_err(|e| ServerFnError::Deserialization(e.to_string()))))
    }
}
