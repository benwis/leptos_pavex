use crate::pages::{home::__Home, __About};
use leptos::prelude::*;
use leptos::{component, view, IntoView};
use leptos_meta::*;
use leptos_router::components::*;
use leptos_router::*;
/*
    components::{Route, Router, Routes},
    StaticSegment,
};*/
#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Meta charset="utf-8" />
        <Meta name="viewport" content="width=device-width,inistial-scale=1.0" />
        <Link rel="preconnect" href="https://fonts.googleapis.com" />
        <Link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="anonymous" />
        <Link href="https://fonts.googleapis.com/css2?family=Open+Sans:wght@400;600;700;800&display=swap" rel="stylesheet" />
        // <Stylesheet id="leptos" href="/pkg/start-pavex.css"/>
        <link rel="stylesheet" href="/pkg/start_pavex.css"/>


        // sets the document title
        <Title text="Leptos Pavex Starter"/>

        // content for this welcome page
        <Router>
            <main>

                <Routes fallback=||{view!{<p>"Not found"</p>}}>
                <Route path=StaticSegment("/") view=__Home/>
                <Route path=StaticSegment("/about") view=__About/>

            </Routes>
            </main>
        </Router>
    }
}
