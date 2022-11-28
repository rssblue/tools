use crate::components::utils;
use sycamore::prelude::*;
use sycamore::suspense::{use_transition, Suspense};
use url::Url;

#[component]
pub fn Validator<G: Html>(cx: Scope) -> View<G> {
    let input_cls = create_signal(cx, String::new());
    let fetching_data = create_signal(cx, false);
    let url_str = create_signal(cx, String::new());
    let transition = use_transition(cx);
    let show_results = create_signal(cx, false);

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

    view! { cx,
    crate::components::ToolsBreadcrumbs(title="Validator")

        h1(class="mb-3") { "Podcast Validator" }
    h2(class="mt-3 text-gray-500") { "Make sure your Podcasting 2.0 feed is valid." }
    p(class="mt-7") {
        a(class="link", href="https://podcastindex.org/namespace/1.0", target="_blank", rel="noopener noreferrer") { "Podcast namespace initiative" }
        " is a community effort to create modern podcasting standards. You can use this tool to check that the elements of your feed are valid."
    }

    p(class="mb-7") {
        "This validator only checks the podcast namespace elements and only analyzes the text content of the feed. For other namespaces and media checks you may try "
            a(class="link", href="https://www.castfeedvalidator.com/", target="_blank", rel="noopener noreferrer") { "Cast Feed Validator" }
        " by Blubrry or "
            a(class="link", href="https://podba.se/validate/", target="_blank", rel="noopener noreferrer") { "podba.se validator" }
        " by Podbase."

    }

    form(class="mb-4") {
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
                        class=format!("input-text-base rounded-t-lg md:rounded-l-lg md:rounded-r-none text-ellipsis z-10 {}", input_cls.get()),
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
                    class=format!("btn-base btn-primary rounded-b-lg md:rounded-r-lg md:rounded-l-none col-span-4 md:col-span-1"),
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
    }

    (if *show_results.get() {
        view!{cx,
        Suspense(fallback=view! { cx, }) {
            Validate(url=url_str.get().to_string())
        }
        }
    } else {
        view! { cx,
        }
    })


    }
}

#[component(inline_props)]
pub async fn Validate<'a, G: Html>(cx: Scope<'a>, url: String) -> View<G> {
    // Use CORS proxy to avoid CORS issues.
    let client = reqwest_wasm::Client::new();
    let url = match Url::parse(&url) {
        Ok(url) => url,
        Err(e) => {
            return view! { cx,
            utils::Error(error=format!("Could not parse the URL ({e})"))
            }
        }
    };
    let url = format!("https://cors-anywhere.herokuapp.com/{}", url.as_str());
    let resp = client.get(&url).send().await;

    let resp = match resp {
        Ok(x) => x,
        Err(e) => {
            return view! {cx,
            utils::Error(error=format!("Could not fetch the feed ({e})"))
            }
        }
    };

    if !resp.status().is_success() {
        return view! {cx,
        utils::Error(error=format!("Could not fetch the feed ({})", resp.status()))
        };
    }

    let text = match resp.text().await {
        Ok(x) => x,
        Err(e) => {
            return view! {cx,
            utils::Error(error=format!("Could not fetch the feed ({e})"))
            }
        }
    };

    let feed = match badpod::from_str(&text) {
        Ok(x) => x,
        Err(e) => {
            return view! {cx,
            utils::Error(error=format!("Could not parse the feed ({e})"))
            }
        }
    };

    let root_node = analyze_rss(&feed);

    view! { cx,
    DisplayNode(node=root_node, root=true)
    }
}

#[derive(PartialEq, Debug, Clone)]
enum Namespace {
    Itunes,
    Podcast,
}

#[derive(PartialEq, Debug, Clone, Default)]
struct TagName(Option<Namespace>, String);

#[derive(PartialEq, Clone, Default)]
struct Node {
    name: TagName,
    children: Vec<Node>,
    attributes: Vec<(String, String)>,
    errors: Vec<Error>,
}

#[derive(PartialEq, Clone)]
enum Error {
    MissingAttribute(String),
    InvalidAttribute(String, String),
    MissingChild(TagName),
    MultipleChildren(TagName),
    AttributeExceedsMaxLength(String, String, usize),
    Custom(String),
}

