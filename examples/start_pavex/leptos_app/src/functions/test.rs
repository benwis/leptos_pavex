use leptos_pavex_macro::server;
//use leptos::prelude::server;
use leptos::prelude::ServerFnError;
#[server(endpoint="greet")]
pub async fn greetings(name: String) -> Result<String, ServerFnError>{
    Ok(format!("Salutations {name}"))
}