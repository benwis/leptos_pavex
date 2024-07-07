use crate::extend_response::ExtendResponse;
use crate::pavex_helpers::AppFunction;
use crate::response_options::ResponseOptions;
use crate::stream::{LeptosPavexStream, PavexStream};
use bytes::Bytes;
use futures::{Stream, StreamExt};
use leptos::nonce::use_nonce;
use leptos::server_fn::error::{
    ServerFnError, ServerFnErrorErr, ServerFnErrorSerde, SERVER_FN_ERROR_HEADER,
};
use leptos::server_fn::response::Res;
use leptos_integration_utils::{BoxedFnOnce, PinnedFuture, PinnedStream};
use pavex::http::header::CONTENT_TYPE;
use pavex::http::HeaderValue;
use pavex::http::{HeaderMap, HeaderName, StatusCode};
use pavex::response::Response;
use reactive_graph::owner::{Owner, Sandboxed};
use std::pin::Pin;
use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

/// This is here because the orphan rule does not allow us to implement foreign traits on a foreign type
pub struct PavexResponse(pub Response);

/// Functions used by leptos_pavex to manipulate Responses and make integration easier.
impl ExtendResponse for PavexResponse {
    type ResponseOptions = ResponseOptions;

    fn from_stream(stream: impl Stream<Item = String> + Send + 'static) -> Self {
        let stream = stream.map(|chunk| Ok(chunk) as Result<String, std::io::Error>);

        let lp_stream = LeptosPavexStream { inner: stream };
        PavexResponse(Response::ok().set_raw_body(lp_stream))
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
            headers.insert(CONTENT_TYPE, HeaderValue::from_str(content_type).unwrap());
        }
    }
}
/// Build an HTML stream for the response, returning it and the current reactive Owner
pub fn build_response(
    app_fn: AppFunction,
    additional_context: impl FnOnce() + Send + 'static,
    stream_builder: fn(
        AppFunction,
        BoxedFnOnce<PinnedStream<String>>,
    ) -> PinnedFuture<PinnedStream<String>>,
) -> (Owner, PinnedFuture<PinnedStream<String>>) {
    let Some(owner) = Owner::current() else {
        panic!("Failed to get Owner for components!");
    };
    let stream = Box::pin(Sandboxed::new({
        let owner = owner.clone();
        async move {
            let stream = owner.with(|| {
                additional_context();

                // run app
                let app = app_fn;

                let nonce = use_nonce()
                    .as_ref()
                    .map(|nonce| format!(" nonce=\"{nonce}\""))
                    .unwrap_or_default();

                let shared_context = Owner::current_shared_context().unwrap();

                let chunks = Box::new({
                    let shared_context = shared_context.clone();
                    move || {
                        Box::pin(shared_context.pending_data().unwrap().map(
                            move |chunk| {
                                format!("<script{nonce}>{chunk}</script>")
                            },
                        ))
                            as Pin<Box<dyn Stream<Item = String> + Send>>
                    }
                });

                // convert app to appropriate response type
                // and chain the app stream, followed by chunks
                // in theory, we could select here, and intersperse them
                // the problem is that during the DOM walk, that would be mean random <script> tags
                // interspersed where we expect other children
                //
                // we also don't actually start hydrating until after the whole stream is complete,
                // so it's not useful to send those scripts down earlier.
                stream_builder(app, chunks)
            });

            stream.await
        }
    }));
    (owner, stream)
}



impl<CustErr> Res<CustErr> for PavexResponse
where
    CustErr: Send + Sync + Debug + FromStr + Display + 'static,
{
    fn try_from_string(content_type: &str, data: String) -> Result<Self, ServerFnError<CustErr>> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, content_type.parse().unwrap());

        let mut res = Response::ok().set_typed_body(data);

        *res.headers_mut() = headers;

        Ok(PavexResponse(res))
    }

    fn try_from_bytes(content_type: &str, data: Bytes) -> Result<Self, ServerFnError<CustErr>> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, content_type.parse().unwrap());
        let mut res = Response::ok().set_typed_body(data);
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

        let mapped_stream = data.map(|n| {
            n.map_err(ServerFnErrorErr::from)
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
        });

        let stream = PavexStream {
            inner: mapped_stream,
        };

        let mut res = Response::ok().set_raw_body(stream);
        *res.headers_mut() = headers;

        Ok(PavexResponse(res))
    }

    fn error_response(path: &str, err: &ServerFnError<CustErr>) -> Self {
        let res = Response::new(StatusCode::INTERNAL_SERVER_ERROR)
            .insert_header(
                HeaderName::from_static(SERVER_FN_ERROR_HEADER),
                HeaderValue::from_str(path).unwrap(),
            )
            .set_typed_body(err.ser().unwrap_or_else(|_| err.to_string()));
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
