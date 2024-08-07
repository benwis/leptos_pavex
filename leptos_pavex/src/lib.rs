#[allow(dead_code)]
pub mod extend_response;
#[cfg(feature = "ssr")]
pub mod file_helpers;

pub mod leptos_routes;
pub mod pavex_helpers;
pub mod request;
pub mod request_parts;
pub mod response;
pub mod response_options;
pub mod server_fn;
pub mod stream;

use bytes::Bytes;
use extend_response::ExtendResponse;
use futures::stream::once;
use futures::{Stream, StreamExt};
use hydration_context::SsrSharedContext;
use leptos::server_fn::redirect::REDIRECT_HEADER;
use reactive_graph::owner::expect_context;
use std::io;
use std::pin::Pin;
use std::sync::Arc;

use crate::request_parts::RequestParts;
use crate::response_options::ResponseOptions;
use leptos::prelude::{provide_context, use_context, Owner};
use leptos::tachys::view::RenderHtml;
use leptos_integration_utils::{BoxedFnOnce, PinnedFuture, PinnedStream};
use leptos_meta::{ServerMetaContext, ServerMetaContextOutput};
use leptos_router::components::provide_server_redirect;
use leptos_router::location::RequestUrl;
use leptos_router::{PathSegment, RouteList, RouteListing, SsrMode, StaticDataMap, StaticMode};
use pavex::http::header::{ACCEPT, LOCATION};
use pavex::http::uri::PathAndQuery;
use pavex::http::StatusCode;
use pavex::http::{HeaderName, HeaderValue};
use pavex::request::body::RawIncomingBody;
use pavex::request::path::MatchedPathPattern;
use pavex::request::RequestHead;
use pavex::response::Response;
use pavex_helpers::{AppFunction, ComponentOwner};
use reactive_graph::computed::ScopedFuture;
use response::PavexResponse;

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

