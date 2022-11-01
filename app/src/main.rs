use sycamore::prelude::*;
use sycamore_router::{Integration, Route, Router};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

mod components;

#[derive(Route)]
enum AppRoutes {
    #[to("/")]
    Index,
    #[to("/podcast-guid")]
    PodcastGuid,
    #[not_found]
    NotFound,
}

fn main() {
    sycamore::render(|cx| {
        view! { cx,
            Router(
                integration=SimpleIntegration::new(),
                view=switch,
            )
        }
    });
}

fn switch<'a, G: Html>(cx: Scope<'a>, route: &'a ReadSignal<AppRoutes>) -> View<G> {
    view! { cx,
        components::Common {
            (match route.get().as_ref() {
                AppRoutes::Index => view!{ cx, components::Index{}},
                AppRoutes::PodcastGuid => view!{ cx, components::PodcastGuid{}},
                AppRoutes::NotFound => view! { cx,
                    "404 Not Found"
                },
            })
        }
    }
}

#[derive(Default, Debug)]
pub struct SimpleIntegration {
    _internal: (),
}

impl SimpleIntegration {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Copy of [HistoryIntegration](sycamore_router::HistoryIntegration), except that nothing is done
/// for `click_handler`; instead content is fetched from the server.
impl Integration for SimpleIntegration {
    fn current_pathname(&self) -> String {
        web_sys::window()
            .unwrap_throw()
            .location()
            .pathname()
            .unwrap_throw()
    }

    fn on_popstate(&self, f: Box<dyn FnMut()>) {
        let closure = Closure::wrap(f);
        web_sys::window()
            .unwrap_throw()
            .add_event_listener_with_callback("popstate", closure.as_ref().unchecked_ref())
            .unwrap_throw();
        closure.forget();
    }

    fn click_handler(&self) -> Box<dyn Fn(web_sys::Event)> {
        Box::new(|_| {})
    }
}
