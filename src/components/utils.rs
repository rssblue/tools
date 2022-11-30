use sycamore::prelude::*;
use wasm_bindgen::{JsCast, JsValue};

pub enum AlertType {
    Success,
    Info,
    Warning,
    Danger,
}

#[component(inline_props)]
pub fn Alert<G: Html>(cx: Scope, type_: AlertType, msg: String) -> View<G> {
    let v = view! { cx,
        span(
            dangerously_set_inner_html=msg.as_str(),
            ){}
    };

    view! { cx,
        AlertHTML(type_=type_, msg=v)
    }
}

#[component(inline_props)]
pub fn AlertHTML<G: Html>(cx: Scope, type_: AlertType, msg: View<G>) -> View<G> {
    let class = match type_ {
        AlertType::Success => "alert alert-success",
        AlertType::Info => "alert alert-info",
        AlertType::Warning => "alert alert-warning",
        AlertType::Danger => "alert alert-danger",
    };

    let icon = match type_ {
        AlertType::Success => Icon::CheckCircle,
        AlertType::Info => Icon::Info,
        AlertType::Warning => Icon::AlertCircle,
        AlertType::Danger => Icon::XCircle,
    };
    let icon = icon
        .to_string()
        .replace("{{ class }}", "inline flex-shrink-0 mr-3 w-6 h-6 stroke-2");

    view! {cx,
    div(
        class=format!("flex items-center {class}"),
        role="alert",
        ) {
        span(
            dangerously_set_inner_html=icon.as_str(),
            ){}
        div {
            (msg)
        }
    }
    }
}

pub enum Icon {
    AlertCircle,
    CheckCircle,
    ChevronRight,
    Info,
    Settings,
    X,
    XCircle,
}

impl std::fmt::Display for Icon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let svg = match self {
            Self::AlertCircle => {
                include_str!("../../assets/svg/feather-icons/alert-circle.svg")
            }
            Self::CheckCircle => {
                include_str!("../../assets/svg/feather-icons/check-circle.svg")
            }
            Self::ChevronRight => {
                include_str!("../../assets/svg/feather-icons/chevron-right.svg")
            }
            Self::Info => include_str!("../../assets/svg/feather-icons/info.svg"),
            Self::Settings => include_str!("../../assets/svg/feather-icons/settings.svg"),
            Self::X => include_str!("../../assets/svg/feather-icons/x.svg"),
            Self::XCircle => include_str!("../../assets/svg/feather-icons/x-circle.svg"),
        };
        write!(f, "{svg}")
    }
}

#[component(inline_props)]
pub fn Link<G: Html>(cx: Scope, url: String, text: String, new_tab: bool) -> View<G> {
    let target = if new_tab { "_blank" } else { "" };
    let rel = if new_tab { "noopener noreferrer" } else { "" };
    let title = if new_tab { "Opens in a new tab" } else { "" };
    view! {cx,
        a(
            class="link",
            href=url,
            target=target,
            rel=rel,
            title=title,
            ) {
            (text)
        }
    }
}

pub fn change_dialog_state(open: bool) {
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            if let Some(dialog) = document.get_element_by_id("settings") {
                let dialog: web_sys::HtmlDialogElement = dialog.dyn_into().unwrap();
                if open {
                    dialog.show_modal().unwrap();
                } else {
                    dialog.close();
                }
            }
        }
    }
}

pub fn get_storage() -> Result<web_sys::Storage, String> {
    let window = match web_sys::window() {
        Some(window) => window,
        None => return Err("No window".to_string()),
    };

    match window.local_storage() {
        Ok(Some(storage)) => Ok(storage),
        Ok(None) => Err("No storage".to_string()),
        Err(err) => Err(err.as_string().unwrap_or("Unknown error".to_string())),
    }
}

pub fn get_from_storage(key: &str) -> Result<Option<String>, String> {
    let storage = get_storage()?;

    storage
        .get_item(key)
        .map_err(|err| err.as_string().unwrap_or("Unknown error".to_string()))
}

pub fn set_in_storage(key: &str, value: &str) -> Result<(), String> {
    let storage = get_storage()?;

    storage
        .set_item(key, value)
        .map_err(|err| err.as_string().unwrap_or("Unknown error".to_string()))
}

pub fn remove_from_storage(key: &str) -> Result<(), String> {
    let storage = get_storage()?;

    storage
        .remove_item(key)
        .map_err(|err| err.as_string().unwrap_or("Unknown error".to_string()))
}
