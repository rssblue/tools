use sycamore::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{KeyboardEvent, Event};
use uuid::Uuid;
use crate::components::utils;
use wasm_bindgen::JsValue;

const TIMELINE_RANGE: f64 = 1000.0;
const TIMELINE_HEIGHT: f64 = 5.0;
const HANDLE_RADIUS: f64 = 8.0;

#[derive(Debug, Clone, PartialEq)]
struct Chapter {
    id: Uuid,
    title: String,
    start_time: f64,
}

#[derive(Debug, Clone, PartialEq, Default)]
enum AudioState {
    #[default]
    Paused,
    Playing,
}

impl AudioState {
    fn toggle_icon(&self) -> String {
        match self {
            AudioState::Paused => utils::Icon::Play.to_string().replace("{{ class }}", "h-5 stroke-2"),
            AudioState::Playing => utils::Icon::Pause.to_string().replace("{{ class }}", "h-5 stroke-2"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
struct Audio {
    url: RcSignal<String>,
    state: RcSignal<AudioState>,
    current_time: RcSignal<f64>,
    duration: RcSignal<f64>,
}

#[derive(Debug, Default, Clone)]
pub struct AppState {
    chapters: RcSignal<Vec<RcSignal<Chapter>>>,
    audio: RcSignal<Audio>,
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
        audio: create_rc_signal(Audio {
            url: create_rc_signal("https://file-examples.com/storage/fe2879c03363c669a9ef954/2017/11/file_example_MP3_700KB.mp3".to_string()),
            state: create_rc_signal(AudioState::Paused),
            current_time: create_rc_signal(0.0),
            duration: create_rc_signal(0.0),
        }),
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

        div(class="grid grid-cols-1 space-y-5") {

        (view! { cx, AudioHTML(audio=app_state.audio.clone()) })

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
        ul(class="not-prose list-none list-outside ml-0 space-y-3") {
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

            input(
                ref=input_ref,
                bind:value=input_value,
                on:blur=move |_| handle_blur(),
                on:keyup=handle_keyup,
            )

            button(
                class="ml-2 px-2 px-2 bg-danger-500 hover:bg-danger-600 text-white rounded",
                on:click=handle_destroy
            ) { "x" }
    }
}
}


#[component(inline_props)]
fn AudioHTML<G: Html>(cx: Scope, audio: RcSignal<Audio>) -> View<G> {
    let audio = create_ref(cx, audio);
    let audio_ref = create_node_ref(cx);

    let handle_x = create_signal(cx, HANDLE_RADIUS);

    let handle_ref = create_node_ref(cx);

    create_selector(cx, || {
        let new_handle_x = seconds_to_handle_x(*audio.get().current_time.get(), *audio.get().duration.get());
        handle_x.set(new_handle_x);
    });

    create_selector(cx, || {
        let new_seconds = handle_x_to_seconds(*handle_x.get(), *audio.get().duration.get());
        audio.get().current_time.set(new_seconds);
    });

    let handle_toggle = move |_| {
        let audio_el = audio_ref.get::<DomNode>().unchecked_into::<web_sys::HtmlAudioElement>();
        match *audio.get().state.get() {
            AudioState::Paused => {
                audio_el.play().unwrap();
                audio.get().state.set(AudioState::Playing);
            }
            AudioState::Playing => {
                audio_el.pause();
                audio.get().state.set(AudioState::Paused);
            }
        }
    };

    let handle_start_drag = move |event: Event| {
        let handle = handle_ref.get::<DomNode>().unchecked_into::<web_sys::SvgElement>();
        handle.set_attribute("data-dragging", "true").unwrap();
        handle.set_attribute("class", "fill-primary-700 cursor-pointer").unwrap();
    };

    let handle_drag = move |event: Event| {
        let handle = handle_ref.get::<DomNode>().unchecked_into::<web_sys::SvgElement>();
        if handle.get_attribute("data-dragging").is_some() {
            let (mouse_x, _) = mouse_position(event);

            let handle_x = mouse_x_to_handle_x(mouse_x);

            let audio_el = audio_ref.get::<DomNode>().unchecked_into::<web_sys::HtmlAudioElement>();
            let duration = audio_el.duration();
            let seconds = handle_x_to_seconds(handle_x, duration);
            audio_el.set_current_time(seconds);
        }
    };

    let handle_end_drag = move |_| {
        let handle = handle_ref.get::<DomNode>().unchecked_into::<web_sys::SvgElement>();
        handle.remove_attribute("data-dragging").unwrap();
        handle.set_attribute("class", "fill-primary-500 cursor-pointer").unwrap();
    };

    let handle_timeupdate = move |_| {
        let audio_el = audio_ref.get::<DomNode>().unchecked_into::<web_sys::HtmlAudioElement>();
        let current_time = audio_el.current_time();
        audio.get().current_time.set(current_time);
    };

    let handle_duration_set = move |_| {
        let audio_el = audio_ref.get::<DomNode>().unchecked_into::<web_sys::HtmlAudioElement>();
        let audio_duration = audio_el.duration();
        audio.get().duration.set(audio_duration);
    };

    let handle_ended = move |_| {
        audio.get().state.set(AudioState::Paused);
    };

    let handle_new_chapter = move |_| {
        let audio_el = audio_ref.get::<DomNode>().unchecked_into::<web_sys::HtmlAudioElement>();
        audio_el.pause();
        audio.get().state.set(AudioState::Paused);

        web_sys::console::log_1(&"new chapter".into());
    };

    view! { cx,
        div(
            class="w-full grid grid-cols-1 justify-items-center",
            on:mousemove=handle_drag,
            on:mouseup=handle_end_drag,
            on:mouseleave=handle_end_drag,
        ) {
            svg(
                class="w-full",
                style="height: 120px",
            ) {
                g {
                    g {
                    rect(
                        id="progress-bar",
                        class="fill-gray-300",
                        x=HANDLE_RADIUS,
                        y=(100.0 + HANDLE_RADIUS - TIMELINE_HEIGHT/2.0),
                        height=TIMELINE_HEIGHT,
                        width=format!("calc(100% - {}px)", HANDLE_RADIUS*2.0),
                    )
                    rect(id="track-fill") {}
        }


        g {
            circle(
                id="handle-new-chapter",
                class="fill-primary-500 cursor-pointer",
                r=HANDLE_RADIUS,
                cx=handle_x,
                cy=HANDLE_RADIUS,
                on:click=handle_new_chapter,
            )

            circle(
                class="fill-primary-500 cursor-pointer",
                r=HANDLE_RADIUS,
                cx=handle_x,
                cy=(100.0 + HANDLE_RADIUS),
                on:mousedown=handle_start_drag,
                ref=handle_ref,
            )   
    }
}
}
    audio(
        ref=audio_ref,
        src=audio.get().url.get().as_str(),
        on:timeupdate=handle_timeupdate,
        on:ended=handle_ended,
        on:canplay=handle_duration_set,
        controls=false,
    )
        div(class="flex flex-row items-center") {
        button(on:click=handle_toggle) { span(dangerously_set_inner_html=audio.get().state.get().toggle_icon().as_str()) }
        div(class="font-mono mx-2 select-none") {
            (seconds_to_timestamp(*audio.get().current_time.get(), *audio.get().duration.get()))
                span(class="text-gray-400") {
                "."
                    (tenths_of_seconds(*audio.get().current_time.get()))
            }
        }
    }
}
}
}

fn seconds_to_timestamp(seconds: f64, duration: f64) -> String {
    let seconds = seconds as u64;
    if duration > 3600.0 {
        return format!(
            "{:02}:{:02}:{:02}",
            seconds / 3600,
            (seconds % 3600) / 60,
            seconds % 60
        )
    }
    if duration > 60.0 {
        return format!("{:02}:{:02}", seconds / 60, seconds % 60)
    }
    format!("{:02}", seconds)
}

fn tenths_of_seconds(seconds: f64) -> u8 {
    (seconds * 10.0) as u8 % 10
}

fn mouse_position(event: Event) -> (f64, f64) {
    let mouse_event = event.unchecked_into::<web_sys::MouseEvent>();
    (mouse_event.client_x() as f64, mouse_event.client_y() as f64)
}

fn seconds_to_fraction(seconds: f64, duration: f64) -> f64 {
    seconds / duration
}

fn fraction_to_handle_x(fraction: f64) -> f64 {
    // The progress bar starts at HANDLE_RADIUS and ends at 100% - HANDLE_RADIUS
    let progress_bar = match web_sys::window().unwrap()
        .document().unwrap()
        .get_element_by_id("progress-bar") {
        Some(el) => el,
        None => return HANDLE_RADIUS,
    };
    let progress_bar = progress_bar.unchecked_into::<web_sys::HtmlElement>();

    let bounding_client_rect = progress_bar.get_bounding_client_rect();
    let width = bounding_client_rect.width();

    let handle_x = fraction * width + HANDLE_RADIUS;
    handle_x
}

fn mouse_x_to_fraction(mouse_x: f64) -> f64 {
    let progress_bar = web_sys::window().unwrap()
        .document().unwrap()
        .get_element_by_id("progress-bar")
        .unwrap()
        .unchecked_into::<web_sys::HtmlElement>();

    let bounding_client_rect = progress_bar.get_bounding_client_rect();
    let left = bounding_client_rect.left();
    let width = bounding_client_rect.width();

    let fraction = (mouse_x - left) / width;

    if fraction < 0.0 {
        return 0.0
    }
    if fraction > 1.0 {
        return 1.0
    }
    fraction 
}

fn mouse_x_to_handle_x(mouse_x: f64) -> f64 {
    let fraction = mouse_x_to_fraction(mouse_x);
    fraction_to_handle_x(fraction)
}

fn seconds_to_handle_x(seconds: f64, duration: f64) -> f64 {
    let fraction = seconds_to_fraction(seconds, duration);
    fraction_to_handle_x(fraction)
}

fn fraction_to_seconds(fraction: f64, duration: f64) -> f64 {
    fraction * duration
}

fn handle_x_to_fraction(handle_x: f64) -> f64 {
    let progress_bar = match web_sys::window().unwrap()
        .document().unwrap()
        .get_element_by_id("progress-bar") {
            Some(el) => el,
            None => return 0.0,
        };
    let progress_bar = progress_bar.unchecked_into::<web_sys::HtmlElement>();

    let bounding_client_rect = progress_bar.get_bounding_client_rect();
    let width = bounding_client_rect.width();

    let fraction = (handle_x - HANDLE_RADIUS) / width;
    fraction 
}

fn handle_x_to_seconds(handle_x: f64, duration: f64) -> f64 {
    let fraction = handle_x_to_fraction(handle_x);
    fraction_to_seconds(fraction, duration)
}
