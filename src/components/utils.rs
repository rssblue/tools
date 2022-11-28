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
pub fn Error<G: Html>(cx: Scope, error: String) -> View<G> {
    let icon = Icon::XCircle
        .to_string()
        .replace("{{ class }}", "inline flex-shrink-0 mr-3 w-6 h-6 stroke-2");

    view! {cx,
    div(
        class="flex items-center alert alert-danger",
        role="alert",
        ) {
        span(
            dangerously_set_inner_html=icon.as_str(),
            ){}
        span(dangerously_set_inner_html=error.as_str()){}
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

#[component(inline_props)]
pub fn Success<G: Html>(cx: Scope, success: String) -> View<G> {
    let icon = Icon::CheckCircle
        .to_string()
        .replace("{{ class }}", "inline flex-shrink-0 mr-3 w-6 h-6 stroke-2");

    view! {cx,
    div(
        class="flex items-center alert alert-success",
        role="alert",
        ) {
        span(
            dangerously_set_inner_html=icon.as_str(),
            ){}
        span(dangerously_set_inner_html=success.as_str()){}
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

#[component(inline_props)]
pub fn IconComponent<G: Html>(cx: Scope, icon: Icon, class: String) -> View<G> {
    let icon = icon.to_string().replace("{{ class }}", class.as_str());

    view! {cx,
    span(
        dangerously_set_inner_html=icon.as_str(),
        ){}
    }
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
