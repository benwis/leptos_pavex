use bytes::Bytes;
use futures::{Stream, StreamExt};
use leptos::server_fn::error::{
    ServerFnError, ServerFnErrorErr, ServerFnErrorSerde, SERVER_FN_ERROR_HEADER,
};
use leptos::server_fn::response::Res;
use pavex::http::header::CONTENT_TYPE;
use pavex::http::{HeaderMap, HeaderName, StatusCode};
use pavex::response::{Response, ResponseHead, ResponseBody};
use pavex::http::{HeaderValue, header::SERVER};
use std::pin::Pin;
use std::{
    fmt::{Debug, Display},
    str::FromStr,
};
use leptos_integration_utils::ExtendResponse;
use pavex::request::body::BufferedBody;
use crate::ResponseOptions;

/// This is here because the orphan rule does not allow us to implement it on IncomingRequest with
/// the generic error. So we have to wrap it to make it happy
pub struct PavexResponse(pub Response);

// impl ExtendResponse for PavexResponse {
//     type ResponseOptions = ResponseOptions;
//
//     fn from_stream(
//         stream: impl Stream<Item = String> + Send + 'static,
//     ) -> Self {
//         PavexResponse(
//             Response::ok().set_typed_body(stream.map(|chunk| Ok(chunk) as Result<String, std::io::Error>))
//         )
//     }
//
//     fn extend_response(&mut self, res_options: &Self::ResponseOptions) {
//         let mut res_options_raw = res_options;
//         if let Some(res_options) = res_options.status(){
//             self.0.set_status()
//         }
//         // *res_options.headers.
//         // if let Ok(status) = *res_options {
//         //     *self.0.status_mut() = status;
//         // }
//         // self.0
//         //     .headers_mut()
//         //     .extend(std::mem::take(&mut res_options.headers));
//     }
//
//     fn set_default_content_type(&mut self, content_type: &str) {
//         let headers = self.0.headers_mut();
//         if !headers.contains_key(CONTENT_TYPE) {
//             // Set the Content Type headers on all responses. This makes Firefox show the page source
//             // without complaining
//             headers.insert(
//                 CONTENT_TYPE,
//                 HeaderValue::from_str(content_type).unwrap(),
//             );
//         }
//     }
// }

/// We can either return a fairly simple Box type for normal bodies or a Stream for Streaming
/// server functions
pub enum PavexBody {
    Plain(BufferedBody),
    Streaming(Pin<Box<dyn Stream<Item = Result<Bytes, Box<dyn std::error::Error>>> + Send>>),
}

impl<CustErr> Res<CustErr> for PavexResponse
where
    CustErr: Send + Sync + Debug + FromStr + Display + 'static,
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
        let body = data.map(|n| {
            n.map_err(ServerFnErrorErr::from)
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
        });

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, content_type.parse().unwrap());

        let mut res = Response::ok()
        .set_typed_body(data.clone());
        *res.headers_mut() = headers;

        Ok(PavexResponse(res))
    }

    fn error_response(path: &str, err: &ServerFnError<CustErr>) -> Self {
        let res = Response::new(StatusCode::INTERNAL_SERVER_ERROR)
            .insert_header(HeaderName::from(SERVER_FN_ERROR_HEADER), path.into())
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