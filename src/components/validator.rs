use crate::components::utils;
use sycamore::prelude::*;
use sycamore::suspense::{use_transition, Suspense};
use url::Url;
use wasm_bindgen::JsValue;

#[derive(Debug, Clone)]
struct ProgramError<G: Html> {
    description: View<G>,
    error: Option<(String, String)>,
}

#[derive(Debug, Clone, PartialEq)]
enum Value {
    Text(String),
    Object(String),
    Url(String),
}

fn md_to_html(md: &str) -> String {
    let mut options = comrak::ComrakOptions::default();
    options.parse.smart = true;

    let html = comrak::markdown_to_html(md, &options).trim().to_string();

    if html.starts_with("<p>") && html.ends_with("</p>") {
        html.trim_start_matches("<p>")
            .trim_end_matches("</p>")
            .to_string()
    } else {
        html
    }
}

#[component]
pub fn Validator<G: Html>(cx: Scope) -> View<G> {
    let _program_error: Option<ProgramError<G>> = None;
    const VALIDATOR_STORAGE_KEY_USE_PROXY: &str = "validator_use_proxy";
    let program_error = create_signal(cx, _program_error);

    let mut url_in_url = String::new();
    // Get 'url' query parameter.
    if let Some(window) = web_sys::window() {
        if let Ok(href) = window.location().href() {
            if let Ok(url) = web_sys::Url::new(&href) {
                if let Some(param) = url.search_params().get("url") {
                    url_in_url = param;
                }
            }
        }
    }

    let input_cls = create_signal(cx, String::new());
    let fetching_data = create_signal(cx, false);
    let url_str = create_signal(cx, String::new());
    let transition = use_transition(cx);
    let show_results = create_signal(cx, false);
    // Use CORS proxy to avoid CORS issues.
    let use_proxy = create_signal(cx, false);

    // Initialize proxy.
    match utils::get_from_storage(VALIDATOR_STORAGE_KEY_USE_PROXY) {
        Ok(Some(use_proxy_storage)) => match use_proxy_storage.as_str() {
            "true" => {
                use_proxy.set(true);
            }
            _ => {
                use_proxy.set(false);
            }
        },
        Ok(None) => {}
        Err(e) => {
            program_error.set(Some(ProgramError {
                description: view! { cx, "Failed to get use_proxy from storage" },
                error: Some(("Original error".to_string(), e)),
            }));
        }
    };

    create_effect(cx, move || {
        let result = if *use_proxy.get() {
            utils::set_in_storage(VALIDATOR_STORAGE_KEY_USE_PROXY, "true")
        } else {
            utils::remove_from_storage(VALIDATOR_STORAGE_KEY_USE_PROXY)
        };
        if let Err(e) = result {
            program_error.set(Some(ProgramError {
                description: view! { cx, "Error when accessing storage to update the settings" },
                error: Some(("Original error".to_string(), e)),
            }));
        }
    });

    let fetch_feed = move |x| transition.start(move || fetching_data.set(x), || ());

    create_effect(cx, move || {
        if *fetching_data.get() {
            input_cls.set("bg-gray-100".to_string());
        } else {
            input_cls.set("".to_string());
        }
    });

    create_effect(cx, move || {
        if *fetching_data.get() {
            show_results.set(true);
        }
    });

    create_effect(cx, move || fetching_data.set(transition.is_pending()));

    if !url_in_url.is_empty() {
        url_str.set(url_in_url);
        fetch_feed(true);
    }

    view! { cx,
            crate::components::ToolsBreadcrumbs(title="Podcast Validator")

            h1(class="mb-3") { "Podcast Validator" }
            h2(class="mt-3 text-gray-500") { "Make sure your Podcasting 2.0 feed is valid." }
            p(class="mt-7") {
                utils::Link(url="https://podcastindex.org/namespace/1.0".to_string(), text="Podcast namespace initiative".to_string(), new_tab=true)
                " is a community effort to create modern podcasting standards. If you utilize any of the new "
                code { "<podcast:*>" }
            " XML tags, this tool will check for any mistakes in your feed."
        }

        p(class="mb-7") {
            "This validator only checks the " em { "podcast namespace" } " elements and only analyzes the " em { "feed" } " itself. For other namespaces and media checks, you can try "
        utils::Link(url="https://validator.livewire.io/".to_string(), text="Livewire Podcast Validator".to_string(), new_tab=true)
        ", "
        utils::Link(url="https://www.castfeedvalidator.com/".to_string(), text="Cast Feed Validator".to_string(), new_tab=true)
        ", and "
        utils::Link(url="https://podba.se/validate/".to_string(), text="Podbase Podcast Validator".to_string(), new_tab=true)
        "."
    }

        form(class="mb-4 space-y-3") {
            // Prevent submission with "Enter".
            button(
                type="submit",
                disabled=true,
                style="display: none",
                aria-hidden="true"
            ){}
        div{
            label(for="url") { "Feed's URL" }
        div(class="grid grid-cols-4") {
            div(class="flex flex-row col-span-4 md:col-span-3") {
                input(
                    class=format!("input-text-base rounded-t-xl md:rounded-l-xl md:rounded-r-none text-ellipsis z-10 {}", input_cls.get()),
                    spellcheck=false,
                    autofocus=true,
                    type="url",
                    id="url",
                    placeholder="https://example.com/feed.xml",
                    autocomplete="off",
                    disabled=*fetching_data.get(),
                    bind:value=url_str,
                )
            }
            button(
                class=format!("btn-base btn-primary rounded-b-xl md:rounded-r-xl md:rounded-l-none col-span-4 md:col-span-1"),
                type="button",
                on:click=move |_| fetch_feed(true),
                disabled=*fetching_data.get(),
            ) {
                (if *fetching_data.get() {
                    "Loading..."
                } else {
                    "Test feed"
                })
            }
        }

    }

        div(class="flex flex-row items-center") {
        div(class="cursor-pointer") {
        input(
            id="use-proxy",
            type="checkbox",
            class="input-checkbox",
            bind:checked=use_proxy,
        )
            label(class="ml-3 cursor-pointer", for="use-proxy") {
            "Route requests through RSS Blue"
        }
    }
    }
    }

        (if *show_results.get() {
            view!{cx,
                Suspense(fallback=view! { cx, }) {
                    Validate(url=url_str.get().to_string(), use_proxy=*use_proxy.get())
                }
            }
        } else {
                view! { cx,
                }
            })

            (if program_error.get().is_some() {
                let error = &*program_error.get();
                view! { cx, DisplayProgramError(program_error=error.clone().unwrap()) }
            } else {
                    view! { cx, }
                })

    }
}

