use bytes::Bytes;
use futures::{Stream, StreamExt};
use leptos::server_fn::error::{
    ServerFnError, ServerFnErrorErr, ServerFnErrorSerde, SERVER_FN_ERROR_HEADER,
};
use leptos::server_fn::response::Res;
use leptos_integration_utils::ExtendResponse;
use pavex::http::header::CONTENT_TYPE;
use pavex::http::{HeaderMap, HeaderName, StatusCode};
use pavex::response::Response;
use pavex::http::HeaderValue;
use std::error::Error;
use std::{
    fmt::{Debug, Display},
    str::FromStr,
};
use crate::response_options::ResponseOptions;
use crate::stream::{LeptosPavexStream, PavexStream};

/// This is here because the orphan rule does not allow us to implement it on IncomingRequest with
/// the generic error. So we have to wrap it to make it happy
pub struct PavexResponse(pub Response);

impl ExtendResponse for PavexResponse {
    type ResponseOptions = ResponseOptions;

    fn from_stream(
        stream: impl Stream<Item = String> + Send + 'static,
    ) -> Self {
        let stream = stream.map(|chunk| Ok(chunk) as Result<String, std::io::Error>);

        let lp_stream = LeptosPavexStream{inner: stream};
        PavexResponse(
            Response::ok()
            .set_raw_body(lp_stream)
        )
    }

    fn extend_response(&mut self, res_options: &Self::ResponseOptions) {
        let mut res_options = res_options.0.write();
        if let Some(status) = res_options.status {
            *self.0.status_mut() = status;
        }
        self.0
            .headers_mut()
            .extend(std::mem::take(&mut res_options.headers));
    }

    fn set_default_content_type(&mut self, content_type: &str) {
        let headers = self.0.headers_mut();
        if !headers.contains_key(CONTENT_TYPE) {
            // Set the Content Type headers on all responses. This makes Firefox show the page source
            // without complaining
            headers.insert(
                CONTENT_TYPE,
                HeaderValue::from_str(content_type).unwrap(),
            );
        }
    }
}

impl<CustErr> Res<CustErr> for PavexResponse
where
    CustErr: Send + Sync + Debug + FromStr + Display + Error +  'static,
    ServerFnError<CustErr>: From<ServerFnErrorErr<CustErr>>,
{
    fn try_from_string(content_type: &str, data: String) -> Result<Self, ServerFnError<CustErr>> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, content_type.parse().unwrap());

        let mut res = Response::ok()
            .set_typed_body(data);

        *res.headers_mut() = headers;
           
        Ok(PavexResponse(res))
    }

    fn try_from_bytes(content_type: &str, data: Bytes) -> Result<Self, ServerFnError<CustErr>> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, content_type.parse().unwrap());
        let mut res = Response::ok()
            .set_typed_body(data);
            *res.headers_mut() = headers;

        Ok(PavexResponse(res))
    }

    fn try_from_stream(
        content_type: &str,
        data: impl Stream<Item = Result<Bytes, ServerFnError<CustErr>>> + Send + 'static,
    ) -> Result<Self, ServerFnError<CustErr>> {
        // let body = data.map(|n| {
        //     n.map_err(ServerFnErrorErr::from)
        //         .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
        // });

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, content_type.parse().unwrap());

        let stream = PavexStream{inner: data};

        let mut res = Response::ok()
        .set_raw_body(stream);
        *res.headers_mut() = headers;

        Ok(PavexResponse(res))
    }

    fn error_response(path: &str, err: &ServerFnError<CustErr>) -> Self {
        let res = Response::new(StatusCode::INTERNAL_SERVER_ERROR)
            .insert_header(HeaderName::from_static(SERVER_FN_ERROR_HEADER), HeaderValue::from_str(path).unwrap())
            .set_typed_body(
                err.ser().unwrap_or_else(|_| err.to_string()));
        PavexResponse(res)
    }

    fn redirect(&mut self, _path: &str) {
        //TODO: Enabling these seems to override location header
        // not sure what's causing that
        //let res_options = expect_context::<ResponseOptions>();
        //res_options.insert_header("Location", path);
        //res_options.set_status(302);
    }
}