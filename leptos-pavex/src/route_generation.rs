use std::cell::{Cell, RefCell};

use leptos::tachys::view::RenderHtml;
use leptos_router::RouteListing;

use crate::pavex_helpers::RouteAppFunction;

#[derive(Debug, Default)]
pub struct RouteList(Vec<RouteListing>);

impl From<Vec<RouteListing>> for RouteList {
    fn from(value: Vec<RouteListing>) -> Self {
        Self(value)
    }
}

impl RouteList {
    pub fn push(&mut self, data: RouteListing) {
        self.0.push(data);
    }
}

impl RouteList {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn into_inner(self) -> Vec<RouteListing> {
        self.0
    }
}

impl RouteList {
    // this is used to indicate to the Router that we are generating
    // a RouteList for server path generation
    thread_local! {
        static IS_GENERATING: Cell<bool> = const { Cell::new(false) };
        static GENERATED: RefCell<Option<RouteList>> = const { RefCell::new(None) };
    }

    pub fn generate(app: RouteAppFunction) -> Option<Self>
    where

    {
        Self::IS_GENERATING.set(true);
        // run the app once, but throw away the HTML
        // the router won't actually route, but will fill the listing
        _ = app.inner().to_html();
        Self::IS_GENERATING.set(false);
        Self::GENERATED.take()
    }

    pub fn is_generating() -> bool {
        Self::IS_GENERATING.get()
    }

    pub fn register(routes: RouteList) {
        Self::GENERATED.with(|inner| {
            *inner.borrow_mut() = Some(routes);
        });
    }
}