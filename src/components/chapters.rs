use sycamore::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{KeyboardEvent, Event};
use uuid::Uuid;
use crate::components::utils;
use wasm_bindgen::JsValue;

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

            // span(class="ml-auto text-gray-500") { (display_time(start_time())) }

            button(
                class="ml-2 px-2 px-2 bg-danger-500 hover:bg-danger-600 text-white rounded",
                on:click=handle_destroy
            ) { "x" }
    }
}
}


#[component(inline_props)]
fn AudioHTML<G: Html>(cx: Scope, audio: RcSignal<Audio>) -> View<G> {
    const TIMELINE_RANGE: f64 = 1000.0;

    let audio = create_ref(cx, audio);
    let audio_ref = create_node_ref(cx);

    let current_time = || audio.get().current_time.clone();
    let state = || audio.get().state.clone();
    let duration = || audio.get().duration.clone();

    let timeline_ref = create_node_ref(cx);
    let handle_ref = create_node_ref(cx);

    let handle_toggle = move |_| {
        let audio_el = audio_ref.get::<DomNode>().unchecked_into::<web_sys::HtmlAudioElement>();
        match *state().get() {
            AudioState::Paused => {
                audio_el.play().unwrap();
                state().set(AudioState::Playing);
            }
            AudioState::Playing => {
                audio_el.pause();
                state().set(AudioState::Paused);
            }
        }
    };

    let handle_change_seek = move |event: Event| {
        let input = event.target().unwrap().unchecked_into::<web_sys::HtmlInputElement>();
        let value = input.value_as_number();
        let audio_el = audio_ref.get::<DomNode>().unchecked_into::<web_sys::HtmlAudioElement>();
        let duration = audio_el.duration();
        let new_time = (value / TIMELINE_RANGE) * duration;
        audio_el.set_current_time(new_time);
    };

    let handle_start_drag = move |event: Event| {
        web_sys::console::log_1(&JsValue::from_str("Start drag"));
        let handle = handle_ref.get::<DomNode>().unchecked_into::<web_sys::SvgElement>();
        let (x, _) = get_mouse_position(event);
        let offset = x - handle.get_attribute("cx").unwrap().parse::<f64>().unwrap();
        handle.set_attribute("data-offset", &offset.to_string()).unwrap();
    };

    let handle_drag = move |event: Event| {
        web_sys::console::log_1(&JsValue::from_str("Drag"));
        let handle = handle_ref.get::<DomNode>().unchecked_into::<web_sys::SvgElement>();
        if let Some(offset) = handle.get_attribute("data-offset") {
            let (x, _) = get_mouse_position(event);
            let offset = offset.parse::<f64>().unwrap();
            let mut new_x = x - offset;
            new_x = new_x.max(10.0);
            handle.set_attribute("cx", &new_x.to_string()).unwrap();
        }
    };

    let handle_end_drag = move |_| {
        web_sys::console::log_1(&JsValue::from_str("End drag"));
        let handle = handle_ref.get::<DomNode>().unchecked_into::<web_sys::SvgElement>();
        handle.remove_attribute("data-offset").unwrap();
    };

    let percent_played = create_signal(cx, 0.0);

    let handle_timeupdate = move |_| {
        let audio_el = audio_ref.get::<DomNode>().unchecked_into::<web_sys::HtmlAudioElement>();
        let audio_current_time = audio_el.current_time();
        let audio_duration = audio_el.duration();
        duration().set(audio_duration);
        let value = (audio_current_time / audio_duration) * TIMELINE_RANGE;
        let timeline = timeline_ref.get::<DomNode>().unchecked_into::<web_sys::HtmlInputElement>();
        timeline.set_value(&value.to_string());
        current_time().set(audio_current_time);

        // let ratio = value / TIMELINE_RANGE;
        // let thumb_width = 30.0;
        // let x = thumb_width/2.0 + (canvas_el.width() as f64 - thumb_width) * ratio;
        percent_played.set(value/TIMELINE_RANGE*100.0);
        web_sys::console::log_1(&format!("{}%", percent_played.get()).into());
    };

    let handle_ended = move |_| {
        state().set(AudioState::Paused);
    };

    view! { cx,
        div(class="w-full grid grid-cols-1 justify-items-center") {
            div(class="grid grid-cols-1 w-full h-20") {
                div(class="w-full relative") {
                    div(class="grid grid-cols-1 absolute text-center", style=format!("left: {}%;", percent_played)) {
                        div(class="w-6 h-6 bg-primary-500 rounded-full -ml-3")
                            div(class="border-l-2 border-primary-500 h-14")
                    }
                }
            }
            // Slider
            svg(
                class="w-full h-20",
                on:mousemove=handle_drag,
                on:mouseup=handle_end_drag,
                on:mouseleave=handle_end_drag,
                ) {
                g {
                g {
                rect(id="track-inner", class="fill-gray-300 w-full", x="0", y="6", height="2")
        rect(id="track-fill") {}
    }
    g {
        circle(
                class="fill-primary-500 cursor-pointer",
                r="7", cx="7", cy="7",
                on:mousedown=handle_start_drag,
                on:mousemove=handle_drag,
                ref=handle_ref,
            )   
}
}
}
    input(type="range", min="0", max=TIMELINE_RANGE, value="0", on:input=handle_change_seek, ref=timeline_ref,
        class="w-full")
        audio(
            ref=audio_ref,
            src=audio.get().url.get().as_str(),
            on:timeupdate=handle_timeupdate,
            on:ended=handle_ended,
            controls=false,
        )
        div(class="flex flex-row items-center") {
        button(on:click=handle_toggle) { span(dangerously_set_inner_html=state().get().toggle_icon().as_str()) }
        div(class="font-mono mx-2") {
            (seconds_to_timestamp(*current_time().get(), *duration().get()))
                span(class="text-gray-400") {
                "."
                    (tenths_of_seconds(*current_time().get()))
            }
        }
    }
}
}
}

fn seconds_to_timestamp(seconds: f64, duration: f64) -> String {
    let seconds = seconds as u64;
    if duration > 3600.0 {
        format!(
            "{:02}:{:02}:{:02}",
            seconds / 3600,
            (seconds % 3600) / 60,
            seconds % 60
        )
    } else {
        format!("{:02}:{:02}", seconds / 60, seconds % 60)
    }
}

fn tenths_of_seconds(seconds: f64) -> u8 {
    (seconds * 10.0) as u8 % 10
}

fn get_mouse_position(event: Event) -> (f64, f64) {
    let mouse_event = event.unchecked_into::<web_sys::MouseEvent>();
    let ctm = mouse_event.target().unwrap().unchecked_into::<web_sys::SvgGraphicsElement>().get_screen_ctm().unwrap();
    let x = (mouse_event.client_x() as f32 - ctm.e()) / ctm.a();
    let y = (mouse_event.client_y() as f32 - ctm.f()) / ctm.d();
    (x as f64, y as f64)
}
