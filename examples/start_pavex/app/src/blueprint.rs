use crate::{configuration, routes, telemetry};
use pavex::blueprint::constructor::Lifecycle;
use pavex::blueprint::linter::Lint;
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

    bp.constructor(
        f!(super::leptos::additional_context_components),
        Lifecycle::RequestScoped,
    );
    bp.constructor(
        f!(super::leptos::additional_context_serverfn),
        Lifecycle::RequestScoped,
    )
    .ignore(Lint::Unused);
    bp.constructor(f!(super::leptos::generate_app), Lifecycle::RequestScoped);

    bp.prebuilt(t!(leptos_config::LeptosOptions))
        .clone_if_necessary();
    bp.prebuilt(t!(std::vec::Vec<leptos_pavex::PavexRouteListing>))
        .clone_if_necessary();

    routes::register(&mut bp);
    bp
}
