use sycamore::prelude::*;
use url::Url;
use uuid::Uuid;

#[derive(Prop)]
pub struct CommonProps<'a, G: Html> {
    children: Children<'a, G>,
}

#[component]
pub fn Common<'a, G: Html>(cx: Scope<'a>, props: CommonProps<'a, G>) -> View<G> {
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

enum Icon {
    AlertCircle(String),
}

impl std::fmt::Display for Icon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::AlertCircle(classes) => format!("<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-linecap='round' stroke-linejoin='round' class='{classes}'><circle cx='12' cy='12' r='10'></circle><line x1='12' y1='8' x2='12' y2='12'></line><line x1='12' y1='16' x2='12.01' y2='16'></line></svg>"),
        };
        write!(f, "{s}")
    }
}

#[component(inline_props)]
fn Warning<G: Html>(cx: Scope, warning: String) -> View<G> {
    view! {cx,
    div(
        class="flex items-center alert alert-warning",
        role="alert",
        ) {
        span(
            dangerously_set_inner_html=Icon::AlertCircle("inline flex-shrink-0 mr-3 w-6 h-6 stroke-2".to_string()).to_string().as_str(),
            ){}
        span(dangerously_set_inner_html=warning.as_str()){}
    }
    }
}

#[component]
pub fn Index<G: Html>(cx: Scope) -> View<G> {
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
pub fn PodcastGuid<G: Html>(cx: Scope) -> View<G> {
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
    h1(class="mb-3") { "Podcast GUID" }
    h2(class="mt-3 text-gray-500") { "Generate a unique, global identifier for your podcast." }
    p(class="my-7") {
            span(class="font-mono") { "<podcast:guid>" }
        " is part of the podcast namespace initiative and aims to provide podcasts with a consistent identity across the RSS ecosystem. Learn more "
            a(
                class="link",
                href="https://podcastindex.org/namespace/1.0#guid",
                target="_blank",
                rel="noopener",
                title="Opens in a new tab",
                ) { "here" }
        "."
    }
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
                spellcheck=false,
                autofocus=true,
                type="url",
                id="url",
                placeholder="example.com/feed.xml",
                autocomplete="off",
                bind:value=url_str,
                )
        }

        (if warnings.get().len() != 0{
            view! { cx,
                Indexed(
                    iterable=warnings,
                    view=|cx, warning| view! { cx, Warning(warning=warning)}
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

fn update_guid(url_str: String) -> (Option<String>, Vec<String>) {
    const NAMESPACE_PODCAST: uuid::Uuid = uuid::Uuid::from_bytes([
        0xea, 0xd4, 0xc2, 0x36, 0xbf, 0x58, 0x58, 0xc6, 0xa2, 0xc6, 0xa6, 0xb2, 0x8d, 0x12, 0x8c,
        0xb6,
    ]);

    let mut warnings = vec![];

    if url_str.is_empty() {
        return (None, vec![]);
    }

    if url_str.ends_with('/') {
        warnings.push(
            "To generate a valid GUID, trailing slashes should be removed from the URL."
                .to_string(),
        );
    }

    if let Ok(url) = Url::parse(url_str.as_str()) {
        let scheme_str = format!("{}://", url.scheme());
        let msg = if url_str.starts_with(scheme_str.as_str()) {
            format!(
                        "To generate a valid GUID, protocol scheme “<span class='font-mono'>{}</span>” should be removed from the URL.",
                        scheme_str,
                    )
        // For some protocols, the format might be different.
        } else {
            "To generate a valid GUID, protocol scheme should be removed from the URL.".to_string()
        };
        warnings.push(msg);
    } else {
        let new_url_str = format!("https://{}", url_str);
        if Url::parse(new_url_str.as_str()).is_err() {
            warnings.push("This does not appear to be a valid URL.".to_string());
        }
    }

    let uuid = Uuid::new_v5(&NAMESPACE_PODCAST, url_str.as_bytes()).to_string();

    (Some(uuid), warnings)
}
