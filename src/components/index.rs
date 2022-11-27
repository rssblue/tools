use sycamore::prelude::*;

#[component]
pub fn Index<G: Html>(cx: Scope) -> View<G> {
    view! { cx,
    h1 { "Tools" }
    div(class="grid grid-cols-1 md:grid-cols-2 gap-2") {
        a(
            class=format!("btn btn-primary"),
            href="/podcast-guid",
            ) { "Podcast GUID" }
        a(
            class=format!("btn btn-primary"),
            href="/plot-op3",
            ) { "Plot OP3" }
        a(
            class=format!("btn btn-primary"),
            href="/validator",
            ) { "Validator" }
    }
    }
}
