use sycamore::prelude::*;

#[component(inline_props)]
pub fn Warning<G: Html>(cx: Scope, warning: String) -> View<G> {
    let icon = Icon::AlertCircle
        .to_string()
        .replace("{{ class }}", "inline flex-shrink-0 mr-3 w-6 h-6 stroke-2");

    view! {cx,
    div(
        class="flex items-center alert alert-warning",
        role="alert",
        ) {
        span(
            dangerously_set_inner_html=icon.as_str(),
            ){}
        span(dangerously_set_inner_html=warning.as_str()){}
    }
    }
}

#[component(inline_props)]
pub fn Info<G: Html>(cx: Scope, info: View<G>) -> View<G> {
    let icon = Icon::Info
        .to_string()
        .replace("{{ class }}", "inline flex-shrink-0 mr-3 w-6 h-6 stroke-2");

    view! {cx,
    div(
        class="flex items-center alert alert-info",
        role="alert",
        ) {
        span(
            dangerously_set_inner_html=icon.as_str(),
            ){}
        div {
            (info)
        }
    }
    }
}

pub enum Icon {
    AlertCircle,
    ChevronRight,
    Info,
    Settings,
    X,
}

impl std::fmt::Display for Icon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let svg = match self {
            Self::AlertCircle => {
                include_str!("../../assets/svg/feather-icons/alert-circle.svg")
            }
            Self::ChevronRight => {
                include_str!("../../assets/svg/feather-icons/chevron-right.svg")
            }
            Self::Info => include_str!("../../assets/svg/feather-icons/info.svg"),
            Self::Settings => include_str!("../../assets/svg/feather-icons/settings.svg"),
            Self::X => include_str!("../../assets/svg/feather-icons/x.svg"),
        };
        write!(f, "{svg}")
    }
}
