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

    let text = resp.text().await;

    let feed = badpod::from_str(&text.unwrap());

    let root_node = analyse_rss(&feed.unwrap());

    view! { cx,
    DisplayNode(node=root_node)
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

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::MissingAttribute(attr) => {
                if attr == "value" {
                    write!(f, "Missing value")
                } else {
                    write!(f, "Missing attribute \"{}\"", attr)
                }
            }
            Error::InvalidAttribute(attr, val) => {
                if attr == "value" {
                    write!(f, "Invalid value: {}", val)
                } else {
                    write!(f, "Invalid attribute: {}={}", attr, val)
                }
            }
            Error::AttributeExceedsMaxLength(attr, val, max) => {
                if attr == "value" {
                    write!(f, "Value exceeds maximum length: {} (max: {})", val, max)
                } else {
                    write!(
                        f,
                        "Attribute exceeds maximum length: {}={} (max: {})",
                        attr, val, max
                    )
                }
            }
            Error::MissingChild(x) => write!(f, "Missing child: {}", x),
            Error::MultipleChildren(x) => write!(f, "Multiple children: {}", x),
            Error::Custom(x) => write!(f, "{}", x),
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
fn DisplayNode<'a, G: Html>(cx: Scope<'a>, node: Node) -> View<G> {
    let children = create_signal(cx, node.children.clone());
    let errors = create_signal(cx, node.errors.clone());
    let attributes = create_signal(cx, node.attributes.clone());
    let have_nested_errors = node.descendants_have_errors();

    view! { cx,
    details(open=true) {
        summary(class="font-mono text-lg") {
            "<"(node.name)">"
                (if have_nested_errors {
                    view! {cx, span(class="text-danger-500") { " ✗" } }
                } else {
                    view! {cx, }
                })
        }
        div(class="pl-3 md:pl-5") {
            Indexed(
                iterable=children,
                view=|cx, x| view! { cx,
                DisplayNode(node=x)
                },
                )
            Indexed(
                iterable=attributes,
                view=|cx, x| view! { cx,
                div(class="font-mono") {
                    span(class="font-bold") { (x.0) } "=" (x.1)
                }
                },
                )
            Indexed(
                iterable=errors,
                view=|cx, x| view! { cx,
                utils::Error(error=x.to_string())
                },
                )
        }
    }
    }
}

fn analyse_rss(rss: &badpod::Rss) -> Node {
    let mut errors = Vec::new();
    let mut children = Vec::new();

    let channel = match rss.channel.len() {
        0 => {
            errors.push(Error::MissingChild(TagName(None, "channel".to_string())));
            return Node {
                name: TagName(None, "rss".to_string()),
                children,
                errors,
                attributes: Vec::new(),
            };
        }
        1 => &rss.channel[0],
        _ => {
            errors.push(Error::MultipleChildren(TagName(
                None,
                "channel".to_string(),
            )));
            return Node {
                name: TagName(None, "rss".to_string()),
                children,
                errors,
                attributes: Vec::new(),
            };
        }
    };

    let channel = analyse_channel(channel);
    children.push(channel);

    Node {
        name: TagName(None, "rss".to_string()),
        children,
        ..Default::default()
    }
}