#[component(inline_props)]
pub async fn Validate<'a, G: Html>(cx: Scope<'a>, url: String, use_proxy: bool) -> View<G> {
    const CORS_PROXY_URL: &str = "https://proxy.rssblue.com?url=";

    // Set 'url' query parameter.
    if let Some(window) = web_sys::window() {
        if let Ok(href) = window.location().href() {
            if let Ok(web_url) = web_sys::Url::new(&href) {
                web_url.search_params().set("url", &url);
                let new_url = web_url.href();
                if let Ok(history) = window.history() {
                    if history
                        .push_state_with_url(&JsValue::NULL, "", Some(&new_url))
                        .is_err()
                    {
                        web_sys::console::error_1(&"Error setting query parameter".into());
                    }
                }
            }
        }
    }

    let url = match Url::parse(&url) {
        Ok(url) => url,
        Err(e) => {
            return view! { cx,
                utils::Alert(type_=utils::AlertType::Danger, msg=format!("Could not parse the URL ({e})"))
            };
        }
    };
    if url.scheme() != "http" && url.scheme() != "https" {
        return view! { cx,
            utils::Alert(type_=utils::AlertType::Danger, msg="URL protocol must be http or https".to_string())
        };
    }

    let url = if use_proxy {
        format!("{}{}", CORS_PROXY_URL, url)
    } else {
        url.to_string()
    };

    let resp = reqwest_wasm::get(url).await;

    let resp = match resp {
        Ok(x) => x,
        Err(e) => {
            let mut description = view! { cx, "Could not fetch the feed." };
            if e.is_request() && !use_proxy {
                description = view! { cx,
                        "Could not make the request. This could be due to a "
                        utils::Link(url="https://developer.mozilla.org/en-US/docs/Web/HTTP/CORS".to_string(), text="CORS".to_string(), new_tab=true)
                        " error, so you can try routing requests through RSS Blue by clicking on the checkbox above."

                        details(class="mt-2") {
                            summary(class="font-bold") { "CORS for feed hosts" }
                            "If you control the server hosting the feed, you can add the following header to the HTTP response to allow CORS requests from any origin:"
                                pre(class="p-1") {
                                "Access-Control-Allow-Origin: *"
                            }
                            "Security-wise, this may not be optimal in every scenario, so we recommend "
                            utils::Link(url="https://developer.mozilla.org/en-US/docs/Web/HTTP/CORS".to_string(), text="reading more about CORS".to_string(), new_tab=true)
                            " to understand what is the best solution for " em { "you" } "."
                    }
                };
            }
            let program_error = ProgramError {
                description,
                error: Some(("Original error".to_string(), e.to_string())),
            };
            return view! { cx, DisplayProgramError(program_error=program_error) };
        }
    };

    let status = resp.status();

    let text = match resp.text().await {
        Ok(x) => x,
        Err(e) => {
            return view! {cx,
                utils::Alert(type_=utils::AlertType::Danger, msg=format!("Could not fetch the feed ({e})"))
            }
        }
    };

    if !status.is_success() {
        let program_error = ProgramError {
            description: view! { cx, (format!("Could not fetch the feed ({})", status)) },
            error: Some(("Response".to_string(), text)),
        };
        return view! { cx, DisplayProgramError(program_error=program_error) };
    }

    let feed = match badpod::from_str(&text) {
        Ok(x) => x,
        Err(e) => {
            return view! {cx,
                utils::Alert(type_=utils::AlertType::Danger, msg=format!("Could not parse the feed ({e})"))
            }
        }
    };

    let root_node = analyze_rss(&feed);

    view! { cx,
        DisplayNode(node=root_node, is_root=true)
    }
}

#[derive(PartialEq, Debug, Clone)]
enum Namespace {
    Podcast,
}

#[derive(PartialEq, Debug, Clone, Default)]
struct TagName(Option<Namespace>, String);

#[derive(PartialEq, Clone, Default)]
struct Node {
    name: TagName,
    children: Vec<Node>,
    attributes: Vec<(String, Value)>,
    errors: Vec<Error>,
}

#[derive(PartialEq, Clone)]
enum Error {
    MissingAttribute(String),
    InvalidAttribute(String, String),
    InvalidAttributeWithReason(String, String, String),
    MissingChild(TagName),
    MultipleChildren(TagName),
    AttributeExceedsMaxLength(String, String, usize),
    Custom(String),
    CustomWithExtraInfo(String, String),
}

const NODE_VALUE: &str = "node value";

#[component(inline_props)]
fn DisplayError<'a, G: Html>(cx: Scope<'a>, error: Error) -> View<G> {
    match error {
        Error::MissingAttribute(attr) => {
            if attr == NODE_VALUE {
                view! { cx,
                    div(class="text-danger-500") {
                        "Missing node value"
                    }
                }
            } else {
                view! { cx,
                    div(class="text-danger-500") {
                        "Missing attribute "
                            code(class="attr") { (attr) }
                    }
                }
            }
        }
        Error::InvalidAttribute(attr, value) => {
            if attr == NODE_VALUE {
                view! { cx,
                        div(class="text-danger-500") {
                            "Invalid node value "
                            code { "“" (value) "”" }
                    }
                }
            } else {
                view! { cx,
                        div(class="text-danger-500") {
                            "Attribute "
                                code(class="attr") { (attr) }
                            " has invalid value "
                            code { "“" (value) "”" }
                    }
                }
            }
        }
        Error::InvalidAttributeWithReason(attr, value, reason) => {
            if attr == NODE_VALUE {
                view! { cx,
                        span(class="text-danger-500") {
                            "Invalid node value "
                            code { "“" (value) "”" }
                    }
                    span(class="text-gray-500") {
                        ": "
                        span(
                            class="from-md",
                            dangerously_set_inner_html=&md_to_html(&reason),
                        ){}
                    }
                }
            } else {
                view! { cx,
                        span(class="text-danger-500") {
                            "Attribute "
                                code(class="attr") { (attr) }
                            " has invalid value "
                            code { "“" (value) "”" }
                    }
                    span(class="text-gray-500") {
                        ": "
                        span(
                            class="from-md",
                            dangerously_set_inner_html=&md_to_html(&reason),
                        ){}

                    }
                }
            }
        }
        Error::MissingChild(tag_name) => {
            view! { cx,
                    div(class="text-danger-500") {
                        "Missing child "
                        code { "<" (tag_name) ">" }
                }
            }
        }
        Error::MultipleChildren(tag_name) => {
            view! { cx,
                    div(class="text-danger-500") {
                        "Only one child "
                        code { "<" (tag_name) ">" }
                    " is allowed"
                }
            }
        }
        Error::AttributeExceedsMaxLength(attr, value, max_len) => {
            if attr == NODE_VALUE {
                view! { cx,
                        div(class="text-danger-500") {
                            "Node value "
                            code { "“" (value) "”" }
                        " exceeds maximum length of "
                        code { (max_len) }
                    " characters"
                }
                }
            } else {
                view! { cx,
                        div(class="text-danger-500") {
                            "Attribute "
                                code(class="attr") { (attr) }
                            " exceeds maximum length of "
                            code { (max_len) }
                        " characters"
                    }
                }
            }
        }
        Error::Custom(msg) => {
            view! { cx,
                div(class="text-danger-500", dangerously_set_inner_html=msg.as_str()) {}
            }
        }
        Error::CustomWithExtraInfo(msg, extra_info) => {
            let show_extra_info = create_signal(cx, false);

            let extra_info = move || {
                if *show_extra_info.get() {
                    extra_info.clone()
                } else {
                    "Learn more".to_string()
                }
            };

            let extra_info_cls = move || {
                if *show_extra_info.get() {
                    ""
                } else {
                    "link cursor-pointer"
                }
            };

            view! { cx,
                div {
                div(class="text-danger-500", dangerously_set_inner_html=msg.as_str()) {}
                }
                div {
                    span(
                        class=extra_info_cls(),
                        on:click=move |_| show_extra_info.set(true),
                    ) {
                        span(dangerously_set_inner_html=extra_info().as_str())
                    }
                }
            }
        }
    }
}

impl std::fmt::Display for TagName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TagName(Some(Namespace::Podcast), x) => write!(f, "podcast:{}", x),
            TagName(None, x) => write!(f, "{}", x),
        }
    }
}

impl Node {
    fn descendants_have_errors(&self) -> bool {
        if !self.errors.is_empty() {
            return true;
        }

        for child in &self.children {
            if child.descendants_have_errors() {
                return true;
            }
        }

        false
    }

    fn descendants_have_podcast_tags(&self) -> bool {
        if self.name.0 == Some(Namespace::Podcast) {
            return true;
        }

        for child in &self.children {
            if child.descendants_have_podcast_tags() {
                return true;
            }
        }

        false
    }
}

