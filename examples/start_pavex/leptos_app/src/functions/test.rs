use leptos::prelude::*;

#[server(endpoint="greet")]
pub async fn greetings(name: String) -> Result<String, ServerFnError>{
    Ok(format!("Salutations {name}"))
}