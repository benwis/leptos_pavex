use leptos::context::provide_context;
use leptos::view;
use leptos_app::pages::App;
use leptos_pavex::pavex_helpers::{
    generate_app_function, generate_route_app_function, AdditionalContextComponent,
    AdditionalContextServerFn, AppFunction, RouteAppFunction,
};
use leptos_pavex::{pass_leptos_context, RouteType};
use pavex::request::RequestHead;
use leptos_meta::MetaTags;
use leptos::prelude::*;

pub fn generate_app(options: LeptosOptions) -> AppFunction {
  
    generate_app_function( move || {
        let options2: LeptosOptions = options.clone();
        view! {
            <!DOCTYPE html>
            <html lang="en">
                <head>
                    <meta charset="utf-8"/>
                    <meta name="viewport" content="width=device-width, initial-scale=1"/>
                    // <AutoReload options=app_state.leptos_options.clone() />
                    <HydrationScripts options=options2.clone()/>
                    <MetaTags/>
                </head>
                <body>
                    <App/>
                </body>
            </html>
        }
    })
}
pub fn generate_route_app(options: LeptosOptions) -> RouteAppFunction {
   let blah = move || {
        view! {
            <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
        }
    };
    RouteAppFunction::new(blah().into_any())

}

pub fn additional_context_components(req_head: &RequestHead) -> AdditionalContextComponent {
    let owner = pass_leptos_context(&RouteType::Component, req_head, || {
        // Pass additional context items here
        provide_context("Test".to_string());
    });
    AdditionalContextComponent::new(owner)
}

pub fn additional_context_serverfn(req_head: &RequestHead) -> AdditionalContextServerFn {
    let owner = pass_leptos_context(&RouteType::ServerFn, req_head, || {
        // Pass additional context items here
        provide_context("Test".to_string());
    });
    AdditionalContextServerFn::new(owner)
}