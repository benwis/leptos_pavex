pub mod extend_response;
pub mod file_helpers;
pub mod leptos_routes;
pub mod pavex_helpers;
pub mod request;
#[allow(dead_code)]
pub mod request_parts;
pub mod response;
pub mod response_options;
pub mod route_generation;
pub mod server_fn;
pub mod stream;

use bytes::Bytes;
use extend_response::ExtendResponse;
use futures::stream::once;
use futures::{Stream, StreamExt};
use hydration_context::SsrSharedContext;
use leptos::server_fn::redirect::REDIRECT_HEADER;
use std::io;
use std::pin::Pin;
use std::sync::Arc;

use crate::request_parts::RequestParts;
use crate::response_options::ResponseOptions;
use leptos::prelude::{provide_context, use_context, Owner};
use leptos::tachys::view::RenderHtml;
use leptos_integration_utils::{BoxedFnOnce, PinnedFuture, PinnedStream};
use leptos_meta::ServerMetaContext;
use leptos_router::components::provide_server_redirect;
use leptos_router::location::RequestUrl;
use leptos_router::{PathSegment, RouteListing, SsrMode, StaticDataMap, StaticMode};
use pavex::http::header::{ACCEPT, LOCATION};
use pavex::http::uri::PathAndQuery;
use pavex::http::StatusCode;
use pavex::http::{HeaderName, HeaderValue};
use pavex::request::body::RawIncomingBody;
use pavex::request::path::MatchedPathPattern;
use pavex::request::RequestHead;
use pavex::response::Response;
use pavex_helpers::{AdditionalContextComponent, AppFunction, RouteAppFunction};
use reactive_graph::computed::ScopedFuture;
use response::PavexResponse;
use route_generation::RouteList;
/// Provides an easy way to redirect the user from within a server function. Mimicking the Remix `redirect()`,
/// it sets a StatusCode of 302 and a LOCATION header with the provided value.
/// If looking to redirect from the client, `leptos_router::use_navigate()` should be used instead
pub fn redirect(path: &str) {
    if let (Some(req), Some(res)) = (
        use_context::<RequestParts>(),
        use_context::<ResponseOptions>(),
    ) {
        // insert the Location header in any case
        res.insert_header(
            LOCATION,
            HeaderValue::from_str(path).expect("Failed to create HeaderValue"),
        );

        let accepts_html = req
            .headers()
            .get(ACCEPT)
            .and_then(|v| v.to_str().ok())
            .map(|v| v.contains("text/html"))
            .unwrap_or(false);
        if accepts_html {
            // if the request accepts text/html, it's a plain form request and needs
            // to have the 302 code set
            res.set_status(StatusCode::FOUND);
        } else {
            // otherwise, we sent it from the server fn client and actually don't want
            // to set a real redirect, as this will break the ability to return data
            // instead, set the REDIRECT_HEADER to indicate that the client should redirect
            res.insert_header(
                HeaderName::from_static(REDIRECT_HEADER),
                HeaderValue::from_str("").unwrap(),
            );
        }
    } else {
        tracing::warn!(
            "Couldn't retrieve either Parts or ResponseOptions while trying \
             to redirect()."
        );
    }
}
fn init_executor() {
    #[cfg(feature = "wasm")]
    let _ = any_spawner::Executor::init_wasm_bindgen();
    #[cfg(all(not(feature = "wasm"), feature = "default"))]
    let _ = any_spawner::Executor::init_tokio();
    #[cfg(all(not(feature = "wasm"), not(feature = "default")))]
    {
        eprintln!(
            "It appears you have set 'default-features = false' on \
             'leptos_pavex', but are not using the 'wasm' feature. Either \
             remove 'default-features = false' or, if you are running in a \
             JS-hosted WASM server environment, add the 'wasm' feature."
        );
    }
}

pub type PinnedHtmlStream = Pin<Box<dyn Stream<Item = io::Result<Bytes>> + Send>>;

