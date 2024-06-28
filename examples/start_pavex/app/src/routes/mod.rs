pub mod greet;
pub mod ping;

use http::Request;
use leptos::config::get_configuration;
use leptos_pavex::leptos_routes::generate_leptos_routes;
use pavex::blueprint::{router::GET, Blueprint};
use pavex::f;
use pavex::request::RequestHead;

use crate::leptos::generate_route_app;

pub fn register(bp: &mut Blueprint) {

    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;

    // Generate routes for routes defined in Leptos for components and server fns
    let mock_request = Request::builder().uri("https://www.leptos.dev/").body(()).unwrap();
    let mock_req_head: RequestHead = mock_request.into_parts().0.into();
    let routes = leptos_pavex::generate_route_list(generate_route_app(leptos_options, &mock_req_head));
    generate_leptos_routes(&routes, bp);
    bp.route(GET, "/api/ping", f!(self::ping::get));
    bp.route(GET, "/api/greet/:name", f!(self::greet::get));
    //bp.route(GET, "/", f!(leptos_pavex::render_route_with_context));
    bp.route(GET, "/*path", f!(leptos_pavex::file_helpers::serve_files));
}
