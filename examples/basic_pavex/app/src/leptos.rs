use leptos::prelude::View;
use leptos_pavex::pavex_helpers::{AdditionalContextComponent, AdditionalContextServerFn};
use leptos_pavex::{pass_leptos_context, RouteType};
use pavex::request::RequestHead;
//A type to hold the App Function Closure so Pavex is happy
pub struct AppFunction(View);

pub fn additional_context_components(req_head: &RequestHead) -> AdditionalContextComponent{
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