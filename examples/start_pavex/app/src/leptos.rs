use http::Request;
use leptos::context::provide_context;
use leptos::view;
use leptos_app::pages::App;
use leptos_pavex::pavex_helpers::{
    generate_app_function, AdditionalContextComponent,
    AdditionalContextServerFn, AppFunction, RouteAppFunction,
};
use leptos_pavex::{pass_leptos_context, RouteType};
use pavex::request::RequestHead;
use leptos_meta::MetaTags;
use leptos::prelude::*;

pub fn generate_app(options: LeptosOptions, req_head: &RequestHead) -> AppFunction {
  
    let context = additional_context_components(req_head);
    let owner = context.owner();
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
    AppFunction::new(owner.with(blah).into_any())
}
pub fn generate_route_app(options: LeptosOptions, req_head: &RequestHead ) -> RouteAppFunction {
    
    // let mock_request = Request::builder()
    // .uri("https://www.leptos.dev/").body(()).unwrap();
    // let mock_req_head: RequestHead = mock_request.into_parts().0.into();

   let context = additional_context_components(req_head);
   let owner = context.owner();
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
    RouteAppFunction::new(owner.with(blah).into_any())

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

pub fn handle_leptos_options() -> LeptosOptions{
   get_configuration(None).unwrap().leptos_options
}