/// Returns an Axum [Handler](axum::handler::Handler) that listens for a `GET` request and tries
/// to route it using [leptos_router], serving an HTML stream of your application.
///
/// This provides a [MetaContext] and a [RouterIntegrationContext] to app’s context before
/// rendering it, and includes any meta tags injected using [leptos_meta].
///
/// The HTML stream is rendered using [render_to_stream](leptos::ssr::render_to_stream), and
/// includes everything described in the documentation for that function.
///
/// This can then be set up at an appropriate route in your application:
/// ## Provided Context Types
/// This function always provides context values including the following types:
/// - [`Parts`]
/// - [`ResponseOptions`]
/// - [`ServerMetaContext`](leptos_meta::ServerMetaContext)
/// - [`RouterIntegrationContext`](leptos_router::RouterIntegrationContext)
#[tracing::instrument(level = "trace", fields(error), skip_all)]
pub async fn render_app_to_stream(
    req_head: RequestHead,
    req_body: RawIncomingBody,
    app_fn: AppFunction,
) -> Response {
    render_app_to_stream_with_context(req_head, req_body, app_fn).await
}

/// Returns an Axum [Handler](axum::handler::Handler) that listens for a `GET` request and tries
/// to route it using [leptos_router], serving an HTML stream of your application.
/// The difference between calling this and `render_app_to_stream_with_context()` is that this
/// one respects the `SsrMode` on each Route and thus requires `Vec<PavexRouteListing>` for route checking.
/// This is useful if you are using `.leptos_routes_with_handler()`
#[tracing::instrument(level = "trace", fields(error), skip_all)]
pub async fn render_route<IV>(
    paths: Vec<PavexRouteListing>,
    req_head: RequestHead,
    req_body: RawIncomingBody,
    matched_path: &MatchedPathPattern,
    context: AdditionalContextComponent,
    app_fn: AppFunction,
) -> Response {
    render_route_with_context(paths, req_head, req_body, matched_path, context, app_fn).await
}

/// Returns an Axum [Handler](axum::handler::Handler) that listens for a `GET` request and tries
/// to route it using [leptos_router], serving an in-order HTML stream of your application.
/// This stream will pause at each `<Suspense/>` node and wait for it to resolve before
/// sending down its HTML. The app will become interactive once it has fully loaded.
///
/// This provides a [MetaContext] and a [RouterIntegrationContext] to app’s context before
/// rendering it, and includes any meta tags injected using [leptos_meta].
///
/// The HTML stream is rendered using [render_to_stream_in_order], and includes everything described in
/// the documentation for that function.
///

/// ## Provided Context Types
/// This function always provides context values including the following types:
/// - [`Parts`]
/// - [`ResponseOptions`]
/// - [`ServerMetaContext`](leptos_meta::ServerMetaContext)
/// - [`RouterIntegrationContext`](leptos_router::RouterIntegrationContext)
#[tracing::instrument(level = "trace", fields(error), skip_all)]
pub async fn render_app_to_stream_in_order(
    req_head: RequestHead,
    req_body: RawIncomingBody,
    app_fn: AppFunction,
) -> Response {
    render_app_to_stream_in_order_with_context(req_head, req_body, app_fn).await
}