#[component(inline_props)]
fn DisplayNode<'a, G: Html>(cx: Scope<'a>, node: Node, is_root: bool) -> View<G> {
    let children = create_signal(cx, node.children.clone());
    let errors = create_signal(cx, node.errors.clone());
    let attributes = create_signal(cx, node.attributes.clone());
    let have_nested_errors = node.descendants_have_errors();
    let have_podcast_tags = node.descendants_have_podcast_tags();
    let name_cls = if have_nested_errors {
        "text-danger-500"
    } else {
        ""
    };
    let details_cls = if is_root { "overflow-x-auto" } else { "" };

    view! { cx,
            (match (is_root, have_podcast_tags, have_nested_errors) {
                (true, false, _) => view! { cx,
                    div(class="mb-5") {
                        utils::Alert(type_=utils::AlertType::Info, msg="No podcast namespace tags found.".to_string())
                    }
                },
                (true, true, false) => view! { cx,
                    div(class="mb-5") {
                        utils::Alert(type_=utils::AlertType::Success, msg="Our analysis has not found any errors in the podcast namespace tags.".to_string())
                    }
                },
                _ => view! { cx, },
            })

            details(class=details_cls, open=have_nested_errors) {
            summary(class=name_cls) {
                code(class="font-bold") { "<"(node.name)">" }
    }
    div(class="pl-1") {
                    div(class="pl-2 md:pl-4 border-l-2 border-gray-200") {
                        ul(class="text-sm my-0") {
                            Indexed(
                                iterable=errors,
                                view=|cx, x| view! { cx,
                                    li(class="my-0 marker:text-danger-500") { DisplayError(error=x) }
                                },
                            )
                                Indexed(
                                    iterable=attributes,
                                    view=|cx, x| view! { cx,
                                        li(class="my-0") {
                                            code { span(class="attr") { (x.0) }  "=" }
                                            (match &x.1 {
                                                Value::Text(s) => {
                                                    let s = s.to_string();
                                                    view! { cx, "“" (s) "”" }
                                                },
                                                Value::Object(s) => {
                                                    let s = s.to_string();
                                                    view! { cx, span(class="italic") { (s) } }
                                                },
                                                Value::Url(s) => {
                                                    let s = s.to_string();
                                                    if s.starts_with("http") {
                                                        view! { cx, utils::Link(url=s.to_string(), text=s.to_string(), new_tab=true) }
                                                    } else {
                                                        view! { cx, span(class="italic") { (s) } }
                                                    }
                                                }
                                            }
                                            )
                                        }
                                    },
                                )
                        }

                        Indexed(
                            iterable=children,
                            view=|cx, x| view! { cx,
                                DisplayNode(node=x, is_root=false)
                            },
                        )
                    }
                }
            }
        }
}

fn analyze_rss(rss: &badpod::Rss) -> Node {
    let mut errors = Vec::new();
    let mut children = Vec::new();

    for channel in &rss.channel {
        children.push(analyze_channel(channel));
    }
    match rss.channel.len() {
        0 => errors.push(Error::MissingChild(TagName(None, "channel".to_string()))),
        1 => (),
        _ => errors.push(Error::MultipleChildren(TagName(
            None,
            "channel".to_string(),
        ))),
    }

    Node {
        name: TagName(None, "rss".to_string()),
        children,
        ..Default::default()
    }
}

fn analyze_channel(channel: &badpod::Channel) -> Node {
    let mut errors = Vec::new();
    let mut children = Vec::new();

    for title in &channel.title {
        children.push(analyze_title(title.to_string()));
    }
    match channel.title.len() {
        0 => errors.push(Error::MissingChild(TagName(None, "title".to_string()))),
        1 => {}
        _ => errors.push(Error::MultipleChildren(TagName(None, "title".to_string()))),
    }

    for guid in &channel.podcast_guid {
        children.push(analyze_podcast_guid(guid));
    }
    if channel.podcast_guid.len() > 1 {
        errors.push(Error::MultipleChildren(TagName(
            Some(Namespace::Podcast),
            "guid".to_string(),
        )));
    }

    for medium in &channel.podcast_medium {
        children.push(analyze_podcast_medium(medium));
    }
    if channel.podcast_medium.len() > 1 {
        errors.push(Error::MultipleChildren(TagName(
            Some(Namespace::Podcast),
            "medium".to_string(),
        )));
    }

    for txt in &channel.podcast_txt {
        children.push(analyze_podcast_txt(txt));
    }

    for block in &channel.podcast_block {
        children.push(analyze_podcast_block(block));
    }

    for locked in &channel.podcast_locked {
        children.push(analyze_podcast_locked(locked));
    }
    if channel.podcast_locked.len() > 1 {
        errors.push(Error::MultipleChildren(TagName(
            Some(Namespace::Podcast),
            "locked".to_string(),
        )));
    }

    for funding in &channel.podcast_funding {
        children.push(analyze_podcast_funding(funding));
    }

    for location in &channel.podcast_location {
        children.push(analyze_podcast_location(location));
    }
    if channel.podcast_location.len() > 1 {
        errors.push(Error::MultipleChildren(TagName(
            Some(Namespace::Podcast),
            "location".to_string(),
        )));
    }

    for person in &channel.podcast_person {
        children.push(analyze_podcast_person(person));
    }

    for trailer in &channel.podcast_trailer {
        children.push(analyze_podcast_trailer(trailer));
    }

    for license in &channel.podcast_license {
        children.push(analyze_podcast_license(license));
    }
    if channel.podcast_license.len() > 1 {
        errors.push(Error::MultipleChildren(TagName(
            Some(Namespace::Podcast),
            "license".to_string(),
        )));
    }

    for v4v_value in &channel.podcast_value {
        children.push(analyze_podcast_value(v4v_value));
    }

    for images in &channel.podcast_images {
        children.push(analyze_podcast_images(images));
    }
    if channel.podcast_images.len() > 1 {
        errors.push(Error::MultipleChildren(TagName(
            Some(Namespace::Podcast),
            "images".to_string(),
        )));
    }

    for item in &channel.item {
        children.push(analyze_item(item));
    }

    for live_item in &channel.podcast_live_item {
        children.push(analyze_podcast_live_item(live_item));
    }

    Node {
        name: TagName(None, "channel".to_string()),
        children,
        errors,
        attributes: Vec::new(),
    }
}

fn analyze_item(item: &badpod::Item) -> Node {
    let mut children = Vec::new();
    let mut errors = Vec::new();

    for title in &item.title {
        children.push(analyze_title(title.to_string()));
    }
    match item.title.len() {
        0 => errors.push(Error::MissingChild(TagName(None, "title".to_string()))),
        1 => {}
        _ => errors.push(Error::MultipleChildren(TagName(None, "title".to_string()))),
    }

    for v4v_value in &item.podcast_value {
        children.push(analyze_podcast_value(v4v_value));
    }

    for person in &item.podcast_person {
        children.push(analyze_podcast_person(person));
    }

    for location in &item.podcast_location {
        children.push(analyze_podcast_location(location));
    }
    if item.podcast_location.len() > 1 {
        errors.push(Error::MultipleChildren(TagName(
            Some(Namespace::Podcast),
            "location".to_string(),
        )));
    }

    for images in &item.podcast_images {
        children.push(analyze_podcast_images(images));
    }
    if item.podcast_images.len() > 1 {
        errors.push(Error::MultipleChildren(TagName(
            Some(Namespace::Podcast),
            "images".to_string(),
        )));
    }

    for txt in &item.podcast_txt {
        children.push(analyze_podcast_txt(txt));
    }

    for transcript in &item.podcast_transcript {
        children.push(analyze_podcast_transcript(transcript));
    }

    for chapters in &item.podcast_chapters {
        children.push(analyze_podcast_chapters(chapters));
    }
    if item.podcast_chapters.len() > 1 {
        errors.push(Error::MultipleChildren(TagName(
            Some(Namespace::Podcast),
            "chapters".to_string(),
        )));
    }

    for soundbite in &item.podcast_soundbite {
        children.push(analyze_podcast_soundbite(soundbite));
    }

    for season in &item.podcast_season {
        children.push(analyze_podcast_season(season));
    }
    if item.podcast_season.len() > 1 {
        errors.push(Error::MultipleChildren(TagName(
            Some(Namespace::Podcast),
            "season".to_string(),
        )));
    }

    for episode in &item.podcast_episode {
        children.push(analyze_podcast_episode(episode));
    }
    if item.podcast_episode.len() > 1 {
        errors.push(Error::MultipleChildren(TagName(
            Some(Namespace::Podcast),
            "episode".to_string(),
        )));
    }

    for license in &item.podcast_license {
        children.push(analyze_podcast_license(license));
    }
    if item.podcast_license.len() > 1 {
        errors.push(Error::MultipleChildren(TagName(
            Some(Namespace::Podcast),
            "license".to_string(),
        )));
    }

    for alternate_enclosure in &item.podcast_alternate_enclosure {
        children.push(analyze_podcast_alternate_enclosure(alternate_enclosure));
    }

    for social_interact in &item.podcast_social_interact {
        children.push(analyze_podcast_social_interact(social_interact));
    }

    Node {
        name: TagName(None, "item".to_string()),
        children,
        errors,
        attributes: Vec::new(),
    }
}

