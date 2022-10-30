use sycamore::prelude::*;
use sycamore_router::{HistoryIntegration, Route, Router};
use url::Url;
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
pub struct CommonComponentProps<'a, G: Html> {
    children: Children<'a, G>,
}

#[component]
fn Common<'a, G: Html>(cx: Scope<'a>, props: CommonComponentProps<'a, G>) -> View<G> {
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

#[derive(Clone, PartialEq, Eq)]
struct Warning {
    msg: String,
}

#[derive(Prop)]
struct WarningComponentProps {
    warning: Warning,
}

#[component]
fn WarningComponent<G: Html>(cx: Scope, props: WarningComponentProps) -> View<G> {
    view! {cx,
    div(
        class="flex items-center alert alert-warning",
        role="alert",
        ) {
        span(
            class="inline flex-shrink-0 mr-3 w-6 h-6 stroke-2",
            dangerously_set_inner_html="<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-linecap='round' stroke-linejoin='round' class='feather feather-alert-circle'><circle cx='12' cy='12' r='10'></circle><line x1='12' y1='8' x2='12' y2='12'></line><line x1='12' y1='16' x2='12.01' y2='16'></line></svg>",
            ){}
        span(dangerously_set_inner_html=props.warning.msg.as_str()){}
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
    let url_str = create_signal(cx, String::new());
    let guid = create_signal(cx, String::new());
    let warnings = create_signal(cx, vec![]);

    create_effect(cx, || {
        // Trim whitespace.
        url_str.set(url_str.get().trim().to_string());

        let (new_guid, new_warnings) = update_guid(url_str.get().to_string());

        match new_guid {
            Some(new_guid) => guid.set(new_guid),
            None => guid.set("".to_string()),
        };
        warnings.set(new_warnings);
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
                bind:value=url_str,
                )
        }

        (if warnings.get().len() != 0{
            view! { cx,
                Indexed(
                    iterable=warnings,
                    view=|cx, warning| view! { cx, WarningComponent(warning=warning)}
                    )
            }
        } else {
            view! { cx, }
        })


        (if !guid.get().is_empty() {
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

fn update_guid(url_str: String) -> (Option<String>, Vec<Warning>) {
    const NAMESPACE_PODCAST: uuid::Uuid = uuid::Uuid::from_bytes([
        0xea, 0xd4, 0xc2, 0x36, 0xbf, 0x58, 0x58, 0xc6, 0xa2, 0xc6, 0xa6, 0xb2, 0x8d, 0x12, 0x8c,
        0xb6,
    ]);

    let mut warnings = vec![];

    if url_str.is_empty() {
        return (None, vec![]);
    }

    if url_str.ends_with('/') {
        warnings.push(Warning {
            msg: "Trailing slashes should be removed from the URL.".to_string(),
        });
    }

    if let Ok(url) = Url::parse(url_str.as_str()) {
        let scheme_str = format!("{}://", url.scheme());
        let msg = if url_str.starts_with(scheme_str.as_str()) {
            format!(
                        "Protocol scheme “<span class='font-mono'>{}</span>” should be removed from the URL.",
                        scheme_str,
                    )
        // For some protocols, the format might be different.
        } else {
            "Protocol scheme should be removed from the URL.".to_string()
        };
        warnings.push(Warning { msg });
    } else {
        let new_url_str = format!("https://{}", url_str);
        if Url::parse(new_url_str.as_str()).is_err() {
            warnings.push(Warning {
                msg: "This does not appear to be a valid URL.".to_string(),
            });
        }
    }

    let uuid = Uuid::new_v5(&NAMESPACE_PODCAST, url_str.as_bytes()).to_string();

    (Some(uuid), warnings)
}
