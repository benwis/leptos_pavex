use leptos::prelude::IntoAny;
use leptos::{
    tachys::{renderer::dom::Dom, view::any_view::AnyView},
    IntoView,
};
use reactive_graph::owner::Owner;

//A type to hold the result of the App Function Closure so Pavex is happy
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
// Provide a constructor for AppFunction so Pavex can generate it at will
// You'll need to put this inside another function so you can specify the function and not Pavex
pub fn generate_app_function<IV>(app_fn: impl Fn() -> IV + Clone + Send + 'static) -> AppFunction
where
    IV: IntoView + 'static,
{
    let any_view = app_fn().into_any();
    AppFunction::new(any_view)
}

// Provide a constructor for AppFunction so Pavex can generate it at will
// You'll need to put this inside another function so you can specify the function and not Pavex
pub fn generate_route_app_function<IV>(
    app_fn: impl Fn() -> IV + Clone + Send + 'static,
) -> RouteAppFunction
where
    IV: IntoView + 'static,
{
    let any_view = app_fn().into_any();
    RouteAppFunction::new(any_view)
}

//A type to hold the result of the App Function Closure so Pavex is happy
pub struct RouteAppFunction(AnyView<Dom>);
impl RouteAppFunction {
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

#[derive(Debug, Default)]
/// A dummy type that holds nothing, but allows us to return a value for the Pavex handler that'll
/// let the user provide stuff to Leptos from the server. Do not register this as a prebuilt type,
/// for it to work you need to build a constructor that calls `create_owner()` first!
pub struct AdditionalContextComponent(Owner);

impl AdditionalContextComponent {
    /// Give this type your additional context in a closure
    pub fn new(owner: Owner) -> Self {
        Self(owner)
    }

    pub fn owner(&self) -> &Owner {
        &self.0
    }
}

#[derive(Debug, Default)]
/// A dummy type that holds nothing, but allows us to return a value for the Pavex handler that'll
/// let the user provide stuff to Leptos from the server. Do not register this as a prebuilt type,
/// for it to work you need to build a constructor that calls `create_owner()` first!
pub struct AdditionalContextServerFn;