/// Spawn an async executor dependent on the environment in which we're running. Tokio for server
/// environments and wasm-bindgen-futures for wasm ones.
fn init_executor() {
    #[cfg(feature = "wasm")]
    let _ = any_spawner::Executor::init_wasm_bindgen();
    #[cfg(all(not(feature = "wasm"), feature = "ssr"))]
    let _ = any_spawner::Executor::init_tokio();
    #[cfg(all(not(feature = "wasm"), not(feature = "ssr")))]
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

/// Returns a Pavex Response containing an HTML stream of your application.
///
/// It provides a MetaContext and a RouterIntegrationContext to the app's context
/// before rendering it, and includes any meta tags injected using leptos_meta.
///
/// The HTML stream is rendered using leptos's render_to_stream and includes everything
/// defined in the documentation for that function.
#[tracing::instrument(level = "trace", fields(error), skip_all)]
pub async fn render_app_to_stream(
    req_head: RequestHead,
    req_body: RawIncomingBody,
    app_fn: AppFunction,
    meta_context_output: ServerMetaContextOutput,
) -> Response {
    render_app_to_stream_and_replace_blocks(req_head, req_body, app_fn, meta_context_output, false)
        .await
}

/// Returns a Pavex Response containing an HTML stream of your application.
///
/// It provides a MetaContext and a RouterIntegrationContext to the app's context
/// before rendering it, and includes any meta tags injected using leptos_meta
///
/// This is a handy entrypoint for a Pavex handler, taking in both additional context
/// and the rendering mode of each route in your Leptos app.
#[tracing::instrument(level = "trace", fields(error), skip_all)]
pub async fn render_route(
    paths: PavexRouteList,
    req_head: RequestHead,
    req_body: RawIncomingBody,
    matched_path: &MatchedPathPattern,
    mut context: ComponentOwner,
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
    let meta_output = context.take_meta_context_output();
    let owner = context.owner();

    match listing.mode() {
        SsrMode::OutOfOrder => {
            owner
                .with(|| {
                    ScopedFuture::new(render_app_to_stream(
                        req_head,
                        req_body,
                        app_fn,
                        meta_output,
                    ))
                })
                .await
        }
        SsrMode::PartiallyBlocked => {
            owner
                .with(|| {
                    ScopedFuture::new(render_app_to_stream_and_replace_blocks(
                        req_head,
                        req_body,
                        app_fn,
                        meta_output,
                        true,
                    ))
                })
                .await
        }
        SsrMode::InOrder => {
            owner
                .with(|| {
                    ScopedFuture::new(render_app_to_stream_in_order(
                        req_head,
                        req_body,
                        app_fn,
                        meta_output,
                    ))
                })
                .await
        }
        SsrMode::Async => {
            owner
                .with(|| {
                    ScopedFuture::new(render_app_async(req_head, req_body, app_fn, meta_output))
                })
                .await
        }
    }
}

/// Returns a Pavex Response containing an HTML stream of your application.
///
/// It provides a MetaContext and a RouterIntegrationContext to the app's context
/// before rendering it, and includes any meta tags injected using leptos_meta.
///
/// The HTML stream is rendered using leptos's render_to_stream and includes everything
/// defined in the documentation for that function.
///
/// For now, this is identical to render_app_to_stream()
#[tracing::instrument(level = "trace", fields(error), skip_all)]
pub async fn render_app_to_stream_and_replace_blocks(
    req_head: RequestHead,
    req_body: RawIncomingBody,
    app_fn: AppFunction,
    meta_context_output: ServerMetaContextOutput,
    replace_blocks: bool,
) -> Response {
    _ = replace_blocks; // TODO
    handle_response(
        req_head,
        req_body,
        app_fn,
        meta_context_output,
        |app, chunks| {
            Box::pin(async move {
                Box::pin(app.inner().to_html_stream_out_of_order().chain(chunks()))
                    as PinnedStream<String>
            })
        },
    )
    .await
}

/// Returns a Pavex Response containing an HTML stream of your application.
///
/// It provides a MetaContext and a RouterIntegrationContext to the app's context
/// before rendering it, and includes any meta tags injected using leptos_meta.
///
/// This will handle Resources in source order, as compared to the default Out of Order streaming.
/// For more info on the different modes, check out the docs on SsrMode.
#[tracing::instrument(level = "trace", fields(error), skip_all)]
pub async fn render_app_to_stream_in_order(
    req_head: RequestHead,
    req_body: RawIncomingBody,
    app_fn: AppFunction,
    meta_context_output: ServerMetaContextOutput,
) -> Response {
    handle_response(
        req_head,
        req_body,
        app_fn,
        meta_context_output,
        |app, chunks| {
            Box::pin(async move {
                Box::pin(app.inner().to_html_stream_in_order().chain(chunks()))
                    as PinnedStream<String>
            })
        },
    )
    .await
}

/// Returns a Pavex Response containing an HTML stream of your application.
///
/// It provides a MetaContext and a RouterIntegrationContext to the app's context
/// before rendering it, and includes any meta tags injected using leptos_meta.
///
/// This will handle all Resources on the server, waiting for them to complete before returning HTML.
/// For more info on the different modes, check out the docs on SsrMode.
#[tracing::instrument(level = "trace", fields(error), skip_all)]
pub async fn render_app_async(
    req_head: RequestHead,
    req_body: RawIncomingBody,
    app_fn: AppFunction,
    meta_context_output: ServerMetaContextOutput,
) -> Response {
    handle_response(
        req_head,
        req_body,
        app_fn,
        meta_context_output,
        |app, chunks| {
            Box::pin(async move {
                let app = app
                    .inner()
                    .to_html_stream_in_order()
                    .collect::<String>()
                    .await;
                let chunks = chunks();
                Box::pin(once(async move { app }).chain(chunks)) as PinnedStream<String>
            })
        },
    )
    .await
}
/// A convenience function leptos_pavex uses to build the Pavex Response in a variety of ways
async fn handle_response(
    req_head: RequestHead,
    _req_body: RawIncomingBody,
    app_fn: AppFunction,
    meta_context_output: ServerMetaContextOutput,
    stream_builder: fn(
        AppFunction,
        BoxedFnOnce<PinnedStream<String>>,
    ) -> PinnedFuture<PinnedStream<String>>,
) -> Response {
    let res_options: ResponseOptions = ResponseOptions::default();
    let meta_context = expect_context::<ServerMetaContext>();

    let additional_context = {
        let meta_context = meta_context.clone();
        let res_options = res_options.clone();
        move || {
            // Need to get the path and query string of the Request
            // For reasons that escape me, if the incoming URI protocol is https, it provides the absolute URI
            let path = req_head.target.path_and_query().unwrap().as_str();

            let full_path = format!("http://leptos.dev{path}");
            let req_parts = RequestParts::new_from_req(&req_head);
            provide_post_contexts(&full_path, meta_context, req_parts, res_options.clone());
        }
    };

    let res = PavexResponse::from_app(
        app_fn,
        meta_context_output,
        additional_context,
        res_options,
        stream_builder,
    )
    .await;

    res.0
}

/// Provide additional information to Leptos from an outside environment. This could be global
/// state, a db pool, or data from Pavex extractors or middleware.
#[tracing::instrument(level = "trace", fields(error), skip_all)]
pub fn provide_initial_contexts(
    req_head: &RequestHead,
    parts: RequestParts,
    meta_context: ServerMetaContext,
) {
    let path = req_head
        .target
        .path_and_query()
        .cloned()
        .unwrap_or(PathAndQuery::from_static("/"));
    provide_context(RequestUrl::new(&path.to_string()));
    provide_context(parts);
    provide_context(meta_context);

    provide_server_redirect(redirect);
    #[cfg(feature = "nonce")]
    leptos::nonce::provide_nonce();
}
// Makes sure the required items are provided to the context, depending on what could be set by the
// user in their own context handler.
#[tracing::instrument(level = "trace", fields(error), skip_all)]
fn provide_post_contexts(
    path: &str,
    meta_context: ServerMetaContext,
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
    if use_context::<ServerMetaContext>().is_none() {
        provide_context(meta_context);
    }
    provide_context(default_res_options);
    provide_server_redirect(redirect);
    #[cfg(feature = "nonce")]
    leptos::nonce::provide_nonce();
}

/// Generates a list of all routes defined in Leptos's Router in your app. We can then use this to automatically
/// create routes in Axum's Router without having to use wildcard matching or fallbacks. Takes in your root app Element
/// as an argument, so it can walk your app tree. This version is tailored to generate Axum compatible paths.
#[tracing::instrument(level = "trace", fields(error), skip_all)]
pub fn generate_route_list(app_fn: AppFunction) -> PavexRouteList {
    generate_route_list_with_exclusions_and_ssg(app_fn, None).0
}

/// Generates a list of all routes defined in Leptos's Router in your app. We can then use this to automatically
/// create routes in Axum's Router without having to use wildcard matching or fallbacks. Take in your root app Element
/// as an argument, so it can walk your app tree. This version is tailored to generate Axum compatible paths.
#[tracing::instrument(level = "trace", fields(error), skip_all)]
pub fn generate_route_list_with_ssg(app_fn: AppFunction) -> (PavexRouteList, StaticDataMap) {
    generate_route_list_with_exclusions_and_ssg(app_fn, None)
}

/// Generates a list of all routes defined in Leptos's Router in your app. We can then use this to automatically
/// create routes in Axum's Router without having to use wildcard matching or fallbacks. Takes in your root app Element
/// as an argument, so it can walk you app tree. This version is tailored to generate Axum compatible paths. Adding excluded_routes
/// to this function will stop `.leptos_routes()` from generating a route for it, allowing a custom handler. These need to be in Axum path format
#[tracing::instrument(level = "trace", fields(error), skip_all)]
pub fn generate_route_list_with_exclusions(
    app_fn: AppFunction,
    excluded_routes: Option<Vec<String>>,
) -> PavexRouteList {
    generate_route_list_with_exclusions_and_ssg(app_fn, excluded_routes).0
}

/// Generates a list of all routes defined in Leptos's Router in your app. We can then use this to automatically
/// create routes in Axum's Router without having to use wildcard matching or fallbacks. Takes in your root app Element
/// as an argument, so it can walk you app tree. This version is tailored to generate Axum compatible paths. Adding excluded_routes
/// to this function will stop `.leptos_routes()` from generating a route for it, allowing a custom handler. These need to be in Axum path format
#[tracing::instrument(level = "trace", fields(error), skip_all)]
pub fn generate_route_list_with_exclusions_and_ssg(
    app_fn: AppFunction,
    excluded_routes: Option<Vec<String>>,
) -> (PavexRouteList, StaticDataMap) {
    generate_route_list_with_exclusions_and_ssg_and_context(app_fn, excluded_routes, || {})
}

/// A convenience type for a collection of Pavex routes
pub type PavexRouteList = Vec<PavexRouteListing>;

/// A route that this application can serve.
#[derive(Clone, Debug, Default)]
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
    app_fn: AppFunction,
    excluded_routes: Option<Vec<String>>,
    additional_context: impl Fn() + 'static + Clone,
) -> (PavexRouteList, StaticDataMap) {
    init_executor();

    let owner = Owner::new_root(None);
    let routes = owner
        .with(|| {
            // stub out a path for now
            provide_context(RequestUrl::new(""));
            let mock_parts = RequestParts::new();
            let (meta_context, _) = ServerMetaContext::new();
            provide_post_contexts("", meta_context, mock_parts, Default::default());
            additional_context();
            let foo = RouteList::generate(move || app_fn.inner());
            foo
        })
        .unwrap();

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

// An enum holding the different types of routes requiring different Owner setups
pub enum RouteType {
    ServerFn,
    Component,
}

/// Provide additional information to Leptos' context from an outside environment. This could be global
/// state, a db pool, or data from Pavex extractors or middleware. This will need to run before
/// Pavex generates the AppFunction
pub fn pass_leptos_context(
    route_type: &RouteType,
    req_head: &RequestHead,
    additional_context: impl Fn() + 'static + Clone,
) -> (Owner, ServerMetaContextOutput) {
    let owner = match route_type {
        RouteType::ServerFn => Owner::new(),
        RouteType::Component => Owner::new_root(Some(Arc::new(SsrSharedContext::new()))),
    };
    let req_parts = RequestParts::new_from_req(&req_head);

    let (meta_context, meta_context_output) = ServerMetaContext::new();
    // Set the created Owner as the current one, by setting the thread local. Pavex pins each request to their own
    // thread, so this should be fineTM
    owner.with(|| {
        provide_initial_contexts(req_head, req_parts, meta_context);
        additional_context();
    });
    (owner, meta_context_output)
}
