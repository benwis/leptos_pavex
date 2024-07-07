use leptos::prelude::IntoAny;
use leptos::{
    tachys::{renderer::dom::Dom, view::any_view::AnyView},
    IntoView,
};
use leptos_meta::ServerMetaContextOutput;
use reactive_graph::owner::Owner;

//A struct to hold the output of the app function closure so Pavex is happy
pub struct AppFunction(AnyView<Dom>);
impl AppFunction {
    pub fn new(any_view: AnyView<Dom>) -> Self {
        Self(any_view)
    }
    pub fn inner(self) -> AnyView<Dom> {
        self.0
    }
    pub fn inner_ref(&self) -> &AnyView<Dom> {
        &self.0
    }
}
/// Provide a helper function for users to put in a Pavex constructor to handle the
/// Leptos application closure.
pub fn generate_app_function<IV>(app_fn: impl Fn() -> IV + Clone + Send + 'static) -> AppFunction
where
    IV: IntoView + 'static,
{
    let any_view = app_fn().into_any();
    AppFunction::new(any_view)
}

/// This type holds the app's root root reactive Owner, which will be generated for each request,
/// and differs between server functions and regular Leptos routes
#[derive(Debug)]
pub struct ComponentOwner {
    owner: Owner,
    meta_output: Option<ServerMetaContextOutput>,
}

impl ComponentOwner {
    pub fn new(owner: Owner, meta_output: ServerMetaContextOutput) -> Self {
        Self { owner, meta_output: Some(meta_output) }
    }

    pub fn owner(&self) -> &Owner {
        &self.owner
    }

    pub fn take_meta_context_output(&mut self) -> ServerMetaContextOutput {
        self.meta_output.take().expect("Trying to access a ServerMetaContextOutput that has already been taken!")
    }
}
/// This type holds the app's root root reactive Owner, which will be generated for each request,
/// and differs between server functions and regular Leptos routes
#[derive(Debug, Default)]
pub struct ServerFnOwner(Owner);
impl ServerFnOwner {
    /// Give this type your additional context in a closure
    pub fn new(owner: Owner) -> Self {
        Self(owner)
    }

    pub fn owner(&self) -> &Owner {
        &self.0
    }
}
