use leptos::prelude::ServerFnError;
//use leptos::prelude::*;
use leptos_pavex_macro::server;

#[server(endpoint="greet")]
pub async fn greetings(name: String) -> Result<String, ServerFnError>{
    Ok(format!("Salutations {name}"))
}