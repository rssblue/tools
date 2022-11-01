use ndarray::Array;
use ndarray_stats::{
    histogram::{strategies::Auto, GridBuilder},
    HistogramExt,
};
use sycamore::prelude::*;

#[component]
pub fn PlotOp3<G: Html>(cx: Scope) -> View<G> {
    let times = vec![7, 12, 5, 4, 1, 2];
    let observations = Array::from_shape_vec((6, 1), times).unwrap();
    let grid = GridBuilder::<Auto<usize>>::from_array(&observations)
        .unwrap()
        .build();

    let histogram = observations.histogram(grid.clone());

    let counts = histogram.counts().to_owned();
    let bins = grid.projections().to_owned();

    view! { cx,
    h1(class="mb-3") { "Plot OP3" }
    h2(class="mt-3 text-gray-500") { "Visualize requests for a podcast media file." }
    p(class="my-7") {
            a(
                class="link",
                href="https://op3.dev",
                target="_blank",
                rel="noopener",
                title="Opens in a new tab",
                ) { "OP3" }
        " is an open-source analytics service. A podcaster can route requests to their show's media files through OP3, and the service will record all those requests. Consider "
            a(
                class="link",
                href="https://github.com/skymethod/op3#commitment-to-sustainable-development",
                target="_blank",
                rel="noopener",
                title="Opens in a new tab",
                ) { "supporting OP3" }
        "!"
    }

    p {
        (format!("{counts:?}"))
            br {}
        (format!("{bins:?}"))
    }

    //// form(class="space-y-4") {
    ////     // Prevent submission with "Enter".
    ////     button(
    ////         type="submit",
    ////         disabled=true,
    ////         style="display: none",
    ////         aria-hidden="true"
    ////         ){}
    ////     div{
    ////         label(for="url") { "Media file's URL" }
    ////         input(
    ////             class="input-text",
    ////             spellcheck=false,
    ////             autofocus=true,
    ////             type="url",
    ////             id="url",
    ////             placeholder="https://example.com/episode-1.mp3",
    ////             autocomplete="off",
    ////             // bind:value=url_str,
    ////             )
    ////     }

    //// }

    }
}
