use crate::{pavex_helpers::AppFunction, response::build_response};
use futures::{stream::once, Future, Stream, StreamExt};
use leptos_integration_utils::{BoxedFnOnce, PinnedFuture, PinnedStream};
use leptos_meta:: ServerMetaContextOutput;
use reactive_graph::owner::Sandboxed;

/// A Pavex specific version of the ExtendResponse trait from leptos_integration_utils,
/// due to our need to take in a sized AppFunction instead of the typical closure
pub trait ExtendResponse: Sized {
    type ResponseOptions: Send;

    fn from_stream(stream: impl Stream<Item = String> + Send + 'static) -> Self;

    fn extend_response(&mut self, opt: &Self::ResponseOptions);

    fn set_default_content_type(&mut self, content_type: &str);

    fn from_app(
        app_fn: AppFunction,
        meta_context: ServerMetaContextOutput,
        additional_context: impl FnOnce() + Send + 'static,
        res_options: Self::ResponseOptions,
        stream_builder: fn(
            AppFunction,
            BoxedFnOnce<PinnedStream<String>>,
        ) -> PinnedFuture<PinnedStream<String>>,
    ) -> impl Future<Output = Self> + Send {
        async move {
            let (owner, stream) = build_response(app_fn, additional_context, stream_builder);
            let stream = stream.await.ready_chunks(32).map(|n| n.join(""));

            let sc = owner.shared_context().unwrap();
            while let Some(pending) = sc.await_deferred() {
                pending.await;
            }

            let mut stream =
                Box::pin(meta_context.inject_meta_context(stream).await);
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
