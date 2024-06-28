use leptos::prelude::ElementChild;
use leptos::{component, view, IntoView};

/// Renders the home page of your application.
#[component]
fn About() -> impl IntoView {
    view! {
        <main>
            <h1>"About Page"</h1>
        </main>
    }
}