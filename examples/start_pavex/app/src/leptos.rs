use leptos::prelude::{
    provide_context, view, ElementChild, GlobalAttributes, HydrationScripts, IntoAny, LeptosOptions,
};
use leptos_app::pages::App;
use leptos_meta::MetaTags;
use leptos_pavex::pavex_helpers::{AppFunction, ComponentOwner, ServerFnOwner};
use leptos_pavex::{pass_leptos_context, RouteType};
use pavex::request::RequestHead;

pub fn generate_app(context: &ComponentOwner, options: LeptosOptions) -> AppFunction {
    let owner = context.owner();
    let fun = move || {
        view! {
            <!DOCTYPE html>
            <html lang="en">
                <head>
                    <meta charset="utf-8"/>
                    <meta name="viewport" content="width=device-width, initial-scale=1"/>
                    // <AutoReload options=options.clone() />
                    <HydrationScripts options/>
                    <MetaTags/>
                </head>
                <body>
                    <App/>
                </body>
            </html>
        }
    };
    AppFunction::new(owner.with(fun).into_any())
}

pub fn additional_context_components(req_head: &RequestHead) -> ComponentOwner {
    let (owner, meta_context_output) = pass_leptos_context(&RouteType::Component, req_head, || {
        // Pass additional context items here
        provide_context("Test".to_string());
    });
    ComponentOwner::new(owner, meta_context_output)
}

pub fn additional_context_serverfn(req_head: &RequestHead) -> ServerFnOwner {
    let (owner, _) = pass_leptos_context(&RouteType::ServerFn, req_head, || {
        // Pass additional context items here
        provide_context("Test".to_string());
    });
    ServerFnOwner::new(owner)
}
