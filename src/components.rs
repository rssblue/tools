use sycamore::prelude::*;

mod index;
pub use index::Index;

mod podcast_guid;
pub use podcast_guid::PodcastGuid;

mod plot_op3;
pub use plot_op3::PlotOp3;

pub mod utils;

pub mod hyper_header;

#[derive(Prop)]
pub struct CommonProps<'a, G: Html> {
    children: Children<'a, G>,
}

#[component]
pub fn Common<'a, G: Html>(cx: Scope<'a>, props: CommonProps<'a, G>) -> View<G> {
    let children = props.children.call(cx);
    view! { cx,
    Nav {}
    main(class="flex-grow") {
        div(class="pt-4 pb-7 mx-auto px-2 lg:px-0 max-w-prose") {
            (children)
        }
    }
    Footer {}
    }
}

#[component]
pub fn Nav<G: Html>(cx: Scope) -> View<G> {
    view! { cx,
    nav {
        a(href="https://rssblue.com") {
            img(
                src="https://rssblue.com/static/dist/img/logo.svg",
                class="h-10 mx-auto mt-3 mb-7",
                alt="RSS Blue Logo",
                ){}
        }
    }
    }
}

#[component]
pub fn Footer<G: Html>(cx: Scope) -> View<G> {
    view! { cx,
    footer(class="not-prose border-gray-200 py-2 border-t text-center") {
        a(
            class="link font-mono text-gray-400 decoration-primary-100",
            href=format!("https://github.com/rssblue/tools/commit/{}", env!("GIT_HASH")),
            target="_blank",
            rel="noopener",
            title="Opens in a new tab",
            ) { (format!("git:{}", env!("GIT_HASH_SHORT"))) }
    }
    }
}

#[component(inline_props)]
pub fn ToolsBreadcrumbs<G: Html>(cx: Scope, title: &'static str) -> View<G> {
    view! { cx,
    nav(class="flex not-prose mb-2", aria-label="Breadcrumb") {
        ol(class="inline-flex items-center space-x-1 md:space-x-2 text-gray-400") {
            ToolsBreadcrumbItem(url=Some("https://rssblue.com"), title="RSS Blue", with_chevron=false)
            ToolsBreadcrumbItem(url=Some("/"), title="Tools", with_chevron=true)
            ToolsBreadcrumbItem(url=None, title=title, with_chevron=true)
        }
    }
    }
}

#[component(inline_props)]
fn ToolsBreadcrumbItem<G: Html>(
    cx: Scope,
    url: Option<&'static str>,
    title: &'static str,
    with_chevron: bool,
) -> View<G> {
    view! { cx,
    li(class="inline-flex items-center", aria-current="page") {
        (if with_chevron {
            view! { cx,
            span(
                dangerously_set_inner_html=utils::Icon::ChevronRight.to_string().replace("{{ class }}", "h-4 stroke-2 mr-1 md:mr-2").as_str(),
                ){}
            }
        } else {
            view! { cx, }
        })
        (if let Some(url) = url {
            view! { cx,
            a(href=url, class="inline-flex items-center text-sm font-medium link link-primary no-underline") {
                (title)
            }
            }
        } else {
            view! { cx,
            span(class="inline-flex items-center text-sm font-medium text-gray-500") {
                (title)
            }
            }
        })
    }
    }
}
