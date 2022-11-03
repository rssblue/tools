use sycamore::prelude::*;
use sycamore_router::{HistoryIntegration, Route, Router};

mod components;

#[derive(Route)]
enum AppRoutes {
    #[to("/")]
    Index,
    #[to("/podcast-guid")]
    PodcastGuid,
    #[to("/plot-op3")]
    PlotOp3,
    #[not_found]
    NotFound,
}

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    sycamore::render(|cx| {
        view! { cx,
            Router(
                integration=HistoryIntegration::new(),
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
                AppRoutes::PlotOp3 => view!{ cx, components::PlotOp3{}},
                AppRoutes::NotFound => view! { cx,
                    "404 Not Found"
                },
            })
        }
    }
}