/// Returns an Axum [Handler](axum::handler::Handler) that listens for a `GET` request and tries
/// to route it using [leptos_router], serving an HTML stream of your application.
///
/// This version allows us to pass Axum State/Extension/Extractor or other infro from Axum or network
/// layers above Leptos itself. To use it, you'll need to write your own handler function that provides
/// the data to leptos in a closure. An example is below
/// ```ignore
/// async fn custom_handler(Path(id): Path<String>, Extension(options): Extension<Arc<LeptosOptions>>, req: PavexRequest) -> Response{
///     let handler = leptos_axum::render_app_to_stream_with_context((*options).clone(),
///     || {
///         provide_context(id.clone());
///     },
///     || view! { <TodoApp/> }
/// );
///     handler(req).await.into_response()
/// }
/// ```
/// Otherwise, this function is identical to [render_app_to_stream].
///
/// ## Provided Context Types
/// This function always provides context values including the following types:
/// - [`Parts`]
/// - [`ResponseOptions`]
/// - [`ServerMetaContext`](leptos_meta::ServerMetaContext)
/// - [`RouterIntegrationContext`](leptos_router::RouterIntegrationContext)
#[tracing::instrument(level = "trace", fields(error), skip_all)]
pub async fn render_app_to_stream_with_context(
    req_head: RequestHead,
    req_body: RawIncomingBody,
    app_fn: AppFunction,
) -> Response {
    render_app_to_stream_with_context_and_replace_blocks(req_head, req_body, app_fn, false).await
}
/// Returns an Axum [Handler](axum::handler::Handler) that listens for a `GET` request and tries
/// to route it using [leptos_router], serving an HTML stream of your application. It allows you
/// to pass in a context function with additional info to be made available to the app
/// The difference between calling this and `render_app_to_stream_with_context()` is that this
/// one respects the `SsrMode` on each Route, and thus requires `Vec<PavexRouteListing>` for route checking.
/// This is useful if you are using `.leptos_routes_with_handler()`.
#[tracing::instrument(level = "trace", fields(error), skip_all)]
pub async fn render_route_with_context(
    paths: Vec<PavexRouteListing>,
    req_head: RequestHead,
    req_body: RawIncomingBody,
    matched_path: &MatchedPathPattern,
    context: AdditionalContextComponent,
    app_fn: AppFunction,
) -> Response {
    // 1. Process route to match the values in routeListing
    let path = &matched_path.to_string();
    // 2. Find RouteListing in paths. This should probably be optimized, we probably don't want to
    // search for this every time
    let listing: &PavexRouteListing = paths
        .iter()
        .find(|r| r.path() == matched_path.inner())
        .unwrap_or_else(|| {
            panic!(
                "Failed to find the route {path} requested by the user. \
                    This suggests that the routing rules in the Router that \
                    call this handler needs to be edited!"
            )
        });
    // 3. Match listing mode against known, and choose function
    let owner = context.owner();
    match listing.mode() {
        SsrMode::OutOfOrder => {
            owner
                .with(|| {
                    ScopedFuture::new(render_app_to_stream_with_context(
                        req_head, req_body, app_fn,
                    ))
                })
                .await
        }
        SsrMode::PartiallyBlocked => {
            owner
                .with(|| {
                    ScopedFuture::new(render_app_to_stream_with_context_and_replace_blocks(
                        req_head, req_body, app_fn, true,
                    ))
                })
                .await
        }
        SsrMode::InOrder => {
            owner
                .with(|| {
                    ScopedFuture::new(render_app_to_stream_in_order_with_context(
                        req_head, req_body, app_fn,
                    ))
                })
                .await
        }
        SsrMode::Async => {
            owner
                .with(|| {
                    ScopedFuture::new(render_app_async_stream_with_context(
                        req_head, req_body, app_fn,
                    ))
                })
                .await
        }
    }
}

/// Returns an Axum [Handler](axum::handler::Handler) that listens for a `GET` request and tries
/// to route it using [leptos_router], serving an HTML stream of your application.
///
/// This version allows us to pass Axum State/Extension/Extractor or other info from Axum or network
/// layers above Leptos itself. To use it, you'll need to write your own handler function that provides
/// the data to leptos in a closure.
///
/// `replace_blocks` additionally lets you specify whether `<Suspense/>` fragments that read
/// from blocking resources should be retrojected into the HTML that's initially served, rather
/// than dynamically inserting them with JavaScript on the client. This means you will have
/// better support if JavaScript is not enabled, in exchange for a marginally slower response time.
///
/// Otherwise, this function is identical to [render_app_to_stream_with_context].
///
/// ## Provided Context Types
/// This function always provides context values including the following types:
/// - [`Parts`]
/// - [`ResponseOptions`]
/// - [`ServerMetaContext`](leptos_meta::ServerMetaContext)
/// - [`RouterIntegrationContext`](leptos_router::RouterIntegrationContext)
#[tracing::instrument(level = "trace", fields(error), skip_all)]
pub async fn render_app_to_stream_with_context_and_replace_blocks(
    req_head: RequestHead,
    req_body: RawIncomingBody,
    app_fn: AppFunction,
    replace_blocks: bool,
) -> Response {
    _ = replace_blocks; // TODO
    handle_response(req_head, req_body, app_fn, |app, chunks| {
        Box::pin(async move {
            Box::pin(app.inner().to_html_stream_out_of_order().chain(chunks()))
                as PinnedStream<String>
        })
    })
    .await
}