fn analyze_podcast_live_item(item: &badpod::podcast::LiveItem) -> Node {
    let mut children = Vec::new();
    let mut attributes = Vec::new();
    let mut errors = Vec::new();

    match &item.status {
        Some(badpod::podcast::LiveItemStatus::Other((s, reason))) => {
            errors.push(Error::InvalidAttributeWithReason(
                "status".to_string(),
                s.to_string(),
                reason.to_string(),
            ))
        }
        Some(s) => attributes.push(("status".to_string(), Value::Object(s.to_string()))),
        None => errors.push(Error::MissingAttribute("status".to_string())),
    }

    match &item.start {
        Some(badpod::DateTime::Other((s, reason))) => {
            errors.push(Error::InvalidAttributeWithReason(
                "start".to_string(),
                s.to_string(),
                reason.to_string(),
            ))
        }
        Some(t) => attributes.push(("start".to_string(), Value::Object(t.to_string()))),
        None => errors.push(Error::MissingAttribute("start".to_string())),
    }

    match &item.end {
        Some(badpod::DateTime::Other((s, reason))) => errors.push(
            Error::InvalidAttributeWithReason("end".to_string(), s.to_string(), reason.to_string()),
        ),
        Some(t) => attributes.push(("end".to_string(), Value::Object(t.to_string()))),
        None => errors.push(Error::MissingAttribute("end".to_string())),
    }

    for title in &item.title {
        children.push(analyze_title(title.to_string()));
    }
    match item.title.len() {
        0 => errors.push(Error::MissingChild(TagName(None, "title".to_string()))),
        1 => {}
        _ => errors.push(Error::MultipleChildren(TagName(None, "title".to_string()))),
    }

    for content_link in &item.podcast_content_link {
        children.push(analyze_podcast_content_link(content_link));
    }
    if item.podcast_content_link.is_empty() {
        errors.push(Error::MissingChild(TagName(
            Some(Namespace::Podcast),
            "contentLink".to_string(),
        )));
    }

    for v4v_value in &item.podcast_value {
        children.push(analyze_podcast_value(v4v_value));
    }

    for person in &item.podcast_person {
        children.push(analyze_podcast_person(person));
    }

    for location in &item.podcast_location {
        children.push(analyze_podcast_location(location));
    }
    if item.podcast_location.len() > 1 {
        errors.push(Error::MultipleChildren(TagName(
            Some(Namespace::Podcast),
            "location".to_string(),
        )));
    }

    for images in &item.podcast_images {
        children.push(analyze_podcast_images(images));
    }
    if item.podcast_images.len() > 1 {
        errors.push(Error::MultipleChildren(TagName(
            Some(Namespace::Podcast),
            "images".to_string(),
        )));
    }

    for txt in &item.podcast_txt {
        children.push(analyze_podcast_txt(txt));
    }

    for transcript in &item.podcast_transcript {
        children.push(analyze_podcast_transcript(transcript));
    }

    for chapters in &item.podcast_chapters {
        children.push(analyze_podcast_chapters(chapters));
    }
    if item.podcast_chapters.len() > 1 {
        errors.push(Error::MultipleChildren(TagName(
            Some(Namespace::Podcast),
            "chapters".to_string(),
        )));
    }

    for soundbite in &item.podcast_soundbite {
        children.push(analyze_podcast_soundbite(soundbite));
    }

    for season in &item.podcast_season {
        children.push(analyze_podcast_season(season));
    }
    if item.podcast_season.len() > 1 {
        errors.push(Error::MultipleChildren(TagName(
            Some(Namespace::Podcast),
            "season".to_string(),
        )));
    }

    for episode in &item.podcast_episode {
        children.push(analyze_podcast_episode(episode));
    }
    if item.podcast_episode.len() > 1 {
        errors.push(Error::MultipleChildren(TagName(
            Some(Namespace::Podcast),
            "episode".to_string(),
        )));
    }

    for license in &item.podcast_license {
        children.push(analyze_podcast_license(license));
    }
    if item.podcast_license.len() > 1 {
        errors.push(Error::MultipleChildren(TagName(
            Some(Namespace::Podcast),
            "license".to_string(),
        )));
    }

    for alternate_enclosure in &item.podcast_alternate_enclosure {
        children.push(analyze_podcast_alternate_enclosure(alternate_enclosure));
    }

    for social_interact in &item.podcast_social_interact {
        children.push(analyze_podcast_social_interact(social_interact));
    }

    Node {
        name: TagName(Some(Namespace::Podcast), "liveItem".to_string()),
        children,
        errors,
        attributes,
    }
}

fn analyze_title(title: String) -> Node {
    Node {
        name: TagName(None, "title".to_string()),
        attributes: vec![(NODE_VALUE.to_string(), Value::Text(title.to_string()))],
        ..Default::default()
    }
}

fn analyze_podcast_value(v4v_value: &badpod::podcast::Value) -> Node {
    let mut children = Vec::new();
    let mut errors = Vec::new();
    let mut attributes = Vec::new();

    for recipient in &v4v_value.value_recipient {
        children.push(analyze_podcast_value_recipient(recipient));
    }
    if v4v_value.value_recipient.is_empty() {
        errors.push(Error::MissingChild(TagName(
            Some(Namespace::Podcast),
            "valueRecipient".to_string(),
        )));
    }

    match &v4v_value.type_ {
        Some(badpod::podcast::ValueType::Other((s, reason))) => {
            errors.push(Error::InvalidAttributeWithReason(
                "type".to_string(),
                s.to_string(),
                reason.to_string(),
            ))
        }
        Some(type_) => {
            attributes.push(("type".to_string(), Value::Object(type_.to_string())));
        }
        None => {
            errors.push(Error::MissingAttribute("type".to_string()));
        }
    }

    match &v4v_value.method {
        Some(badpod::podcast::ValueMethod::Other((s, reason))) => {
            errors.push(Error::InvalidAttributeWithReason(
                "method".to_string(),
                s.to_string(),
                reason.to_string(),
            ))
        }
        Some(method) => {
            attributes.push(("method".to_string(), Value::Object(method.to_string())));
        }
        None => {
            errors.push(Error::MissingAttribute("method".to_string()));
        }
    }

    match &v4v_value.suggested {
        Some(badpod::Float::Ok(f)) => {
            attributes.push(("suggested".to_string(), Value::Object(f.to_string())));
        }
        Some(badpod::Float::Other((s, reason))) => {
            errors.push(Error::InvalidAttributeWithReason(
                "suggested".to_string(),
                s.to_string(),
                reason.to_string(),
            ));
        }
        None => {}
    };

    Node {
        name: TagName(Some(Namespace::Podcast), "value".to_string()),
        children,
        errors,
        attributes,
    }
}

fn analyze_podcast_value_recipient(recipient: &badpod::podcast::ValueRecipient) -> Node {
    let mut errors = Vec::new();
    let mut attributes = Vec::new();

    if let Some(type_) = &recipient.type_ {
        match type_ {
            badpod::podcast::ValueRecipientType::Other((s, reason)) => {
                errors.push(Error::InvalidAttributeWithReason(
                    "type".to_string(),
                    s.to_string(),
                    reason.to_string(),
                ));
            }
            _ => {
                attributes.push(("type".to_string(), Value::Object(type_.to_string())));
            }
        }
    }

    if let Some(address) = &recipient.address {
        attributes.push(("address".to_string(), Value::Text(address.to_string())));
    } else {
        errors.push(Error::MissingAttribute("address".to_string()));
    }

    if let Some(split) = &recipient.split {
        match split {
            badpod::Integer::Ok(i) => {
                attributes.push(("split".to_string(), Value::Object(i.to_string())));
            }
            badpod::Integer::Other((s, reason)) => {
                errors.push(Error::InvalidAttributeWithReason(
                    "split".to_string(),
                    s.to_string(),
                    reason.to_string(),
                ));
            }
        }
    }

    if let Some(name) = &recipient.name {
        attributes.push(("name".to_string(), Value::Text(name.to_string())));
    }

    if let Some(custom_key) = &recipient.custom_key {
        attributes.push(("customKey".to_string(), Value::Text(custom_key.to_string())));
    }

    if let Some(custom_value) = &recipient.custom_value {
        attributes.push((
            "customValue".to_string(),
            Value::Text(custom_value.to_string()),
        ));
    }

    match (&recipient.custom_key, &recipient.custom_value) {
        (Some(_), None) => {
            errors.push(Error::MissingAttribute("customValue".to_string()));
        }
        (None, Some(_)) => {
            errors.push(Error::MissingAttribute("customKey".to_string()));
        }
        _ => {}
    }

    if let Some(fee) = &recipient.fee {
        match fee {
            badpod::Bool::Ok(b) => {
                attributes.push(("fee".to_string(), Value::Object(b.to_string())));
            }
            badpod::Bool::Other((s, reason)) => {
                errors.push(Error::InvalidAttributeWithReason(
                    "fee".to_string(),
                    s.to_string(),
                    reason.to_string(),
                ));
            }
        }
    }

    Node {
        name: TagName(Some(Namespace::Podcast), "valueRecipient".to_string()),
        children: vec![],
        errors,
        attributes,
    }
}

