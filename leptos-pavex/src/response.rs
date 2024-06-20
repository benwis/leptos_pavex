use bytes::Bytes;
use futures::{Stream, StreamExt};
use leptos::nonce::use_nonce;
use leptos::server_fn::error::{
    ServerFnError, ServerFnErrorErr, ServerFnErrorSerde, SERVER_FN_ERROR_HEADER,
};
use leptos::server_fn::response::Res;
use leptos_integration_utils::{PinnedFuture, PinnedStream, BoxedFnOnce};
use leptos_meta::ServerMetaContext;
use pavex::http::header::CONTENT_TYPE;
use pavex::http::{HeaderMap, HeaderName, StatusCode};
use pavex::response::Response;
use pavex::http::HeaderValue;
use reactive_graph::owner::{Owner, Sandboxed};
use std::error::Error;
use std::pin::Pin;
use std::{
    fmt::{Debug, Display},
    str::FromStr,
    future::Future,
};
use crate::extend_response::ExtendResponse;
use crate::pavex_helpers::AppFunction;
use crate::response_options::ResponseOptions;
use crate::stream::{LeptosPavexStream, PavexStream};
use futures_util::stream::once;

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

    fn from_app(
        app_fn: AppFunction,
        meta_context: ServerMetaContext,
        additional_context: impl FnOnce() + Send + 'static,
        res_options: Self::ResponseOptions,
        stream_builder: fn(
            AppFunction,
            BoxedFnOnce<PinnedStream<String>>,
        ) -> PinnedFuture<PinnedStream<String>>,
    ) -> impl Future<Output = Self> + Send
    {
        async move {
            let (owner, stream) = build_response(
                app_fn,
                meta_context,
                additional_context,
                stream_builder,
            );
            let mut stream = stream.await;

            // wait for the first chunk of the stream, then set the status and headers
            let first_chunk = stream.next().await.unwrap_or_default();

            let mut res = Self::from_stream(Sandboxed::new(
                once(async move { first_chunk })
                    .chain(stream)
                    // drop the owner, cleaning up the reactive runtime,
                    // once the stream is over
                    .chain(once(async move {
                        drop(owner);
                        Default::default()
                    })),
            ));

            res.extend_response(&res_options);

            // Set the Content Type headers on all responses. This makes Firefox show the page source
            // without complaining
            res.set_default_content_type("text/html; charset=utf-8");

            res
        }
    }
}

pub fn build_response(
    app_fn: AppFunction,
    meta_context: ServerMetaContext,
    additional_context: impl FnOnce() + Send + 'static,
    stream_builder: fn(
        AppFunction,
        BoxedFnOnce<PinnedStream<String>>,
    ) -> PinnedFuture<PinnedStream<String>>,
) -> (Owner, PinnedFuture<PinnedStream<String>>)
{
    let Some(owner) = Owner::current() else{
        panic!("Failed to get Owner for components!");
    };
    let stream = Box::pin(Sandboxed::new({
        let owner = owner.clone();
        async move {
            let stream = owner
                .with(|| {
                    additional_context();

                    // // run app
                    // let app: leptos::tachys::view::any_view::AnyView<leptos::prelude::Dom> = app_fn.inner();

                    let nonce = use_nonce()
                        .as_ref()
                        .map(|nonce| format!(" nonce=\"{nonce}\""))
                        .unwrap_or_default();

                    let shared_context =
                        Owner::current_shared_context().unwrap();
                    let chunks = Box::new(move || {
                        Box::pin(shared_context.pending_data().unwrap().map(
                            move |chunk| {
                                format!("<script{nonce}>{chunk}</script>")
                            },
                        ))
                            as Pin<Box<dyn Stream<Item = String> + Send>>
                    });

                    // convert app to appropriate response type
                    // and chain the app stream, followed by chunks
                    // in theory, we could select here, and intersperse them
                    // the problem is that during the DOM walk, that would be mean random <script> tags
                    // interspersed where we expect other children
                    //
                    // we also don't actually start hydrating until after the whole stream is complete,
                    // so it's not useful to send those scripts down earlier.
                    stream_builder(app_fn, chunks)
                })
                .await;
            Box::pin(meta_context.inject_meta_context(stream).await)
                as PinnedStream<String>
        }
    }));
    (owner, stream)
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