/// Returns an Axum [Handler](axum::handler::Handler) that listens for a `GET` request and tries
/// to route it using [leptos_router], serving an in-order HTML stream of your application.
/// This stream will pause at each `<Suspense/>` node and wait for it to resolve before
/// sending down its HTML. The app will become interactive once it has fully loaded.
///
/// This version allows us to pass Axum State/Extension/Extractor or other infro from Axum or network
/// layers above Leptos itself. To use it, you'll need to write your own handler function that provides
/// the data to leptos in a closure. An example is below
/// ```ignore
/// async fn custom_handler(Path(id): Path<String>, Extension(options): Extension<Arc<LeptosOptions>>, req: PavexRequest) -> Response{
///     let handler = leptos_axum::render_app_to_stream_in_order_with_context((*options).clone(),
///     move || {
///         provide_context(id.clone());
///     },
///     || view! { <TodoApp/> }
/// );
///     handler(req).await.into_response()
/// }
/// ```
/// Otherwise, this function is identical to [render_app_to_stream].
///
/// ## Provided Context Types
/// This function always provides context values including the following types:
/// - [`Parts`]
/// - [`ResponseOptions`]
/// - [`ServerMetaContext`](leptos_meta::ServerMetaContext)
/// - [`RouterIntegrationContext`](leptos_router::RouterIntegrationContext)
#[tracing::instrument(level = "trace", fields(error), skip_all)]
pub async fn render_app_to_stream_in_order_with_context(
    req_head: RequestHead,
    req_body: RawIncomingBody,
    app_fn: AppFunction,
) -> Response {
    handle_response(req_head, req_body, app_fn, |app, chunks| {
        Box::pin(async move {
            Box::pin(app.inner().to_html_stream_in_order().chain(chunks())) as PinnedStream<String>
        })
    })
    .await
}

async fn handle_response(
    req_head: RequestHead,
    _req_body: RawIncomingBody,
    app_fn: AppFunction,
    stream_builder: fn(
        AppFunction,
        BoxedFnOnce<PinnedStream<String>>,
    ) -> PinnedFuture<PinnedStream<String>>,
) -> Response {
    let res_options = ResponseOptions::default();
    let meta_context = ServerMetaContext::new();

    let additional_context = {
        let meta_context = meta_context.clone();
        let res_options = res_options.clone();
        move || {
            // Need to get the path and query string of the Request
            // For reasons that escape me, if the incoming URI protocol is https, it provides the absolute URI
            let path = req_head.target.path_and_query().unwrap().as_str();

            let full_path = format!("http://leptos.dev{path}");
            let req_parts = RequestParts::new_from_req(&req_head);
            provide_post_contexts(&full_path, &meta_context, req_parts, res_options.clone());
        }
    };

    let res = PavexResponse::from_app(
        app_fn,
        meta_context,
        additional_context,
        res_options,
        stream_builder,
    )
    .await;

    res.0
}

/// Provide Context one might want available to people in the additional context environment
#[tracing::instrument(level = "trace", fields(error), skip_all)]
pub fn provide_initial_contexts(req_head: &RequestHead, parts: RequestParts) {
    let path = req_head
        .target
        .path_and_query()
        .cloned()
        .unwrap_or(PathAndQuery::from_static("/"));
    provide_context(RequestUrl::new(&path.to_string()));
    provide_context(parts);
    provide_server_redirect(redirect);
    #[cfg(feature = "nonce")]
    leptos::nonce::provide_nonce();
}
// Makes sure the stuff that could be added to context earlier is set, and add the remaining stuff
#[tracing::instrument(level = "trace", fields(error), skip_all)]
fn provide_post_contexts(
    path: &str,
    meta_context: &ServerMetaContext,
    parts: RequestParts,
    default_res_options: ResponseOptions,
) {
    // These will be set if the Pavex user is adding their own context, otherwise we need to add them
    if use_context::<RequestUrl>().is_none() {
        provide_context(RequestUrl::new(path));
    }
    if use_context::<RequestParts>().is_none() {
        provide_context(parts);
    }
    provide_context(meta_context.clone());
    provide_context(default_res_options);
    provide_server_redirect(redirect);
    #[cfg(feature = "nonce")]
    leptos::nonce::provide_nonce();
}

