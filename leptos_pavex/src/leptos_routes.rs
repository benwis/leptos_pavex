use http::Method;
use leptos_router::Method as LeptosMethod;
use pavex::{
    blueprint::{
        router::{DELETE, GET, PATCH, POST, PUT},
        Blueprint,
    },
    f,
};

use crate::{init_executor, PavexRouteList};

pub fn generate_leptos_routes(paths: &PavexRouteList, bp: &mut Blueprint) {
    init_executor();
    println!("GENERATE LEPTOS ROUTES");

    // register server functions
    for (path, method) in crate::server_fn::server_fn_paths() {
        println!("REGISTERING SERVER FN ROUTE: {path}");
        let method = match method {
            Method::GET => GET,
            Method::POST => POST,
            Method::PUT => PUT,
            Method::DELETE => DELETE,
            Method::PATCH => PATCH,
            _ => {
                panic!(
                    "Unsupported server function HTTP method: \
                 {method:?}"
                );
            }
        };
        println!("METHOD: {method:?}");
        println!("PATH: {path}");
        bp.route(
            
            method,
            path,
            f!(crate::server_fn::handle_server_fns),
        );
    }

    // register router paths
    for listing in paths.iter() {
        let path = listing.path();
        println!("REGISTERING REGULAR PATH: {path}");
        for method in listing.methods() {
            bp.route(
                match method {
                    LeptosMethod::Get => GET,
                    LeptosMethod::Post => POST,
                    LeptosMethod::Put => PUT,
                    LeptosMethod::Delete => DELETE,
                    LeptosMethod::Patch => PATCH,
                },
                path,
                f!(crate::render_route_with_context), 
            );
        }
    }
}
