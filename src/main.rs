use sycamore::prelude::*;
use sycamore_router::{HistoryIntegration, Route, Router};
use uuid::Uuid;

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
                integration=HistoryIntegration::new(),
                view=|cx, route: &ReadSignal<AppRoutes>| {
                    view! { cx,
                        Common {
                            (match route.get().as_ref() {
                                AppRoutes::Index => view!{ cx, Index{}},
                                AppRoutes::PodcastGuid => view!{ cx, PodcastGuid{}},
                                AppRoutes::NotFound => view! { cx,
                                    "404 Not Found"
                                },
                            })
                        }
                    }
                }
            )
        }
    });
}

#[derive(Prop)]
pub struct MyComponentProps<'a, G: Html> {
    children: Children<'a, G>,
}

#[component]
fn Common<'a, G: Html>(cx: Scope<'a>, props: MyComponentProps<'a, G>) -> View<G> {
    let children = props.children.call(cx);
    view! { cx,
    a(href="https://rssblue.com") {
        img(
            src="https://rssblue.com/static/dist/img/logo.svg",
            class="h-10 mx-auto mt-3 mb-7",
            alt="RSS Blue Logo",
            ){}
    }
    main(class="flex-grow") {
        div(class="pt-4 pb-7 mx-auto px-2 lg:px-0 max-w-prose") {
        (children)
        }
    }
    }
}

#[component]
fn Index<G: Html>(cx: Scope) -> View<G> {
    view! { cx,
    h1 { "Tools" }
    div(class="grid grid-cols-1 md:grid-cols-2 gap-2") {
        a(
            class=format!("btn btn-primary"),
            href="/podcast-guid",
            ) { "Podcast GUID" }
    }
    }
}

#[component]
fn PodcastGuid<G: Html>(cx: Scope) -> View<G> {
    const NAMESPACE_PODCAST: uuid::Uuid = uuid::Uuid::from_bytes([
        0xea, 0xd4, 0xc2, 0x36, 0xbf, 0x58, 0x58, 0xc6, 0xa2, 0xc6, 0xa6, 0xb2, 0x8d, 0x12, 0x8c,
        0xb6,
    ]);

    let url = create_signal(cx, String::new());
    let guid = create_signal(cx, String::new());
    create_effect(cx, || {
        // Trim whitespace.
        url.set(url.get().trim().to_string());
        let uuid = Uuid::new_v5(&NAMESPACE_PODCAST, url.get().as_bytes());
        guid.set(uuid.to_string());
    });

    view! { cx,
    h1 { "Podcast GUID" }
    h2(class="text-gray-500") { "Generate a unique, global identifier for your podcast." }
    form(class="space-y-4") {
        // Prevent submission with "Enter".
        button(
            type="submit",
            disabled=true,
            style="display: none",
            aria-hidden="true"
            ){}
        div{
            label(for="url") { "Podcast feed's URL" }
            input(
                class="input-text",
                type="url",
                id="url",
                placeholder="example.com/podcast-feed",
                autocomplete="off",
                bind:value=url,
                )
        }
        (if !url.get().is_empty() {
            view! { cx,
            div {
                "GUID"
                div(
                    type="text",
                    class="input-text select-all font-mono",
                    ) {
                    (guid.get())
                }
            }
            }
        } else {
            view! { cx, }
        })

    }

    }
}