#[component(inline_props)]
pub fn DisplayError<'a, G: Html>(cx: Scope<'a>, error: Error) -> View<G> {
    match error {
        Error::MissingAttribute(attr) => {
            if attr == "value" {
                view! { cx,
                div(class="text-red-500") {
                    "Missing value"
                }
                }
            } else {
                let attr = if attr == "value_attr" {
                    "value".to_string()
                } else {
                    attr.to_string()
                };
                view! { cx,
                div(class="text-red-500") {
                    "Missing attribute "
                        code(class="attr") { (attr) }
                }
                }
            }
        }
        Error::InvalidAttribute(attr, value) => {
            if attr == "value" {
                view! { cx,
                div(class="text-red-500") {
                    "Invalid value "
                        code { "\"" (value) "\"" }
                }
                }
            } else {
                let attr = if attr == "value_attr" {
                    "value".to_string()
                } else {
                    attr.to_string()
                };
                view! { cx,
                div(class="text-red-500") {
                    "Attribute "
                        code(class="attr") { (attr) }
                    " has invalid value "
                        code { "\"" (value) "\"" }
                }
                }
            }
        }
        Error::MissingChild(tag_name) => {
            view! { cx,
            div(class="text-red-500") {
                "Missing child "
                    code { "<" (tag_name) ">" }
            }
            }
        }
        Error::MultipleChildren(tag_name) => {
            view! { cx,
            div(class="text-red-500") {
                "Only one child "
                    code { "<" (tag_name) ">" }
                " is allowed"
            }
            }
        }
        Error::AttributeExceedsMaxLength(attr, value, max_len) => {
            if attr == "value" {
                view! { cx,
                div(class="text-red-500") {
                    "Value "
                        code { "\"" (value) "\"" }
                    " exceeds maximum length of "
                        code { (max_len) }
                    " characters"
                }
                }
            } else {
                let attr = if attr == "value_attr" {
                    "value".to_string()
                } else {
                    attr.to_string()
                };
                view! { cx,
                div(class="text-red-500") {
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
            div(class="text-red-500", dangerously_set_inner_html=msg.as_str()) {}
            }
        }
    }
}

impl std::fmt::Display for TagName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TagName(Some(Namespace::Itunes), x) => write!(f, "itunes:{}", x),
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
}

#[component(inline_props)]
fn DisplayNode<'a, G: Html>(cx: Scope<'a>, node: Node, root: bool) -> View<G> {
    let children = create_signal(cx, node.children.clone());
    let errors = create_signal(cx, node.errors.clone());
    let attributes = create_signal(cx, node.attributes.clone());
    let have_nested_errors = node.descendants_have_errors();
    let name_cls = if have_nested_errors {
        "text-danger-500"
    } else {
        ""
    };

    view! { cx,
    (if !have_nested_errors && root {
        view! { cx,
        div(class="mb-5") {
            utils::Success(success="Our analysis has not found any errors in the podcast namespace tags.".to_string())
        }
        }
    } else {
        view! { cx,
        }
    })
    details(open=have_nested_errors) {
        summary(class=name_cls) {
            code(class="font-bold") { "<"(node.name)">" }
        }
        div(class="pl-1") {
            div(class="pl-2 md:pl-4 border-l-2 border-gray-200") {
                ul(class="text-sm my-0") {
                    Indexed(
                        iterable=errors,
                        view=|cx, x| view! { cx,
                        li(class="my-0 marker:text-danger-500 text-danger-500") { DisplayError(error=x) }
                        },
                        )
                        Indexed(
                            iterable=attributes,
                            view=|cx, x| view! { cx,
                            li(class="my-0") {
                                code { span(class="attr") { (x.0) }  "=" (x.1) }
                            }
                            },
                            )
                }

                Indexed(
                    iterable=children,
                    view=|cx, x| view! { cx,
                    DisplayNode(node=x, root=false)
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
    if channel.podcast_value.len() > 1 {
        errors.push(Error::MultipleChildren(TagName(
            Some(Namespace::Podcast),
            "value".to_string(),
        )));
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
    if item.podcast_value.len() > 1 {
        errors.push(Error::MultipleChildren(TagName(
            Some(Namespace::Podcast),
            "value".to_string(),
        )));
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
        Some(badpod::podcast::LiveItemStatus::Other(s)) => {
            errors.push(Error::InvalidAttribute("status".to_string(), s.to_string()))
        }
        Some(s) => attributes.push(("status".to_string(), s.to_string())),
        None => errors.push(Error::MissingAttribute("status".to_string())),
    }

    match &item.start {
        Some(badpod::DateTime::Other(s)) => {
            errors.push(Error::InvalidAttribute("start".to_string(), s.to_string()))
        }
        Some(t) => attributes.push(("start".to_string(), t.to_string())),
        None => errors.push(Error::MissingAttribute("start".to_string())),
    }

    match &item.end {
        Some(badpod::DateTime::Other(s)) => {
            errors.push(Error::InvalidAttribute("end".to_string(), s.to_string()))
        }
        Some(t) => attributes.push(("end".to_string(), t.to_string())),
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

    for content_link in &item.content_link {
        children.push(analyze_podcast_content_link(content_link));
    }
    if item.content_link.is_empty() {
        errors.push(Error::MissingChild(TagName(
            Some(Namespace::Podcast),
            "contentLink".to_string(),
        )));
    }

    for v4v_value in &item.podcast_value {
        children.push(analyze_podcast_value(v4v_value));
    }
    if item.podcast_value.len() > 1 {
        errors.push(Error::MultipleChildren(TagName(
            Some(Namespace::Podcast),
            "value".to_string(),
        )));
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
        attributes: vec![("value".to_string(), format!("\"{}\"", title))],
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
        Some(badpod::podcast::ValueType::Other(s)) => {
            errors.push(Error::InvalidAttribute("type".to_string(), s.to_string()));
        }
        Some(type_) => {
            attributes.push(("type".to_string(), type_.to_string()));
        }
        None => {
            errors.push(Error::MissingAttribute("type".to_string()));
        }
    }

    match &v4v_value.method {
        Some(badpod::podcast::ValueMethod::Other(s)) => {
            errors.push(Error::InvalidAttribute("method".to_string(), s.to_string()));
        }
        Some(method) => {
            attributes.push(("method".to_string(), method.to_string()));
        }
        None => {
            errors.push(Error::MissingAttribute("method".to_string()));
        }
    }

    match &v4v_value.suggested {
        Some(badpod::Float::Ok(f)) => {
            attributes.push(("suggested".to_string(), f.to_string()));
        }
        Some(badpod::Float::Other(s)) => {
            errors.push(Error::InvalidAttribute(
                "suggested".to_string(),
                s.to_string(),
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
    let mut children = Vec::new();
    let mut errors = Vec::new();
    let mut attributes = Vec::new();

    if let Some(type_) = &recipient.type_ {
        match type_ {
            badpod::podcast::ValueRecipientType::Other(s) => {
                errors.push(Error::InvalidAttribute("type".to_string(), s.to_string()));
            }
            _ => {
                attributes.push(("type".to_string(), type_.to_string()));
            }
        }
    }

    if let Some(address) = &recipient.address {
        attributes.push(("address".to_string(), format!("\"{}\"", address)));
    } else {
        errors.push(Error::MissingAttribute("address".to_string()));
    }

    if let Some(split) = &recipient.split {
        match split {
            badpod::Integer::Ok(i) => {
                attributes.push(("split".to_string(), i.to_string()));
            }
            badpod::Integer::Other(s) => {
                errors.push(Error::InvalidAttribute("split".to_string(), s.to_string()));
            }
        }
    }

    if let Some(name) = &recipient.name {
        attributes.push(("name".to_string(), format!("\"{}\"", name)));
    }

    if let Some(custom_key) = &recipient.custom_key {
        attributes.push(("customKey".to_string(), format!("\"{}\"", custom_key)));
    }

    if let Some(custom_value) = &recipient.custom_value {
        attributes.push(("customValue".to_string(), format!("\"{}\"", custom_value)));
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
                attributes.push(("fee".to_string(), b.to_string()));
            }
            badpod::Bool::Other(s) => {
                errors.push(Error::InvalidAttribute("fee".to_string(), s.to_string()));
            }
        }
    }

    Node {
        name: TagName(Some(Namespace::Podcast), "valueRecipient".to_string()),
        children,
        errors,
        attributes,
    }
}

fn analyze_podcast_location(location: &badpod::podcast::Location) -> Node {
    let mut children = Vec::new();
    let mut errors = Vec::new();
    let mut attributes = Vec::new();

    if let Some(value) = &location.value {
        if value.len() > 128 {
            errors.push(Error::AttributeExceedsMaxLength(
                "value".to_string(),
                value.to_string(),
                128,
            ));
        } else {
            attributes.push(("value".to_string(), format!("\"{}\"", value)));
        }
    } else {
        errors.push(Error::MissingAttribute("value".to_string()));
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
                attributes.push(("geo".to_string(), geo_str.to_string()));
            }
            badpod::podcast::Geo::Other(s) => {
                errors.push(Error::InvalidAttribute("geo".to_string(), s.to_string()));
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
                attributes.push(("osm".to_string(), osm_str.to_string()));
            }
            badpod::podcast::Osm::Other(s) => {
                errors.push(Error::InvalidAttribute("osm".to_string(), s.to_string()));
            }
        }
    }

    Node {
        name: TagName(Some(Namespace::Podcast), "location".to_string()),
        children,
        errors,
        attributes,
    }
}

fn analyze_podcast_guid(guid: &badpod::podcast::Guid) -> Node {
    let mut errors = Vec::new();
    let mut attributes = Vec::new();

    match guid {
        badpod::podcast::Guid::Ok(guid) => {
            attributes.push(("value".to_string(), format!("\"{}\"", guid)));
        }
        badpod::podcast::Guid::Other(guid) => {
            errors.push(Error::InvalidAttribute(
                "value".to_string(),
                guid.to_string(),
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
        badpod::podcast::Medium::Other(medium) => {
            errors.push(Error::InvalidAttribute(
                "value".to_string(),
                medium.to_string(),
            ));
        }
        _ => {
            attributes.push(("value".to_string(), medium.to_string()));
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
        attributes.push(("value".to_string(), format!("\"{}\"", value)));
    } else {
        errors.push(Error::MissingAttribute("value".to_string()));
    }

    if let Some(service) = &txt.purpose {
        attributes.push(("service".to_string(), format!("\"{}\"", service)));
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
            attributes.push(("value".to_string(), b.to_string()));
        }
        Some(badpod::Bool::Other(s)) => {
            errors.push(Error::InvalidAttribute("value".to_string(), s.to_string()));
        }
        None => {
            errors.push(Error::MissingAttribute("value".to_string()));
        }
    }

    if let Some(id) = &block.id {
        match id {
            badpod::podcast::Service::Other(s) => {
                errors.push(Error::InvalidAttribute("id".to_string(), s.to_string()));
            }
            _ => {
                attributes.push(("id".to_string(), id.to_string()));
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
            attributes.push(("value".to_string(), b.to_string()));
        }
        Some(badpod::Bool::Other(b)) => {
            errors.push(Error::InvalidAttribute("value".to_string(), b.to_string()));
        }
        None => {
            errors.push(Error::MissingAttribute("value".to_string()));
        }
    }

    if let Some(owner) = &locked.owner {
        attributes.push(("owner".to_string(), format!("\"{}\"", owner)));
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
                "value".to_string(),
                value.to_string(),
                128,
            ));
        } else {
            attributes.push(("value".to_string(), format!("\"{}\"", value)));
        }
    } else {
        errors.push(Error::MissingAttribute("value".to_string()));
    }

    if let Some(url) = &funding.url {
        attributes.push(("url".to_string(), format!("\"{}\"", url)));
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
                "value".to_string(),
                name.to_string(),
                128,
            ));
        } else {
            attributes.push(("value".to_string(), format!("\"{}\"", name)));
        }
    } else {
        errors.push(Error::MissingAttribute("value".to_string()));
    }

    if let Some(group) = &person.group {
        match group {
            badpod::podcast::PersonGroup::Other(s) => {
                errors.push(Error::InvalidAttribute("group".to_string(), s.to_string()));
            }
            _ => {
                attributes.push(("group".to_string(), group.to_string()));
            }
        }
    }

    if let Some(role) = &person.role {
        match role {
            badpod::podcast::PersonRole::Other(s) => {
                errors.push(Error::InvalidAttribute("role".to_string(), s.to_string()));
            }
            _ => {
                attributes.push(("role".to_string(), role.to_string()));
            }
        }
    }

    if let Some(image) = &person.img {
        attributes.push(("img".to_string(), format!("\"{}\"", image)));
    }

    if let Some(href) = &person.href {
        attributes.push(("href".to_string(), format!("\"{}\"", href)));
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
                "value".to_string(),
                tile.to_string(),
                128,
            ));
        } else {
            attributes.push(("value".to_string(), format!("\"{}\"", tile)));
        }
    } else {
        errors.push(Error::MissingAttribute("value".to_string()));
    }

    if let Some(url) = &trailer.url {
        attributes.push(("url".to_string(), format!("\"{}\"", url)));
    } else {
        errors.push(Error::MissingAttribute("url".to_string()));
    }

    if let Some(pub_date) = &trailer.pub_date {
        match pub_date {
            badpod::DateTime::Ok(dt) => {
                attributes.push(("pubDate".to_string(), dt.to_string()));
            }
            badpod::DateTime::Other(s) => {
                errors.push(Error::InvalidAttribute(
                    "pubDate".to_string(),
                    s.to_string(),
                ));
            }
        }
    }

    if let Some(length) = &trailer.length {
        match length {
            badpod::Integer::Ok(i) => {
                attributes.push(("length".to_string(), i.to_string()));
            }
            badpod::Integer::Other(s) => {
                errors.push(Error::InvalidAttribute("length".to_string(), s.to_string()));
            }
        }
    }

    if let Some(mimetype) = &trailer.type_ {
        match mimetype {
            badpod::MimeEnclosure::Other(s) => {
                errors.push(Error::InvalidAttribute("type".to_string(), s.to_string()));
            }
            _ => {
                attributes.push(("type".to_string(), mimetype.to_string()));
            }
        }
    }

    if let Some(season) = &trailer.season {
        match season {
            badpod::Integer::Ok(i) => {
                attributes.push(("season".to_string(), i.to_string()));
            }
            badpod::Integer::Other(s) => {
                errors.push(Error::InvalidAttribute("season".to_string(), s.to_string()));
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
        attributes.push(("url".to_string(), format!("\"{}\"", url)));
    }

    if let Some(value) = &license.value {
        match value {
            badpod::podcast::LicenseType::Other(s) => {
                if s.len() > 128 {
                    errors.push(Error::AttributeExceedsMaxLength(
                        "value".to_string(),
                        s.to_string(),
                        128,
                    ));
                } else {
                    attributes.push(("value".to_string(), format!("\"{}\"", s)));
                }
                if license.url.is_none() {
                    errors.push(Error::MissingAttribute("url".to_string()));
                }
            }
            _ => {
                attributes.push(("value".to_string(), value.to_string()));
            }
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
    let mut img_strs = Vec::new();

    for image in &images.srcset {
        match image {
            badpod::podcast::Image::Ok(url, width) => {
                img_strs.push(format!("{{ url: \"{}\", width: {} }}", url, width));
            }
            badpod::podcast::Image::Other(s) => {
                let errors = vec![Error::InvalidAttribute("srcset".to_string(), s.to_string())];
                return Node {
                    name: TagName(Some(Namespace::Podcast), "images".to_string()),
                    errors,
                    ..Default::default()
                };
            }
        }
    }
    let value = format!("[{}]", img_strs.join(", "));
    let attributes = vec![("srcset".to_string(), value)];

    Node {
        name: TagName(Some(Namespace::Podcast), "images".to_string()),
        attributes,
        ..Default::default()
    }
}

fn analyze_podcast_transcript(transcript: &badpod::podcast::Transcript) -> Node {
    let mut errors = Vec::new();
    let mut attributes = Vec::new();

    if let Some(url) = &transcript.url {
        attributes.push(("url".to_string(), format!("\"{}\"", url)));
    } else {
        errors.push(Error::MissingAttribute("url".to_string()));
    }

    if let Some(type_) = &transcript.type_ {
        match type_ {
            badpod::MimeTranscript::ApplicationSrt => {
                errors.push(Error::Custom(
                    "\"<code>application/srt</code>\" in attribute <code class=\"font-bold\">type</code> is not a valid mime type. <a class=\"link\" href=\"https://github.com/Podcastindex-org/podcast-namespace/pull/331\" target=\"_blank\" rel=\"noopener noreferrer\">On February 3, 2022</a>, the recognized alternative for SubRip files in the podcast namespace specification became \"<code>application/x-subrip</code>\". However, keep in mind that although podcast players like Podverse and Podcast Addict have updated their codebases, some other apps may still only recognize \"<code>application/srt</code>\" at this time."
                    .to_string(),
                ));
            }
            badpod::MimeTranscript::Other(s) => {
                errors.push(Error::InvalidAttribute("type".to_string(), s.to_string()));
            }
            _ => {
                attributes.push(("type".to_string(), type_.to_string()));
            }
        }
    } else {
        errors.push(Error::MissingAttribute("type".to_string()));
    }

    if let Some(language) = &transcript.language {
        match language {
            badpod::Language::Other(s) => {
                errors.push(Error::InvalidAttribute(
                    "language".to_string(),
                    s.to_string(),
                ));
            }
            _ => {
                attributes.push(("language".to_string(), language.to_string()));
            }
        }
    }

    if let Some(rel) = &transcript.rel {
        match rel {
            badpod::podcast::TranscriptRel::Other(s) => {
                errors.push(Error::InvalidAttribute("rel".to_string(), s.to_string()));
            }
            _ => {
                attributes.push(("rel".to_string(), rel.to_string()));
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
        attributes.push(("url".to_string(), format!("\"{}\"", url)));
    } else {
        errors.push(Error::MissingAttribute("url".to_string()));
    }

    if let Some(type_) = &chapters.type_ {
        match type_ {
            badpod::MimeChapters::Other(s) => {
                errors.push(Error::InvalidAttribute("type".to_string(), s.to_string()));
            }
            _ => {
                attributes.push(("type".to_string(), type_.to_string()));
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
            attributes.push(("start".to_string(), f.to_string()));
        }
        Some(badpod::Float::Other(s)) => {
            errors.push(Error::InvalidAttribute("start".to_string(), s.to_string()));
        }
        None => {
            errors.push(Error::MissingAttribute("start".to_string()));
        }
    }

    match &soundbite.duration {
        Some(badpod::Float::Ok(f)) => {
            attributes.push(("duration".to_string(), f.to_string()));
        }
        Some(badpod::Float::Other(s)) => {
            errors.push(Error::InvalidAttribute(
                "duration".to_string(),
                s.to_string(),
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
            attributes.push(("value".to_string(), i.to_string()));
        }
        Some(badpod::Integer::Other(s)) => {
            errors.push(Error::InvalidAttribute("value".to_string(), s.to_string()));
        }
        None => {
            errors.push(Error::MissingAttribute("value".to_string()));
        }
    }

    if let Some(name) = &season.name {
        attributes.push(("name".to_string(), format!("\"{}\"", name)));
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
        Some(badpod::Number::Other(s)) => {
            errors.push(Error::InvalidAttribute("value".to_string(), s.to_string()));
        }
        Some(n) => {
            attributes.push(("value".to_string(), n.to_string()));
        }
        None => {
            errors.push(Error::MissingAttribute("value".to_string()));
        }
    }

    if let Some(display) = &episode.display {
        attributes.push(("display".to_string(), format!("\"{}\"", display)));
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
            badpod::MimeEnclosure::Other(s) => {
                errors.push(Error::InvalidAttribute("type".to_string(), s.to_string()));
            }
            _ => {
                attributes.push(("type".to_string(), type_.to_string()));
            }
        }
    } else {
        errors.push(Error::MissingAttribute("type".to_string()));
    }

    if let Some(length) = &alternate_enclosure.length {
        match length {
            badpod::Integer::Other(s) => {
                errors.push(Error::InvalidAttribute("length".to_string(), s.to_string()));
            }
            _ => {
                attributes.push(("length".to_string(), length.to_string()));
            }
        }
    }

    if let Some(bit_rate) = &alternate_enclosure.bit_rate {
        match bit_rate {
            badpod::Float::Other(s) => {
                errors.push(Error::InvalidAttribute(
                    "bitrate".to_string(),
                    s.to_string(),
                ));
            }
            _ => {
                attributes.push(("bitrate".to_string(), bit_rate.to_string()));
            }
        }
    }

    if let Some(height) = &alternate_enclosure.height {
        match height {
            badpod::Integer::Other(s) => {
                errors.push(Error::InvalidAttribute("height".to_string(), s.to_string()));
            }
            _ => {
                attributes.push(("height".to_string(), height.to_string()));
            }
        }
    }

    if let Some(lang) = &alternate_enclosure.language {
        attributes.push(("lang".to_string(), lang.to_string()));
    }

    if let Some(title) = &alternate_enclosure.title {
        attributes.push(("title".to_string(), format!("\"{}\"", title)));
    }

    if let Some(rel) = &alternate_enclosure.rel {
        if rel.len() > 32 {
            errors.push(Error::AttributeExceedsMaxLength(
                "rel".to_string(),
                rel.to_string(),
                32,
            ));
        } else {
            attributes.push(("rel".to_string(), format!("\"{}\"", rel)));
        }
    }

    if let Some(default) = &alternate_enclosure.default {
        match default {
            badpod::Bool::Other(s) => {
                errors.push(Error::InvalidAttribute(
                    "default".to_string(),
                    s.to_string(),
                ));
            }
            _ => {
                attributes.push(("default".to_string(), default.to_string()));
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
        attributes.push(("uri".to_string(), format!("\"{}\"", uri)));
    } else {
        errors.push(Error::MissingAttribute("uri".to_string()));
    }

    if let Some(content_type) = &source.type_ {
        match content_type {
            badpod::MimeEnclosure::Other(s) => {
                errors.push(Error::InvalidAttribute(
                    "contentType".to_string(),
                    s.to_string(),
                ));
            }
            _ => {
                attributes.push(("contentType".to_string(), content_type.to_string()));
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
        Some(badpod::podcast::IntegrityType::Other(s)) => {
            errors.push(Error::InvalidAttribute("type".to_string(), s.to_string()));
        }
        Some(t) => {
            attributes.push(("type".to_string(), t.to_string()));
        }
        None => {
            errors.push(Error::MissingAttribute("type".to_string()));
        }
    }

    if let Some(value) = &integrity.value {
        attributes.push(("value_attr".to_string(), format!("\"{}\"", value)));
    } else {
        errors.push(Error::MissingAttribute("value_attr".to_string()));
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
        attributes.push(("uri".to_string(), format!("\"{}\"", uri)));
    } else {
        errors.push(Error::MissingAttribute("uri".to_string()));
    }

    match &social_interact.protocol {
        Some(badpod::podcast::SocialProtocol::Other(s)) => {
            errors.push(Error::InvalidAttribute(
                "protocol".to_string(),
                s.to_string(),
            ));
        }
        Some(p) => {
            attributes.push(("protocol".to_string(), p.to_string()));
        }
        None => {
            errors.push(Error::MissingAttribute("protocol".to_string()));
        }
    }

    if let Some(account_id) = &social_interact.account_id {
        attributes.push(("accountId".to_string(), format!("\"{}\"", account_id)));
    }

    if let Some(account_url) = &social_interact.account_url {
        attributes.push(("accountUrl".to_string(), format!("\"{}\"", account_url)));
    }

    if let Some(priority) = &social_interact.priority {
        match priority {
            badpod::Integer::Other(s) => {
                errors.push(Error::InvalidAttribute(
                    "priority".to_string(),
                    s.to_string(),
                ));
            }
            _ => {
                attributes.push(("priority".to_string(), priority.to_string()));
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

fn analyze_podcast_content_link(content_link: &badpod::podcast::ContentLink) -> Node {
    let mut errors = Vec::new();
    let mut attributes = Vec::new();

    if let Some(value) = &content_link.value {
        attributes.push(("value".to_string(), format!("\"{}\"", value)));
    } else {
        errors.push(Error::MissingAttribute("value".to_string()));
    }

    if let Some(href) = &content_link.href {
        attributes.push(("href".to_string(), format!("\"{}\"", href)));
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
