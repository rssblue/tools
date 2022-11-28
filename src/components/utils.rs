use sycamore::prelude::*;

pub enum AlertType {
    Success,
    Info,
    Warning,
    Danger,
}

#[component(inline_props)]
pub fn Alert<G: Html>(cx: Scope, type_: AlertType, msg: String) -> View<G> {
    let v = view! { cx, (msg) };
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
