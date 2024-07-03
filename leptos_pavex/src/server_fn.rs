use crate::pavex_helpers::AdditionalContextServerFn;
use crate::request_parts::RequestParts;
use crate::response_options::ResponseOptions;
use crate::{request::PavexRequest, response::PavexResponse};
use dashmap::DashMap;
use leptos::prelude::{provide_context, ScopedFuture};
use leptos::server_fn::middleware::Service;
use leptos::server_fn::{codec::Encoding, initialize_server_fn_map, ServerFn, ServerFnTraitObj};
use once_cell::sync::Lazy;
use pavex::http::{HeaderName, Method as HttpMethod, StatusCode};
use pavex::request::body::RawIncomingBody;
use pavex::request::RequestHead;
use pavex::response::Response;
use url::Url;

#[allow(unused)] // used by server integrations
type LazyServerFnMap<Req, Res> = Lazy<DashMap<&'static str, ServerFnTraitObj<Req, Res>>>;

static REGISTERED_SERVER_FUNCTIONS: LazyServerFnMap<PavexRequest, PavexResponse> =
    initialize_server_fn_map!(PavexRequest, PavexResponse);

/// Explicitly register a server function. This is only necessary if you are
/// running the server in a WASM environment (or a rare environment that the
/// `inventory`) crate doesn't support. Spin is one of those environments
pub fn register_explicit<T>()
where
    T: ServerFn<ServerRequest = PavexRequest, ServerResponse = PavexResponse> + 'static,
{
    REGISTERED_SERVER_FUNCTIONS.insert(
        T::PATH,
        ServerFnTraitObj::new(
            T::PATH,
            T::InputEncoding::METHOD,
            |req| Box::pin(T::run_on_server(req)),
            T::middlewares,
        ),
    );
}

/// The set of all registered server function paths.
pub fn server_fn_paths() -> impl Iterator<Item = (&'static str, HttpMethod)> {
    REGISTERED_SERVER_FUNCTIONS
        .iter()
        .map(|item| (item.path(), item.method()))
}
pub async fn handle_server_fns(
    req_head: RequestHead,
    req_body: RawIncomingBody,
    context: AdditionalContextServerFn,
) -> Response {
    handle_server_fns_with_context(req_head, req_body, context).await
}
pub async fn handle_server_fns_with_context(
    req_head: RequestHead,
    req_body: RawIncomingBody,
    context: AdditionalContextServerFn,
) -> Response {
    let pq = req_head.target.path_and_query().unwrap();
    match crate::server_fn::get_server_fn_by_path(pq.as_str()) {
        Some(lepfn) => {
            let owner = context.owner();
            let blah = owner.with(|| {
                ScopedFuture::new(async move {
                    let req_parts = RequestParts::new_from_req(&req_head);
                    provide_context(req_parts.clone());
                    let res_options = ResponseOptions::default();
                    provide_context(res_options.clone());
                    let pavex_req = PavexRequest::new_from_req(req_head, req_body);
                    let (mut pavex_res, req_parts, res_options) =
                        (lepfn.clone().run(pavex_req).await, req_parts, res_options);
                    // If the Accept header contains text/html, then this is a request from
                    // a regular html form, so we should set up a redirect to either the referrer
                    // or the user specified location

                    let req_headers = req_parts.headers();
                    let accepts = req_headers.get("Accept");
                    let accepts_html_bool = match accepts {
                        Some(h) => match h.to_str().unwrap().contains("text/html") {
                            true => true,
                            false => false,
                        },
                        None => false,
                    };

                    if accepts_html_bool {
                        let referrer = &req_headers.get("Referer");
                        let location = &req_headers.get("Location");
                        if location.is_none() {
                            if let Some(referrer) = *referrer {
                                res_options.insert_header(
                                    HeaderName::from_static("location"),
                                    referrer.to_owned(),
                                );
                            }
                        }
                        // Set status
                        if res_options.status().is_none() {
                            res_options.set_status(StatusCode::FOUND);
                        }
                    }

                    pavex_res
                        .0
                        .headers_mut()
                        .extend(std::mem::take(&mut res_options.headers()));

                    if let Some(status) = res_options.status() {
                        pavex_res.0 = pavex_res.0.set_status(status);
                    }
                    pavex_res.0
                })
            });
            blah.await
        }
        //None => panic!("Server FN path {} not found", &pq)
        None => Response::new(StatusCode::BAD_REQUEST).set_typed_body(format!(
            "Could not find a server function at the route {pq}. \
                 \n\nIt's likely that either
                         1. The API prefix you specify in the `#[server]` \
                 macro doesn't match the prefix at which your server function \
                 handler is mounted, or \n2. You are on a platform that \
                 doesn't support automatic server function registration and \
                 you need to call ServerFn::register_explicit() on the server \
                 function type, somewhere in your `main` function.",
        )),
    }
}

/// Returns the server function at the given path
pub fn get_server_fn_by_path(path: &str) -> Option<ServerFnTraitObj<PavexRequest, PavexResponse>> {
    // Sanitize Url to prevent query string or ids causing issues. To do that Url wants a full url,
    // so we give it a fake one. We're only using the path anyway!
    let full_url = format!("http://leptos.dev{}", path);
    let Ok(url) = Url::parse(&full_url) else {
        println!("Failed to parse: {full_url:?}");
        return None;
    };
    REGISTERED_SERVER_FUNCTIONS
        .get_mut(url.path())
        .map(|f| f.clone())
}