/// Returns an Axum [Handler](axum::handler::Handler) that listens for a `GET` request and tries
/// to route it using [leptos_router], asynchronously rendering an HTML page after all
/// `async` [Resource](leptos::Resource)s have loaded.
///
/// The provides a [MetaContext] and a [RouterIntegrationContext] to app’s context before
/// rendering it, and includes any meta tags injected using [leptos_meta].
///
/// The HTML stream is rendered using [render_to_string_async], and includes everything described in
/// the documentation for that function.
/// ## Provided Context Types
/// This function always provides context values including the following types:
/// - [`Parts`]
/// - [`ResponseOptions`]
/// - [`ServerMetaContext`](leptos_meta::ServerMetaContext)
/// - [`RouterIntegrationContext`](leptos_router::RouterIntegrationContext)
#[tracing::instrument(level = "trace", fields(error), skip_all)]
pub async fn render_app_async(
    req_head: RequestHead,
    req_body: RawIncomingBody,
    app_fn: AppFunction,
) -> Response {
    render_app_async_with_context(req_head, req_body, app_fn).await
}

/// Returns an Axum [Handler](axum::handler::Handler) that listens for a `GET` request and tries
/// to route it using [leptos_router], asynchronously rendering an HTML page after all
/// `async` [Resource](leptos::Resource)s have loaded.
///
/// This version allows us to pass Axum State/Extension/Extractor or other infro from Axum or network
/// layers above Leptos itself. To use it, you'll need to write your own handler function that provides
/// the data to leptos in a closure. An example is below
/// ```ignore
/// async fn custom_handler(Path(id): Path<String>, Extension(options): Extension<Arc<LeptosOptions>>, req: PavexRequest) -> Response{
///     let handler = leptos_axum::render_app_async_with_context((*options).clone(),
///     move || {
///         provide_context(id.clone());
///     },
///     || view! { <TodoApp/> }
/// );
///     handler(req).await.into_response()
/// }
/// ```
/// Otherwise, this function is identical to [render_app_to_stream].
///
/// ## Provided Context Types
/// This function always provides context values including the following types:
/// - [`Parts`]
/// - [`ResponseOptions`]
/// - [`ServerMetaContext`](leptos_meta::ServerMetaContext)
/// - [`RouterIntegrationContext`](leptos_router::RouterIntegrationContext)
#[tracing::instrument(level = "trace", fields(error), skip_all)]
pub async fn render_app_async_stream_with_context(
    req_head: RequestHead,
    req_body: RawIncomingBody,
    app_fn: AppFunction,
) -> Response {
    handle_response(req_head, req_body, app_fn, |app, chunks| {
        Box::pin(async move {
            let app = app
                .inner()
                .to_html_stream_in_order()
                .collect::<String>()
                .await;
            let chunks = chunks();
            Box::pin(once(async move { app }).chain(chunks)) as PinnedStream<String>
        })
    })
    .await
}

/// Returns an Axum [Handler](axum::handler::Handler) that listens for a `GET` request and tries
/// to route it using [leptos_router], asynchronously rendering an HTML page after all
/// `async` [Resource](leptos::Resource)s have loaded.
///
/// This version allows us to pass Axum State/Extension/Extractor or other infro from Axum or network
/// layers above Leptos itself. To use it, you'll need to write your own handler function that provides
/// the data to leptos in a closure. An example is below
/// ```ignore
/// async fn custom_handler(Path(id): Path<String>, Extension(options): Extension<Arc<LeptosOptions>>, req: PavexRequest) -> Response{
///     let handler = leptos_axum::render_app_async_with_context((*options).clone(),
///     move || {
///         provide_context(id.clone());
///     },
///     || view! { <TodoApp/> }
/// );
///     handler(req).await.into_response()
/// }
/// ```
/// Otherwise, this function is identical to [render_app_to_stream].
///
/// ## Provided Context Types
/// This function always provides context values including the following types:
/// - [`Parts`]
/// - [`ResponseOptions`]
/// - [`ServerMetaContext`](leptos_meta::ServerMetaContext)
/// - [`RouterIntegrationContext`](leptos_router::RouterIntegrationContext)
#[tracing::instrument(level = "trace", fields(error), skip_all)]
pub async fn render_app_async_with_context(
    req_head: RequestHead,
    req_body: RawIncomingBody,
    app_fn: AppFunction,
) -> Response {
    handle_response(req_head, req_body, app_fn, |app, chunks| {
        Box::pin(async move {
            let app = app
                .inner()
                .to_html_stream_in_order()
                .collect::<String>()
                .await;
            let chunks = chunks();
            Box::pin(once(async move { app }).chain(chunks)) as PinnedStream<String>
        })
    })
    .await
}

