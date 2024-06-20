#[derive(Debug, Default)]
/// A dummy type that holds nothing, but allows us to return a value for the Pavex handler that'll 
/// let the user provide stuff to Leptos from the server. Do not register this as a prebuilt type,
/// for it to work you need to build a constructor that calls `create_owner()` first!
pub struct AdditionalContextComponent;

impl AdditionalContextComponent{
    /// Give this type your additional context in a closure
    pub fn new(context_fn: impl Fn() + 'static + Clone + Send ){
        context_fn();
    }
}

#[derive(Debug, Default)]
/// A dummy type that holds nothing, but allows us to return a value for the Pavex handler that'll 
/// let the user provide stuff to Leptos from the server. Do not register this as a prebuilt type,
/// for it to work you need to build a constructor that calls `create_owner()` first!
pub struct AdditionalContextServerFn;