use sycamore::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{KeyboardEvent, Event};
use uuid::Uuid;
use wasm_bindgen::JsValue;

#[derive(Debug, Clone, PartialEq)]
struct Chapter {
    id: Uuid,
    title: String,
    start_time: f64,
}

#[derive(Debug, Default, Clone)]
pub struct AppState {
    chapters: RcSignal<Vec<RcSignal<Chapter>>>,
}

impl AppState {
    fn add_chapter(&self, title: String, start_time: f64) {
        self.chapters.modify().push(create_rc_signal(Chapter {
            title,
            start_time,
            id: Uuid::new_v4(),
        }))
    }

    fn remove_chapter(&self, id: Uuid) {
        self.chapters.modify().retain(|chapter| chapter.get().id != id);
    }
}

#[component]
pub fn Chapters<G: Html>(cx: Scope) -> View<G> {
    let app_state = AppState {
        chapters: Default::default(),
    };
    let app_state = provide_context(cx, app_state);

    let chapters_empty = create_selector(cx, || app_state.chapters.get().is_empty());

    view! { cx,
        crate::components::ToolsBreadcrumbs(title="Chapters")
        h1(class="mb-3") { "Chapters" }
        h2(class="mt-3 text-gray-500") { "Generate podcast chapters." }
        p(class="my-7") {
            "It's easy!"
        }

        HeaderHTML {}
            (if *chapters_empty.get() {
                view! { cx,
                    "Empty"
                }
            } else {
                    view! { cx,
                        ChaptersListHTML {}
                    }
                })
    }
}

#[component]
fn HeaderHTML<G: Html>(cx: Scope) -> View<G> {
    let app_state = use_context::<AppState>(cx);
    let input_value = create_signal(cx, String::new());

    let handle_keyup = |event: Event| {
        let keyup_event: KeyboardEvent = event.unchecked_into();
        let key = keyup_event.key();
        if key == "Enter" {
            let mut title = input_value.get().as_ref().clone();
            title = title.trim().to_string();

            if !title.is_empty() {
                web_sys::console::log_1(&JsValue::from_str(&format!("Title: {}", title)));
                app_state.add_chapter(title, 0.0);
                input_value.set("".to_string());
            }
        }
    };

    view! { cx,
        input(
            placeholder="Chapter title",
            bind:value=input_value,
            on:keyup=handle_keyup,
        )
    }
}

#[component]
fn ChaptersListHTML<G: Html>(cx: Scope) -> View<G> {
    let app_state = use_context::<AppState>(cx);

    let chapters = create_memo(cx, || {
        app_state
            .chapters
            .get()
            .iter()
            .cloned()
            .collect::<Vec<_>>()
    });

    view! { cx,
        ul {
            Keyed(
        iterable=chapters,
        view=|cx, chapter| view! { cx,
            ChapterHTML(chapter=chapter)
        },
        key=|chapter| chapter.get().id,
        )
    }
    }
}

#[component(inline_props)]
fn ChapterHTML<G: Html>(cx: Scope, chapter: RcSignal<Chapter>) -> View<G> {
    let app_state = use_context::<AppState>(cx);
    // Make `chapter` live as long as the scope.
    let chapter = create_ref(cx, chapter);

    let title = || chapter.get().title.clone();
    let start_time = || chapter.get().start_time.clone();
    let id = chapter.get().id;

    let handle_destroy = move |_| {
        app_state.remove_chapter(id);
    };

    let is_editing = create_signal(cx, false);

    let input_ref = create_node_ref(cx);
    let input_value = create_signal(cx, title());

    let handle_blur = move || {
        is_editing.set(false);

        let mut value = input_value.get().as_ref().clone();
        value = value.trim().to_string();

        if value.is_empty() {
            app_state.remove_chapter(id);
        } else {
            chapter.modify().title = value;
        }
    };

    let handle_keyup = move |event: Event| {
        let keyup_event: KeyboardEvent = event.unchecked_into();
        let key = keyup_event.key();
        match key.as_str() {
        "Enter" => handle_blur(),
        "Escape" => is_editing.set(false),
        _ => (),
        }
    };

    view! { cx,
        li {
            button(
                class="mr-2 px-2 py-1 bg-danger-500 text-white rounded",
                on:click=handle_destroy
            ) { "x" }

        input(
            ref=input_ref,
            bind:value=input_value,
            on:blur=move |_| handle_blur(),
            on:keyup=handle_keyup,
        )

        span(class="ml-auto text-gray-500") { (display_time(start_time())) }
    }
}
}

fn display_time(seconds: f64) -> String {
    let hours = seconds / 3600.0;
    let minutes = (seconds % 3600.0) / 60.0;

    let seconds = seconds % 60.0;

    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}