/// Generates a list of all routes defined in Leptos's Router in your app. We can then use this to automatically
/// create routes in Axum's Router without having to use wildcard matching or fallbacks. Takes in your root app Element
/// as an argument, so it can walk your app tree. This version is tailored to generate Axum compatible paths.
#[tracing::instrument(level = "trace", fields(error), skip_all)]
pub fn generate_route_list(app_fn: RouteAppFunction) -> Vec<PavexRouteListing> {
    generate_route_list_with_exclusions_and_ssg(app_fn, None).0
}

/// Generates a list of all routes defined in Leptos's Router in your app. We can then use this to automatically
/// create routes in Axum's Router without having to use wildcard matching or fallbacks. Take in your root app Element
/// as an argument, so it can walk your app tree. This version is tailored to generate Axum compatible paths.
#[tracing::instrument(level = "trace", fields(error), skip_all)]
pub fn generate_route_list_with_ssg(
    app_fn: RouteAppFunction,
) -> (Vec<PavexRouteListing>, StaticDataMap) {
    generate_route_list_with_exclusions_and_ssg(app_fn, None)
}

/// Generates a list of all routes defined in Leptos's Router in your app. We can then use this to automatically
/// create routes in Axum's Router without having to use wildcard matching or fallbacks. Takes in your root app Element
/// as an argument, so it can walk you app tree. This version is tailored to generate Axum compatible paths. Adding excluded_routes
/// to this function will stop `.leptos_routes()` from generating a route for it, allowing a custom handler. These need to be in Axum path format
#[tracing::instrument(level = "trace", fields(error), skip_all)]
pub fn generate_route_list_with_exclusions(
    app_fn: RouteAppFunction,
    excluded_routes: Option<Vec<String>>,
) -> Vec<PavexRouteListing> {
    generate_route_list_with_exclusions_and_ssg(app_fn, excluded_routes).0
}

/// Generates a list of all routes defined in Leptos's Router in your app. We can then use this to automatically
/// create routes in Axum's Router without having to use wildcard matching or fallbacks. Takes in your root app Element
/// as an argument, so it can walk you app tree. This version is tailored to generate Axum compatible paths. Adding excluded_routes
/// to this function will stop `.leptos_routes()` from generating a route for it, allowing a custom handler. These need to be in Axum path format
#[tracing::instrument(level = "trace", fields(error), skip_all)]
pub fn generate_route_list_with_exclusions_and_ssg(
    app_fn: RouteAppFunction,
    excluded_routes: Option<Vec<String>>,
) -> (Vec<PavexRouteListing>, StaticDataMap) {
    generate_route_list_with_exclusions_and_ssg_and_context(app_fn, excluded_routes, || {})
}
#[derive(Clone, Debug, Default)]
/// A route that this application can serve.
pub struct PavexRouteListing {
    path: String,
    mode: SsrMode,
    methods: Vec<leptos_router::Method>,
    static_mode: Option<(StaticMode, StaticDataMap)>,
}

impl From<RouteListing> for PavexRouteListing {
    fn from(value: RouteListing) -> Self {
        let path = value.path().to_pavex_path();
        let path = if path.is_empty() {
            "/".to_string()
        } else {
            path
        };
        let mode = value.mode();
        let methods = value.methods().collect();
        let static_mode = value.into_static_parts();
        Self {
            path,
            mode,
            methods,
            static_mode,
        }
    }
}

impl PavexRouteListing {
    /// Create a route listing from its parts.
    pub fn new(
        path: String,
        mode: SsrMode,
        methods: impl IntoIterator<Item = leptos_router::Method>,
        static_mode: Option<(StaticMode, StaticDataMap)>,
    ) -> Self {
        Self {
            path,
            mode,
            methods: methods.into_iter().collect(),
            static_mode,
        }
    }

