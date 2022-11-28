use crate::components::utils;
use sycamore::prelude::*;
use url::Url;
use uuid::Uuid;

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
    crate::components::ToolsBreadcrumbs(title="Podcast GUID")
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
                    view=|cx, warning| view! { cx, utils::Alert(type_=utils::AlertType::Warning, msg=warning)}
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
