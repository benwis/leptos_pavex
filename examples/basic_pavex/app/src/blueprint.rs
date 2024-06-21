use crate::{configuration, routes, telemetry};
use pavex::blueprint::constructor::Lifecycle;
use pavex::blueprint::Blueprint;
use pavex::kit::ApiKit;
use pavex::{f, t};

/// The main blueprint, containing all the routes, middlewares, constructors and error handlers
/// required by our API.
pub fn blueprint() -> Blueprint {
    let mut bp = Blueprint::new();
    ApiKit::new().register(&mut bp);
    telemetry::register(&mut bp);
    configuration::register(&mut bp);

    // Register the Leptos types we need to pass to render_routes_with_context
    bp.constructor(
        f!(leptos_pavex::generate_route_list),
        Lifecycle::RequestScoped,
    )
    .clone_if_necessary();
    bp.constructor(
        f!(super::leptos::additional_context_components),
        Lifecycle::RequestScoped,
    );
    bp.constructor(
        f!(super::leptos::additional_context_serverfn),
        Lifecycle::RequestScoped,
    );
    bp.constructor(f!(super::leptos::generate_app), Lifecycle::RequestScoped);
    bp.constructor(
        f!(super::leptos::generate_route_app),
        Lifecycle::RequestScoped,
    );

    routes::register(&mut bp);
    bp
}