fn analyze_podcast_location(location: &badpod::podcast::Location) -> Node {
    let mut errors = Vec::new();
    let mut attributes = Vec::new();

    if let Some(value) = &location.value {
        if value.len() > 128 {
            errors.push(Error::AttributeExceedsMaxLength(
                NODE_VALUE.to_string(),
                value.to_string(),
                128,
            ));
        } else {
            attributes.push((NODE_VALUE.to_string(), Value::Text(value.to_string())));
        }
    } else {
        errors.push(Error::MissingAttribute(NODE_VALUE.to_string()));
    }

    if let Some(geo) = &location.geo {
        match geo {
            badpod::podcast::Geo::Ok {
                latitude,
                longitude,
                altitude,
                uncertainty,
            } => {
                let mut geo_str = format!("{{ latitude: {}, longitude: {}", latitude, longitude);
                if let Some(altitude) = altitude {
                    geo_str.push_str(format!(", altitude: {}", altitude).as_str());
                }
                if let Some(uncertainty) = uncertainty {
                    geo_str.push_str(format!(", uncertainty: {}", uncertainty).as_str());
                }
                geo_str.push_str(" }");
                attributes.push(("geo".to_string(), Value::Object(geo_str)));
            }
            badpod::podcast::Geo::Other((s, reason)) => {
                errors.push(Error::InvalidAttributeWithReason(
                    "geo".to_string(),
                    s.to_string(),
                    reason.to_string(),
                ));
            }
        }
    }

    if let Some(osm) = &location.osm {
        match osm {
            badpod::podcast::Osm::Ok {
                type_,
                id,
                revision,
            } => {
                let mut osm_str = format!("{{ type: {:?}, id: {}", type_, id);
                if let Some(revision) = revision {
                    osm_str.push_str(format!(", revision: {}", revision).as_str());
                }
                osm_str.push_str(" }");
                attributes.push(("osm".to_string(), Value::Object(osm_str)));
            }
            badpod::podcast::Osm::Other((s, reason)) => {
                errors.push(Error::InvalidAttributeWithReason(
                    "osm".to_string(),
                    s.to_string(),
                    reason.to_string(),
                ));
            }
        }
    }

    Node {
        name: TagName(Some(Namespace::Podcast), "location".to_string()),
        errors,
        attributes,
        ..Default::default()
    }
}

fn analyze_podcast_guid(guid: &badpod::podcast::Guid) -> Node {
    let mut errors = Vec::new();
    let mut attributes = Vec::new();

    match guid {
        badpod::podcast::Guid::Ok(guid) => {
            attributes.push((NODE_VALUE.to_string(), Value::Text(guid.to_string())));
        }
        badpod::podcast::Guid::Other((s, reason)) => {
            errors.push(Error::InvalidAttributeWithReason(
                NODE_VALUE.to_string(),
                s.to_string(),
                reason.to_string(),
            ));
        }
    }

    Node {
        name: TagName(Some(Namespace::Podcast), "guid".to_string()),
        errors,
        attributes,
        ..Default::default()
    }
}

fn analyze_podcast_medium(medium: &badpod::podcast::Medium) -> Node {
    let mut errors = Vec::new();
    let mut attributes = Vec::new();

    match medium {
        badpod::podcast::Medium::Other((s, reason)) => {
            errors.push(Error::InvalidAttributeWithReason(
                NODE_VALUE.to_string(),
                s.to_string(),
                reason.to_string(),
            ));
        }
        _ => {
            attributes.push((NODE_VALUE.to_string(), Value::Object(medium.to_string())));
        }
    }

    Node {
        name: TagName(Some(Namespace::Podcast), "medium".to_string()),
        errors,
        attributes,
        ..Default::default()
    }
}

fn analyze_podcast_txt(txt: &badpod::podcast::Txt) -> Node {
    let mut errors = Vec::new();
    let mut attributes = Vec::new();

    if let Some(value) = &txt.value {
        attributes.push((NODE_VALUE.to_string(), Value::Text(value.to_string())));
    } else {
        errors.push(Error::MissingAttribute(NODE_VALUE.to_string()));
    }

    match &txt.purpose {
        Some(badpod::podcast::TxtPurpose::Other((s, reason))) => {
            errors.push(Error::InvalidAttributeWithReason(
                "purpose".to_string(),
                s.to_string(),
                reason.to_string(),
            ));
        }
        Some(purpose) => {
            attributes.push(("purpose".to_string(), Value::Object(purpose.to_string())));
        }
        None => {}
    }

    Node {
        name: TagName(Some(Namespace::Podcast), "txt".to_string()),
        errors,
        attributes,
        ..Default::default()
    }
}

fn analyze_podcast_block(block: &badpod::podcast::Block) -> Node {
    let mut errors = Vec::new();
    let mut attributes = Vec::new();

    match &block.value {
        Some(badpod::Bool::Ok(b)) => {
            attributes.push((NODE_VALUE.to_string(), Value::Object(b.to_string())));
        }
        Some(badpod::Bool::Other((s, reason))) => {
            errors.push(Error::InvalidAttributeWithReason(
                NODE_VALUE.to_string(),
                s.to_string(),
                reason.to_string(),
            ));
        }
        None => {
            errors.push(Error::MissingAttribute(NODE_VALUE.to_string()));
        }
    }

    if let Some(id) = &block.id {
        match id {
            badpod::podcast::Service::Other((s, reason)) => {
                errors.push(Error::InvalidAttributeWithReason(
                    "id".to_string(),
                    s.to_string(),
                    reason.to_string(),
                ));
            }
            _ => {
                attributes.push(("id".to_string(), Value::Object(id.to_string())));
            }
        }
    }

    Node {
        name: TagName(Some(Namespace::Podcast), "block".to_string()),
        errors,
        attributes,
        ..Default::default()
    }
}

fn analyze_podcast_locked(locked: &badpod::podcast::Locked) -> Node {
    let mut errors = Vec::new();
    let mut attributes = Vec::new();

    match &locked.value {
        Some(badpod::Bool::Ok(b)) => {
            attributes.push((NODE_VALUE.to_string(), Value::Object(b.to_string())));
        }
        Some(badpod::Bool::Other((s, reason))) => {
            errors.push(Error::InvalidAttributeWithReason(
                NODE_VALUE.to_string(),
                s.to_string(),
                reason.to_string(),
            ));
        }
        None => {
            errors.push(Error::MissingAttribute(NODE_VALUE.to_string()));
        }
    }

    if let Some(owner) = &locked.owner {
        attributes.push(("owner".to_string(), Value::Text(owner.to_string())));
    }

    Node {
        name: TagName(Some(Namespace::Podcast), "locked".to_string()),
        errors,
        attributes,
        ..Default::default()
    }
}

