pub mod greet;
pub mod ping;

use leptos_pavex::leptos_routes::generate_leptos_routes;
use pavex::blueprint::{router::GET, Blueprint};
use pavex::f;

use crate::leptos::generate_route_app;

pub fn register(bp: &mut Blueprint) {
    // Generate routes for routes defined in Leptos for components and server fns
    let routes = leptos_pavex::generate_route_list(generate_route_app());
    generate_leptos_routes(&routes, bp);
    bp.route(GET, "/api/ping", f!(self::ping::get));
    bp.route(GET, "/api/greet/:name", f!(self::greet::get));
    //bp.route(GET, "/", f!(leptos_pavex::render_route_with_context));
    bp.route(GET, "/*path", f!(leptos_pavex::file_helpers::serve_files));
}