    /// The path this route handles.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// The rendering mode for this path.
    pub fn mode(&self) -> SsrMode {
        self.mode
    }

    /// The HTTP request methods this path can handle.
    pub fn methods(&self) -> impl Iterator<Item = leptos_router::Method> + '_ {
        self.methods.iter().copied()
    }

    /// Whether this route is statically rendered.
    #[inline(always)]
    pub fn static_mode(&self) -> Option<StaticMode> {
        self.static_mode.as_ref().map(|n| n.0)
    }
}

trait PavexPath {
    fn to_pavex_path(&self) -> String;
}

impl PavexPath for &[PathSegment] {
    fn to_pavex_path(&self) -> String {
        let mut path = String::new();
        for segment in self.iter() {
            // TODO trailing slash handling
            let raw = segment.as_raw_str();
            if !raw.is_empty() && !raw.starts_with('/') {
                path.push('/');
            }
            match segment {
                PathSegment::Static(s) => path.push_str(s),
                PathSegment::Param(s) => {
                    path.push(':');
                    path.push_str(s);
                }
                PathSegment::Splat(s) => {
                    path.push('*');
                    path.push_str(s);
                }
                PathSegment::Unit => {}
            }
        }
        path
    }
}

/// Generates a list of all routes defined in Leptos's Router in your app. We can then use this to automatically
/// create routes in Axum's Router without having to use wildcard matching or fallbacks. Take in your root app Element
/// as an argument, so it can walk your app tree. This version is tailored to generate Axum compatible paths. Adding excluded_routes
/// to this function will stop `.leptos_routes()` from generating a route for it, allowing a custom handler. These need to be in Axum path format
/// Additional context will be provided to the app Element.
#[tracing::instrument(level = "trace", fields(error), skip_all)]
pub fn generate_route_list_with_exclusions_and_ssg_and_context(
    app_fn: RouteAppFunction,
    excluded_routes: Option<Vec<String>>,
    additional_context: impl Fn() + 'static + Clone,
) -> (Vec<PavexRouteListing>, StaticDataMap) {
    init_executor();

    let owner = Owner::new_root(None);
    let routes = owner
        .with(|| {
            // stub out a path for now
            provide_context(RequestUrl::new(""));
            let mock_parts = RequestParts::new();

            provide_post_contexts("", &Default::default(), mock_parts, Default::default());
            additional_context();
            RouteList::generate(app_fn)
        })
        .unwrap_or_default();

    // Axum's Router defines Root routes as "/" not ""
    let mut routes = routes
        .into_inner()
        .into_iter()
        .map(PavexRouteListing::from)
        .collect::<Vec<_>>();

    (
        if routes.is_empty() {
            vec![PavexRouteListing::new(
                "/".to_string(),
                Default::default(),
                [leptos_router::Method::Get],
                None,
            )]
        } else {
            // Routes to exclude from auto generation
            if let Some(excluded_routes) = excluded_routes {
                routes.retain(|p| !excluded_routes.iter().any(|e| e == p.path()))
            }
            routes
        },
        StaticDataMap::new(), // TODO
                              //static_data_map,
    )
}

// An Enum to specify
pub enum RouteType {
    ServerFn,
    Component,
}
/// A function to pass Leptos additional context from Pavex
/// The owner needs to exist for calls to provide_context() to be successful. Pavex needs this to run
/// before the constructor for additional context
pub fn pass_leptos_context(
    route_type: &RouteType,
    req_head: &RequestHead,
    additional_context: impl Fn() + 'static + Clone,
) -> Owner {
    let owner = match route_type {
        RouteType::ServerFn => Owner::new(),
        RouteType::Component => Owner::new_root(Some(Arc::new(SsrSharedContext::new()))),
    };
    let req_parts = RequestParts::new_from_req(&req_head);
    // Set the created Owner as the current one, by setting the thread local. Pavex pins each request to their own
    // thread, so this should be fineTM
    owner.with(|| {
        provide_initial_contexts(req_head, req_parts);
        additional_context();
    });
    owner
}