fn analyze_podcast_funding(funding: &badpod::podcast::Funding) -> Node {
    let mut errors = Vec::new();
    let mut attributes = Vec::new();

    if let Some(value) = &funding.value {
        if value.len() > 128 {
            errors.push(Error::AttributeExceedsMaxLength(
                NODE_VALUE.to_string(),
                value.to_string(),
                128,
            ));
        } else {
            attributes.push((NODE_VALUE.to_string(), Value::Text(value.to_string())));
        }
    } else {
        errors.push(Error::MissingAttribute(NODE_VALUE.to_string()));
    }

    if let Some(url) = &funding.url {
        match url {
            badpod::Url::Ok(url) => {
                attributes.push(("url".to_string(), Value::Url(url.to_string())));
            }
            badpod::Url::Other((s, reason)) => {
                errors.push(Error::InvalidAttributeWithReason(
                    "url".to_string(),
                    s.to_string(),
                    reason.to_string(),
                ));
            }
        }
    } else {
        errors.push(Error::MissingAttribute("url".to_string()));
    }

    Node {
        name: TagName(Some(Namespace::Podcast), "funding".to_string()),
        errors,
        attributes,
        ..Default::default()
    }
}

fn analyze_podcast_person(person: &badpod::podcast::Person) -> Node {
    let mut errors = Vec::new();
    let mut attributes = Vec::new();

    if let Some(name) = &person.value {
        if name.len() > 128 {
            errors.push(Error::AttributeExceedsMaxLength(
                NODE_VALUE.to_string(),
                name.to_string(),
                128,
            ));
        } else {
            attributes.push((NODE_VALUE.to_string(), Value::Text(name.to_string())));
        }
    } else {
        errors.push(Error::MissingAttribute(NODE_VALUE.to_string()));
    }

    if let Some(group) = &person.group {
        match group {
            badpod::podcast::PersonGroup::Other((s, reason)) => {
                errors.push(Error::InvalidAttributeWithReason(
                    "group".to_string(),
                    s.to_string(),
                    reason.to_string(),
                ));
            }
            _ => {
                attributes.push(("group".to_string(), Value::Object(group.to_string())));
            }
        }
    }

    if let Some(role) = &person.role {
        match role {
            badpod::podcast::PersonRole::Other((s, reason)) => {
                errors.push(Error::InvalidAttributeWithReason(
                    "role".to_string(),
                    s.to_string(),
                    reason.to_string(),
                ));
            }
            _ => {
                attributes.push(("role".to_string(), Value::Object(role.to_string())));
            }
        }
    }

    if let Some(image) = &person.img {
        match image {
            badpod::Url::Ok(url) => {
                attributes.push(("img".to_string(), Value::Url(url.to_string())));
            }
            badpod::Url::Other((s, reason)) => {
                errors.push(Error::InvalidAttributeWithReason(
                    "img".to_string(),
                    s.to_string(),
                    reason.to_string(),
                ));
            }
        }
    }

    if let Some(href) = &person.href {
        match href {
            badpod::Url::Ok(url) => {
                attributes.push(("href".to_string(), Value::Url(url.to_string())));
            }
            badpod::Url::Other((s, reason)) => {
                errors.push(Error::InvalidAttributeWithReason(
                    "href".to_string(),
                    s.to_string(),
                    reason.to_string(),
                ));
            }
        }
    }

    Node {
        name: TagName(Some(Namespace::Podcast), "person".to_string()),
        errors,
        attributes,
        ..Default::default()
    }
}

fn analyze_podcast_trailer(trailer: &badpod::podcast::Trailer) -> Node {
    let mut errors = Vec::new();
    let mut attributes = Vec::new();

    if let Some(tile) = &trailer.value {
        if tile.len() > 128 {
            errors.push(Error::AttributeExceedsMaxLength(
                NODE_VALUE.to_string(),
                tile.to_string(),
                128,
            ));
        } else {
            attributes.push((NODE_VALUE.to_string(), Value::Text(tile.to_string())));
        }
    } else {
        errors.push(Error::MissingAttribute(NODE_VALUE.to_string()));
    }

    if let Some(url) = &trailer.url {
        match url {
            badpod::Url::Ok(url) => {
                attributes.push(("url".to_string(), Value::Url(url.to_string())));
            }
            badpod::Url::Other((s, reason)) => {
                errors.push(Error::InvalidAttributeWithReason(
                    "url".to_string(),
                    s.to_string(),
                    reason.to_string(),
                ));
            }
        }
    } else {
        errors.push(Error::MissingAttribute("url".to_string()));
    }

    if let Some(pub_date) = &trailer.pub_date {
        match pub_date {
            badpod::DateTime::Ok(dt) => {
                attributes.push(("pubDate".to_string(), Value::Object(dt.to_string())));
            }
            badpod::DateTime::Other((s, reason)) => {
                errors.push(Error::InvalidAttributeWithReason(
                    "pubDate".to_string(),
                    s.to_string(),
                    reason.to_string(),
                ));
            }
        }
    }

    if let Some(length) = &trailer.length {
        match length {
            badpod::Integer::Ok(i) => {
                attributes.push(("length".to_string(), Value::Object(i.to_string())));
            }
            badpod::Integer::Other((s, reason)) => {
                errors.push(Error::InvalidAttributeWithReason(
                    "length".to_string(),
                    s.to_string(),
                    reason.to_string(),
                ));
            }
        }
    }

    if let Some(mimetype) = &trailer.type_ {
        match mimetype {
            badpod::MimeEnclosure::Other((s, _)) => {
                errors.push(Error::InvalidAttribute("type".to_string(), s.to_string()));
            }
            _ => {
                attributes.push(("type".to_string(), Value::Text(mimetype.to_string())));
            }
        }
    }

    if let Some(season) = &trailer.season {
        match season {
            badpod::Integer::Ok(i) => {
                attributes.push(("season".to_string(), Value::Object(i.to_string())));
            }
            badpod::Integer::Other((s, reason)) => {
                errors.push(Error::InvalidAttributeWithReason(
                    "season".to_string(),
                    s.to_string(),
                    reason.to_string(),
                ));
            }
        }
    }

    Node {
        name: TagName(Some(Namespace::Podcast), "trailer".to_string()),
        errors,
        attributes,
        ..Default::default()
    }
}

fn analyze_podcast_license(license: &badpod::podcast::License) -> Node {
    let mut errors = Vec::new();
    let mut attributes = Vec::new();

    if let Some(url) = &license.url {
        match url {
            badpod::Url::Ok(url) => {
                attributes.push(("url".to_string(), Value::Url(url.to_string())));
            }
            badpod::Url::Other((s, reason)) => {
                errors.push(Error::InvalidAttributeWithReason(
                    "url".to_string(),
                    s.to_string(),
                    reason.to_string(),
                ));
            }
        }
    }

    match &license.value {
        Some(value) => {
            match value {
                badpod::podcast::LicenseType::Other((s, _)) => {
                    if s.len() > 128 {
                        errors.push(Error::AttributeExceedsMaxLength(
                            NODE_VALUE.to_string(),
                            s.to_string(),
                            128,
                        ));
                    } else {
                        attributes.push((NODE_VALUE.to_string(), Value::Text(s.to_string())));
                    }
                    if license.url.is_none() {
                        errors.push(Error::MissingAttribute("url".to_string()));
                    }
                }
                _ => {
                    attributes.push((NODE_VALUE.to_string(), Value::Object(value.to_string())));
                }
            };
        }
        None => {
            errors.push(Error::MissingAttribute(NODE_VALUE.to_string()));
        }
    }

    Node {
        name: TagName(Some(Namespace::Podcast), "license".to_string()),
        errors,
        attributes,
        ..Default::default()
    }
}

fn analyze_podcast_images(images: &badpod::podcast::Images) -> Node {
    let mut attributes = Vec::new();
    let mut errors = Vec::new();

    match &images.srcset {
        badpod::podcast::ImageSrcSet::Ok(image_data) => {
            let mut img_strs = Vec::new();
            for (url, width) in image_data {
                img_strs.push(format!("{{ url: \"{}\", width: {} }}", url, width));
            }
            let value = format!("[ {} ]", img_strs.join(", "));
            attributes.push(("srcset".to_string(), Value::Object(value)));
        }
        badpod::podcast::ImageSrcSet::Other((s, reason)) => {
            errors.push(Error::InvalidAttributeWithReason(
                "srcset".to_string(),
                s.to_string(),
                reason.to_string(),
            ));
        }
    }

    Node {
        name: TagName(Some(Namespace::Podcast), "images".to_string()),
        attributes,
        errors,
        ..Default::default()
    }
}

