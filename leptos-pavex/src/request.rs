use bytes::Bytes;
use futures::{Stream, StreamExt};
use leptos::server_fn::{error::ServerFnError, request::Req};
use std::borrow::Cow;
use pavex::request::body::{BodySizeLimit, BufferedBody, RawIncomingBody};
use pavex::request::RequestHead;
use pavex::response::body::raw::RawBody;

/// This is here because the orphan rule does not allow us to implement it on IncomingRequest with
/// the generic error. So we have to wrap it to make it happy
#[derive(Debug)]
pub struct PavexRequest {
    pub head: RequestHead,
    pub body: RawIncomingBody,
}
impl PavexRequest {
    pub fn new_from_req(head: RequestHead, body: RawIncomingBody) -> Self {
        Self {
        head,
        body,
        }
    }
}

impl<CustErr> Req<CustErr> for PavexRequest
where
    CustErr: 'static,
{
    fn as_query(&self) -> Option<&str> {
        Some(self.head.target.to_string().as_ref())
    }

    fn to_content_type(&self) -> Option<Cow<'_, str>> {
        self.head
            .headers
            .get("Content-Type")
            .map(|h| h.to_str()).ok()

    }

    fn accepts(&self) -> Option<Cow<'_, str>> {
        self.head
            .headers
            .get("Accept")
            .map(|h| h.to_str()).ok()

    }

    fn referer(&self) -> Option<Cow<'_, str>> {
        self
            .head
            .headers
            .get("Referer")
            .map(|h| h.to_str()).ok()

    }

    async fn try_into_bytes(self) -> Result<Bytes, ServerFnError<CustErr>> {
        let buf = BufferedBody::extract(&self.head,self.body, BodySizeLimit::Disabled)
            .await
            .map_err(|e| ServerFnError::Deserialization(e.to_string()))?;
        Ok(buf.bytes)
    }

    async fn try_into_string(self) -> Result<String, ServerFnError<CustErr>> {
        let buf = BufferedBody::extract(&self.head,self.body, BodySizeLimit::Disabled)
            .await
            .map_err(|e| ServerFnError::Deserialization(e.to_string()))?;
        String::from_utf8(Vec::from(buf.bytes)).map_err(|e| ServerFnError::Deserialization(e.to_string()))
    }

    fn try_into_stream(
        self,
    ) -> Result<
        impl Stream<Item = Result<Bytes, ServerFnError>> + Send + 'static,
        ServerFnError<CustErr>,
    > {
        Ok(self.body.poll_frame().map(|chunk| {
            chunk
                .flatten(|c| Bytes::copy_from_slice(&c))
        }))
    }
}
