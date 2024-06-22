use http::Method;
use leptos_router::Method as LeptosMethod;
use pavex::{
    blueprint::{
        router::{DELETE, GET, PATCH, POST, PUT},
        Blueprint,
    },
    f,
};

use crate::{init_executor, PavexRouteListing};

pub fn generate_leptos_routes(paths: &Vec<PavexRouteListing>, bp: &mut Blueprint) {
    init_executor();

    // register server functions
    for (path, method) in server_fn::axum::server_fn_paths() {
        bp.route(
            match method {
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
            },
            path,
            f!(crate::server_fn::handle_server_fns),
        );
    }

    // register router paths
    for listing in paths.iter() {
        let path = listing.path();

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
                f!(crate::render_route_with_context), // match listing.mode() {
                                                      //     SsrMode::OutOfOrder => f!(crate::render_app_to_stream),
                                                      //     SsrMode::PartiallyBlocked => {
                                                      //         f!(crate::render_app_to_stream_with_context_and_replace_blocks)
                                                      //     }
                                                      //     SsrMode::InOrder => f!(crate::render_app_to_stream_in_order_with_context),
                                                      //     SsrMode::Async => f!(crate::render_app_async_with_context),
                                                      // },
            );
        }
    }
}