fn analyze_podcast_transcript(transcript: &badpod::podcast::Transcript) -> Node {
    let mut errors = Vec::new();
    let mut attributes = Vec::new();

    if let Some(url) = &transcript.url {
        match url {
            badpod::Url::Ok(url) => {
                attributes.push(("url".to_string(), Value::Url(url.to_string())));
            }
            badpod::Url::Other((s, reason)) => {
                errors.push(Error::InvalidAttributeWithReason(
                    "url".to_string(),
                    s.to_string(),
                    reason.to_string(),
                ));
            }
        }
    } else {
        errors.push(Error::MissingAttribute("url".to_string()));
    }

    if let Some(type_) = &transcript.type_ {
        match type_ {
            badpod::MimeTranscript::ApplicationSrt => {
                errors.push(Error::CustomWithExtraInfo(
                    "\"<code>application/srt</code>\" in attribute <code class=\"font-bold\">type</code> is not a valid mime type.".to_string(),
                    "<a class=\"link\" href=\"https://github.com/Podcastindex-org/podcast-namespace/pull/331\" target=\"_blank\" rel=\"noopener noreferrer\">On February 3, 2022</a>, the recognized alternative for SubRip files in the podcast namespace specification became \"<code>application/x-subrip</code>\". However, keep in mind that although podcast players like Podverse and Podcast Addict have updated their codebases, some other apps may still only recognize \"<code>application/srt</code>\" at this time."
                        .to_string(),
                ));
            }
            badpod::MimeTranscript::Other((s, _)) => {
                errors.push(Error::InvalidAttribute("type".to_string(), s.to_string()));
            }
            _ => {
                attributes.push(("type".to_string(), Value::Object(type_.to_string())));
            }
        }
    } else {
        errors.push(Error::MissingAttribute("type".to_string()));
    }

    if let Some(language) = &transcript.language {
        match language {
            badpod::Language::Other((s, reason)) => {
                errors.push(Error::InvalidAttributeWithReason(
                    "language".to_string(),
                    s.to_string(),
                    reason.to_string(),
                ));
            }
            _ => {
                attributes.push(("language".to_string(), Value::Object(language.to_string())));
            }
        }
    }

    if let Some(rel) = &transcript.rel {
        match rel {
            badpod::podcast::TranscriptRel::Other((s, reason)) => {
                errors.push(Error::InvalidAttributeWithReason(
                    "rel".to_string(),
                    s.to_string(),
                    reason.to_string(),
                ));
            }
            _ => {
                attributes.push(("rel".to_string(), Value::Object(rel.to_string())));
            }
        }
    }

    Node {
        name: TagName(Some(Namespace::Podcast), "transcript".to_string()),
        errors,
        attributes,
        ..Default::default()
    }
}

fn analyze_podcast_chapters(chapters: &badpod::podcast::Chapters) -> Node {
    let mut errors = Vec::new();
    let mut attributes = Vec::new();

    if let Some(url) = &chapters.url {
        match url {
            badpod::Url::Ok(url) => {
                attributes.push(("url".to_string(), Value::Url(url.to_string())));
            }
            badpod::Url::Other((s, reason)) => {
                errors.push(Error::InvalidAttributeWithReason(
                    "url".to_string(),
                    s.to_string(),
                    reason.to_string(),
                ));
            }
        }
    } else {
        errors.push(Error::MissingAttribute("url".to_string()));
    }

    if let Some(type_) = &chapters.type_ {
        match type_ {
            badpod::MimeChapters::Other((s, _)) => {
                errors.push(Error::InvalidAttribute("type".to_string(), s.to_string()));
            }
            _ => {
                attributes.push(("type".to_string(), Value::Object(type_.to_string())));
            }
        }
    } else {
        errors.push(Error::MissingAttribute("type".to_string()));
    }

    Node {
        name: TagName(Some(Namespace::Podcast), "chapters".to_string()),
        errors,
        attributes,
        ..Default::default()
    }
}

fn analyze_podcast_soundbite(soundbite: &badpod::podcast::Soundbite) -> Node {
    let mut errors = Vec::new();
    let mut attributes = Vec::new();

    match &soundbite.start_time {
        Some(badpod::Float::Ok(f)) => {
            attributes.push(("startTime".to_string(), Value::Object(f.to_string())));
        }
        Some(badpod::Float::Other((s, reason))) => {
            errors.push(Error::InvalidAttributeWithReason(
                "startTime".to_string(),
                s.to_string(),
                reason.to_string(),
            ));
        }
        None => {
            errors.push(Error::MissingAttribute("startTime".to_string()));
        }
    }

    match &soundbite.duration {
        Some(badpod::Float::Ok(f)) => {
            attributes.push(("duration".to_string(), Value::Object(f.to_string())));
        }
        Some(badpod::Float::Other((s, reason))) => {
            errors.push(Error::InvalidAttributeWithReason(
                "duration".to_string(),
                s.to_string(),
                reason.to_string(),
            ));
        }
        None => {
            errors.push(Error::MissingAttribute("duration".to_string()));
        }
    }

    Node {
        name: TagName(Some(Namespace::Podcast), "soundbite".to_string()),
        errors,
        attributes,
        ..Default::default()
    }
}

fn analyze_podcast_season(season: &badpod::podcast::Season) -> Node {
    let mut errors = Vec::new();
    let mut attributes = Vec::new();

    match &season.value {
        Some(badpod::Integer::Ok(i)) => {
            attributes.push((NODE_VALUE.to_string(), Value::Object(i.to_string())));
        }
        Some(badpod::Integer::Other((s, reason))) => {
            errors.push(Error::InvalidAttributeWithReason(
                NODE_VALUE.to_string(),
                s.to_string(),
                reason.to_string(),
            ));
        }
        None => {
            errors.push(Error::MissingAttribute(NODE_VALUE.to_string()));
        }
    }

    if let Some(name) = &season.name {
        attributes.push(("name".to_string(), Value::Text(name.to_string())));
    }

    Node {
        name: TagName(Some(Namespace::Podcast), "season".to_string()),
        errors,
        attributes,
        ..Default::default()
    }
}

fn analyze_podcast_episode(episode: &badpod::podcast::Episode) -> Node {
    let mut errors = Vec::new();
    let mut attributes = Vec::new();

    match &episode.value {
        Some(badpod::Number::Other((s, reason))) => {
            errors.push(Error::InvalidAttributeWithReason(
                NODE_VALUE.to_string(),
                s.to_string(),
                reason.to_string(),
            ));
        }
        Some(n) => {
            attributes.push((NODE_VALUE.to_string(), Value::Object(n.to_string())));
        }
        None => {
            errors.push(Error::MissingAttribute(NODE_VALUE.to_string()));
        }
    }

    if let Some(display) = &episode.display {
        attributes.push(("display".to_string(), Value::Text(display.to_string())));
    }

    Node {
        name: TagName(Some(Namespace::Podcast), "episode".to_string()),
        errors,
        attributes,
        ..Default::default()
    }
}