fn analyse_channel(channel: &badpod::Channel) -> Node {
    let mut errors = Vec::new();
    let mut children = Vec::new();

    if !channel.title.is_empty() {
        let node = Node {
            name: TagName(None, "title".to_string()),
            attributes: vec![("value".to_string(), format!("\"{}\"", channel.title[0]))],
            ..Default::default()
        };

        children.push(node);

        if channel.title.len() > 1 {
            errors.push(Error::MultipleChildren(TagName(None, "title".to_string())));
        }
    } else {
        errors.push(Error::MissingChild(TagName(None, "title".to_string())));
    }

    if !channel.podcast_guid.is_empty() {
        let mut node = Node {
            name: TagName(Some(Namespace::Podcast), "guid".to_string()),
            ..Default::default()
        };
        match &channel.podcast_guid[0] {
            badpod::podcast::Guid::Ok(guid) => {
                node.attributes
                    .push(("value".to_string(), format!("\"{}\"", guid)));
            }
            badpod::podcast::Guid::Other(guid) => {
                node.errors.push(Error::InvalidAttribute(
                    "value".to_string(),
                    guid.to_string(),
                ));
            }
        }
        children.push(node);

        if channel.podcast_guid.len() > 1 {
            errors.push(Error::MultipleChildren(TagName(
                Some(Namespace::Podcast),
                "guid".to_string(),
            )));
        }
    }

    if !channel.podcast_medium.is_empty() {
        match &channel.podcast_medium[0] {
            badpod::podcast::Medium::Other(medium) => {
                children.push(Node {
                    name: TagName(Some(Namespace::Podcast), "medium".to_string()),
                    errors: vec![Error::InvalidAttribute(
                        "value".to_string(),
                        format!("\"{}\"", medium),
                    )],
                    ..Default::default()
                });
            }
            _ => {
                children.push(Node {
                    name: TagName(Some(Namespace::Podcast), "medium".to_string()),
                    attributes: vec![("value".to_string(), channel.podcast_medium[0].to_string())],
                    ..Default::default()
                });
            }
        }

        if channel.podcast_medium.len() > 1 {
            errors.push(Error::MultipleChildren(TagName(
                Some(Namespace::Podcast),
                "medium".to_string(),
            )));
        }
    }

    for txt in &channel.podcast_txt {
        let mut node = Node {
            name: TagName(Some(Namespace::Podcast), "txt".to_string()),
            ..Default::default()
        };

        if let Some(value) = &txt.value {
            node.attributes
                .push(("value".to_string(), format!("\"{}\"", value)));
        } else {
            node.errors
                .push(Error::MissingAttribute("value".to_string()));
        }

        if let Some(service) = &txt.purpose {
            node.attributes
                .push(("service".to_string(), format!("\"{}\"", service)));
        }

        children.push(node);
    }

    for block in &channel.podcast_block {
        let mut node = Node {
            name: TagName(Some(Namespace::Podcast), "block".to_string()),
            ..Default::default()
        };

        if let Some(value) = &block.value {
            match value {
                badpod::Bool::Ok(b) => {
                    node.attributes.push(("value".to_string(), b.to_string()));
                }
                badpod::Bool::Other(b) => {
                    node.errors.push(Error::InvalidAttribute(
                        "value".to_string(),
                        format!("\"{}\"", b),
                    ));
                }
            }
        } else {
            node.errors
                .push(Error::MissingAttribute("value".to_string()));
        }

        if let Some(id) = &block.id {
            match id {
                badpod::podcast::Service::Other(s) => {
                    node.errors.push(Error::InvalidAttribute(
                        "id".to_string(),
                        format!("\"{}\"", s),
                    ));
                }
                _ => {
                    node.attributes.push(("id".to_string(), id.to_string()));
                }
            }
        }

        children.push(node);
    }

    if !channel.podcast_locked.is_empty() {
        let mut node = Node {
            name: TagName(Some(Namespace::Podcast), "locked".to_string()),
            ..Default::default()
        };

        let locked = &channel.podcast_locked[0];

        if let Some(value) = &locked.value {
            match value {
                badpod::Bool::Ok(b) => {
                    node.attributes.push(("value".to_string(), b.to_string()));
                }
                badpod::Bool::Other(b) => {
                    node.errors
                        .push(Error::InvalidAttribute("value".to_string(), b.to_string()));
                }
            }
        } else {
            node.errors
                .push(Error::MissingAttribute("value".to_string()));
        }

        if let Some(owner) = &locked.owner {
            node.attributes
                .push(("owner".to_string(), format!("\"{}\"", owner)));
        }

        if channel.podcast_locked.len() > 1 {
            errors.push(Error::MultipleChildren(TagName(
                Some(Namespace::Podcast),
                "locked".to_string(),
            )));
        }

        children.push(node);
    }

    for funding in &channel.podcast_funding {
        let mut node = Node {
            name: TagName(Some(Namespace::Podcast), "funding".to_string()),
            ..Default::default()
        };

        if let Some(value) = &funding.value {
            if value.len() > 128 {
                node.errors.push(Error::AttributeExceedsMaxLength(
                    "value".to_string(),
                    value.to_string(),
                    128,
                ));
            } else {
                node.attributes
                    .push(("value".to_string(), format!("\"{}\"", value)));
            }
        } else {
            node.errors
                .push(Error::MissingAttribute("value".to_string()));
        }

        if let Some(url) = &funding.url {
            node.attributes
                .push(("url".to_string(), format!("\"{}\"", url)));
        } else {
            node.errors.push(Error::MissingAttribute("url".to_string()));
        }

        children.push(node);
    }

    if !channel.podcast_location.is_empty() {
        let mut node = Node {
            name: TagName(Some(Namespace::Podcast), "location".to_string()),
            ..Default::default()
        };

        let location = &channel.podcast_location[0];

        if let Some(value) = &location.value {
            if value.len() > 128 {
                node.errors.push(Error::AttributeExceedsMaxLength(
                    "value".to_string(),
                    value.to_string(),
                    128,
                ));
            } else {
                node.attributes
                    .push(("value".to_string(), format!("\"{}\"", value)));
            }
        } else {
            node.errors
                .push(Error::MissingAttribute("value".to_string()));
        }

        if let Some(geo) = &location.geo {
            match geo {
                badpod::podcast::Geo::Ok {
                    latitude,
                    longitude,
                    altitude,
                    uncertainty,
                } => {
                    let mut geo_str =
                        format!("{{ latitude: {}, longitude: {}", latitude, longitude);
                    if let Some(altitude) = altitude {
                        geo_str.push_str(format!(", altitude: {}", altitude).as_str());
                    }
                    if let Some(uncertainty) = uncertainty {
                        geo_str.push_str(format!(", uncertainty: {}", uncertainty).as_str());
                    }
                    geo_str.push_str(" }");
                    node.attributes
                        .push(("geo".to_string(), geo_str.to_string()));
                }
                badpod::podcast::Geo::Other(s) => {
                    node.errors.push(Error::InvalidAttribute(
                        "geo".to_string(),
                        format!("\"{}\"", s),
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
                    node.attributes
                        .push(("osm".to_string(), osm_str.to_string()));
                }
                badpod::podcast::Osm::Other(s) => {
                    node.errors.push(Error::InvalidAttribute(
                        "osm".to_string(),
                        format!("\"{}\"", s),
                    ));
                }
            }
        }

        if channel.podcast_location.len() > 1 {
            errors.push(Error::MultipleChildren(TagName(
                Some(Namespace::Podcast),
                "location".to_string(),
            )));
        }

        children.push(node);
    }

    for person in &channel.podcast_person {
        let mut node = Node {
            name: TagName(Some(Namespace::Podcast), "person".to_string()),
            ..Default::default()
        };

        if let Some(name) = &person.value {
            if name.len() > 128 {
                node.errors.push(Error::AttributeExceedsMaxLength(
                    "value".to_string(),
                    name.to_string(),
                    128,
                ));
            } else {
                node.attributes
                    .push(("value".to_string(), format!("\"{}\"", name)));
            }
        } else {
            node.errors
                .push(Error::MissingAttribute("value".to_string()));
        }

        if let Some(group) = &person.group {
            match group {
                badpod::podcast::PersonGroup::Other(s) => {
                    node.errors.push(Error::InvalidAttribute(
                        "group".to_string(),
                        format!("\"{}\"", s),
                    ));
                }
                _ => {
                    node.attributes
                        .push(("group".to_string(), group.to_string()));
                }
            }
        }

        if let Some(role) = &person.role {
            match role {
                badpod::podcast::PersonRole::Other(s) => {
                    node.errors.push(Error::InvalidAttribute(
                        "role".to_string(),
                        format!("\"{}\"", s),
                    ));
                }
                _ => {
                    node.attributes.push(("role".to_string(), role.to_string()));
                }
            }
        }

        if let Some(image) = &person.img {
            node.attributes
                .push(("img".to_string(), format!("\"{}\"", image)));
        }

        if let Some(href) = &person.href {
            node.attributes
                .push(("href".to_string(), format!("\"{}\"", href)));
        }

        children.push(node);
    }

    for trailer in &channel.podcast_trailer {
        let mut node = Node {
            name: TagName(Some(Namespace::Podcast), "trailer".to_string()),
            ..Default::default()
        };

        if let Some(tile) = &trailer.value {
            if tile.len() > 128 {
                node.errors.push(Error::AttributeExceedsMaxLength(
                    "value".to_string(),
                    tile.to_string(),
                    128,
                ));
            } else {
                node.attributes
                    .push(("value".to_string(), format!("\"{}\"", tile)));
            }
        } else {
            node.errors
                .push(Error::MissingAttribute("value".to_string()));
        }

        if let Some(url) = &trailer.url {
            node.attributes
                .push(("url".to_string(), format!("\"{}\"", url)));
        } else {
            node.errors.push(Error::MissingAttribute("url".to_string()));
        }

        if let Some(pub_date) = &trailer.pub_date {
            match pub_date {
                badpod::DateTime::Ok(dt) => {
                    node.attributes
                        .push(("pubDate".to_string(), dt.to_string()));
                }
                badpod::DateTime::Other(s) => {
                    node.errors.push(Error::InvalidAttribute(
                        "pubDate".to_string(),
                        format!("\"{}\"", s),
                    ));
                }
            }
        }

        if let Some(length) = &trailer.length {
            match length {
                badpod::Integer::Ok(i) => {
                    node.attributes.push(("length".to_string(), i.to_string()));
                }
                badpod::Integer::Other(s) => {
                    node.errors.push(Error::InvalidAttribute(
                        "length".to_string(),
                        format!("\"{}\"", s),
                    ));
                }
            }
        }

        if let Some(mimetype) = &trailer.type_ {
            match mimetype {
                badpod::MimeEnclosure::Other(s) => {
                    node.errors.push(Error::InvalidAttribute(
                        "type".to_string(),
                        format!("\"{}\"", s),
                    ));
                }
                _ => {
                    node.attributes
                        .push(("type".to_string(), mimetype.to_string()));
                }
            }
        }

        if let Some(season) = &trailer.season {
            match season {
                badpod::Integer::Ok(i) => {
                    node.attributes.push(("season".to_string(), i.to_string()));
                }
                badpod::Integer::Other(s) => {
                    node.errors.push(Error::InvalidAttribute(
                        "season".to_string(),
                        format!("\"{}\"", s),
                    ));
                }
            }
        }

        children.push(node);
    }

    Node {
        name: TagName(None, "channel".to_string()),
        children,
        errors,
        attributes: Vec::new(),
    }
}
