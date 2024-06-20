use leptos::context::provide_context;
use leptos::view;
use leptos_pavex::pavex_helpers::{generate_app_function, generate_route_app_function, AdditionalContextComponent, AdditionalContextServerFn, AppFunction, RouteAppFunction};
use leptos_pavex::{pass_leptos_context, RouteType};
use pavex::request::RequestHead;
use leptos::prelude::ElementChild;

pub fn generate_app() -> AppFunction{
    generate_app_function(||{view! {<p>"Hello from Pavex!"</p>}})
}
pub fn generate_route_app() -> RouteAppFunction{
    generate_route_app_function(||{view! {<p>"Hello from Pavex!"</p>}})
}

pub fn additional_context_components(req_head: &RequestHead) -> AdditionalContextComponent{
    println!("ADDITIONAL CONTEXT ADDED");
    pass_leptos_context(&RouteType::Component, req_head, ||{
    // Pass additional context items here
    provide_context("Test".to_string());
    });
    AdditionalContextComponent
}

pub fn additional_context_serverfn(req_head: &RequestHead) -> AdditionalContextServerFn{
    pass_leptos_context(&RouteType::ServerFn, req_head, ||{
        // Pass additional context items here
        provide_context("Test".to_string());
    });
    AdditionalContextServerFn
}