fn analyze_podcast_alternate_enclosure(
    alternate_enclosure: &badpod::podcast::AlternateEnclosure,
) -> Node {
    let mut errors = Vec::new();
    let mut attributes = Vec::new();
    let mut children = Vec::new();

    if let Some(type_) = &alternate_enclosure.type_ {
        match type_ {
            badpod::MimeEnclosure::Other((s, _)) => {
                errors.push(Error::InvalidAttribute("type".to_string(), s.to_string()));
            }
            _ => {
                attributes.push(("type".to_string(), Value::Object(type_.to_string())));
            }
        }
    } else {
        errors.push(Error::MissingAttribute("type".to_string()));
    }

    if let Some(length) = &alternate_enclosure.length {
        match length {
            badpod::Integer::Other((s, reason)) => {
                errors.push(Error::InvalidAttributeWithReason(
                    "length".to_string(),
                    s.to_string(),
                    reason.to_string(),
                ));
            }
            _ => {
                attributes.push(("length".to_string(), Value::Object(length.to_string())));
            }
        }
    }

    if let Some(bit_rate) = &alternate_enclosure.bit_rate {
        match bit_rate {
            badpod::Float::Other((s, reason)) => {
                errors.push(Error::InvalidAttributeWithReason(
                    "bitrate".to_string(),
                    s.to_string(),
                    reason.to_string(),
                ));
            }
            _ => {
                attributes.push(("bitrate".to_string(), Value::Object(bit_rate.to_string())));
            }
        }
    }

    if let Some(height) = &alternate_enclosure.height {
        match height {
            badpod::Integer::Other((s, reason)) => {
                errors.push(Error::InvalidAttributeWithReason(
                    "height".to_string(),
                    s.to_string(),
                    reason.to_string(),
                ));
            }
            _ => {
                attributes.push(("height".to_string(), Value::Object(height.to_string())));
            }
        }
    }

    if let Some(lang) = &alternate_enclosure.language {
        attributes.push(("lang".to_string(), Value::Text(lang.to_string())));
    }

    if let Some(title) = &alternate_enclosure.title {
        attributes.push(("title".to_string(), Value::Text(title.to_string())));
    }

    if let Some(rel) = &alternate_enclosure.rel {
        if rel.len() > 32 {
            errors.push(Error::AttributeExceedsMaxLength(
                "rel".to_string(),
                rel.to_string(),
                32,
            ));
        } else {
            attributes.push(("rel".to_string(), Value::Text(rel.to_string())));
        }
    }

    if let Some(default) = &alternate_enclosure.default {
        match default {
            badpod::Bool::Other((s, reason)) => {
                errors.push(Error::InvalidAttributeWithReason(
                    "default".to_string(),
                    s.to_string(),
                    reason.to_string(),
                ));
            }
            _ => {
                attributes.push(("default".to_string(), Value::Object(default.to_string())));
            }
        }
    }

    for source in &alternate_enclosure.podcast_source {
        children.push(analyze_podcast_source(source));
    }
    if children.is_empty() {
        errors.push(Error::MissingChild(TagName(
            Some(Namespace::Podcast),
            "source".to_string(),
        )));
    }

    for integrity in &alternate_enclosure.podcast_integrity {
        children.push(analyze_podcast_integrity(integrity));
    }
    if alternate_enclosure.podcast_integrity.len() > 1 {
        errors.push(Error::MultipleChildren(TagName(
            Some(Namespace::Podcast),
            "integrity".to_string(),
        )));
    }

    Node {
        name: TagName(Some(Namespace::Podcast), "alternateEnclosure".to_string()),
        errors,
        attributes,
        children,
    }
}

fn analyze_podcast_source(source: &badpod::podcast::Source) -> Node {
    let mut errors = Vec::new();
    let mut attributes = Vec::new();

    if let Some(uri) = &source.uri {
        match uri {
            badpod::Url::Ok(url) => {
                attributes.push(("uri".to_string(), Value::Url(url.to_string())));
            }
            badpod::Url::Other((s, reason)) => {
                errors.push(Error::InvalidAttributeWithReason(
                    "uri".to_string(),
                    s.to_string(),
                    reason.to_string(),
                ));
            }
        }
    } else {
        errors.push(Error::MissingAttribute("uri".to_string()));
    }

    if let Some(content_type) = &source.type_ {
        match content_type {
            badpod::MimeEnclosure::Other((s, _)) => {
                errors.push(Error::InvalidAttribute(
                    "contentType".to_string(),
                    s.to_string(),
                ));
            }
            _ => {
                attributes.push((
                    "contentType".to_string(),
                    Value::Object(content_type.to_string()),
                ));
            }
        }
    }

    Node {
        name: TagName(Some(Namespace::Podcast), "source".to_string()),
        errors,
        attributes,
        ..Default::default()
    }
}

fn analyze_podcast_integrity(integrity: &badpod::podcast::Integrity) -> Node {
    let mut errors = Vec::new();
    let mut attributes = Vec::new();

    match &integrity.type_ {
        Some(badpod::podcast::IntegrityType::Other((s, reason))) => {
            errors.push(Error::InvalidAttributeWithReason(
                "type".to_string(),
                s.to_string(),
                reason.to_string(),
            ));
        }
        Some(t) => {
            attributes.push(("type".to_string(), Value::Object(t.to_string())));
        }
        None => {
            errors.push(Error::MissingAttribute("type".to_string()));
        }
    }

    if let Some(value) = &integrity.value {
        attributes.push(("value".to_string(), Value::Text(value.to_string())));
    } else {
        errors.push(Error::MissingAttribute("value".to_string()));
    }

    Node {
        name: TagName(Some(Namespace::Podcast), "integrity".to_string()),
        errors,
        attributes,
        ..Default::default()
    }
}

fn analyze_podcast_social_interact(social_interact: &badpod::podcast::SocialInteract) -> Node {
    let mut errors = Vec::new();
    let mut attributes = Vec::new();

    if let Some(uri) = &social_interact.uri {
        match uri {
            badpod::Url::Ok(url) => {
                attributes.push(("uri".to_string(), Value::Url(url.to_string())));
            }
            badpod::Url::Other((s, reason)) => {
                errors.push(Error::InvalidAttributeWithReason(
                    "uri".to_string(),
                    s.to_string(),
                    reason.to_string(),
                ));
            }
        }
    } else {
        errors.push(Error::MissingAttribute("uri".to_string()));
    }

    match &social_interact.protocol {
        Some(badpod::podcast::SocialProtocol::Other((s, reason))) => {
            errors.push(Error::InvalidAttributeWithReason(
                "protocol".to_string(),
                s.to_string(),
                reason.to_string(),
            ));
        }
        Some(p) => {
            attributes.push(("protocol".to_string(), Value::Object(p.to_string())));
        }
        None => {
            errors.push(Error::MissingAttribute("protocol".to_string()));
        }
    }

    if let Some(account_id) = &social_interact.account_id {
        attributes.push(("accountId".to_string(), Value::Text(account_id.to_string())));
    }

    if let Some(account_url) = &social_interact.account_url {
        match account_url {
            badpod::Url::Ok(url) => {
                attributes.push(("accountUrl".to_string(), Value::Url(url.to_string())));
            }
            badpod::Url::Other((s, reason)) => {
                errors.push(Error::InvalidAttributeWithReason(
                    "accountUrl".to_string(),
                    s.to_string(),
                    reason.to_string(),
                ));
            }
        }
    }

    if let Some(priority) = &social_interact.priority {
        match priority {
            badpod::Integer::Other((s, reason)) => {
                errors.push(Error::InvalidAttributeWithReason(
                    "priority".to_string(),
                    s.to_string(),
                    reason.to_string(),
                ));
            }
            _ => {
                attributes.push(("priority".to_string(), Value::Object(priority.to_string())));
            }
        }
    }

    Node {
        name: TagName(Some(Namespace::Podcast), "socialInteract".to_string()),
        errors,
        attributes,
        ..Default::default()
    }
}

#[component(inline_props)]
fn DisplayProgramError<G: Html>(cx: Scope, program_error: ProgramError<G>) -> View<G> {
    let mut input = view! { cx, (program_error.description) };

    if let Some((error_name, error)) = program_error.error {
        input = view! { cx,
            (input)
            details(class="mt-2") {
                summary(class="font-bold") { (error_name) }
                div(class="text-xs font-mono") { (error) }
            }
        }
    }

    view! {cx,
        utils::AlertHTML(type_ = utils::AlertType::Danger, msg = input)
    }
}

fn analyze_podcast_content_link(content_link: &badpod::podcast::ContentLink) -> Node {
    let mut errors = Vec::new();
    let mut attributes = Vec::new();

    if let Some(value) = &content_link.value {
        attributes.push((NODE_VALUE.to_string(), Value::Text(value.to_string())));
    } else {
        errors.push(Error::MissingAttribute(NODE_VALUE.to_string()));
    }

    if let Some(href) = &content_link.href {
        match href {
            badpod::Url::Ok(url) => {
                attributes.push(("href".to_string(), Value::Url(url.to_string())));
            }
            badpod::Url::Other((s, reason)) => {
                errors.push(Error::InvalidAttributeWithReason(
                    "href".to_string(),
                    s.to_string(),
                    reason.to_string(),
                ));
            }
        }
    } else {
        errors.push(Error::MissingAttribute("href".to_string()));
    }

    Node {
        name: TagName(Some(Namespace::Podcast), "contentLink".to_string()),
        errors,
        attributes,
        ..Default::default()
    }
}
