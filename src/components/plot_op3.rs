use futures::executor;
use serde::Deserialize;
use std::collections::HashMap;
use sycamore::prelude::*;
use sycamore::suspense::Suspense;

#[derive(Debug, Deserialize, Eq, Clone)]
enum Continent {
    #[serde(rename = "AS")]
    Asia,
    #[serde(rename = "AU")]
    Australia,
    #[serde(rename = "EU")]
    Europe,
    #[serde(rename = "NA")]
    NorthAmerica,
    #[serde(rename = "SA")]
    SouthAmerica,
}

impl std::fmt::Display for Continent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Asia => write!(f, "Asia"),
            Self::Australia => write!(f, "Australia"),
            Self::Europe => write!(f, "Europe"),
            Self::NorthAmerica => write!(f, "North America"),
            Self::SouthAmerica => write!(f, "South America"),
        }
    }
}

impl PartialEq for Continent {
    fn eq(&self, other: &Self) -> bool {
        format!("{:?}", self) == format!("{:?}", other)
    }
}

impl std::hash::Hash for Continent {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        format!("{:?}", self).hash(state)
    }
}

#[derive(Debug, Deserialize, PartialEq)]
struct Row {
    continent: Continent,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Op3Response {
    rows: Vec<Row>,
}

#[component]
pub async fn Continents<G: Html>(cx: Scope<'_>) -> View<G> {
    let mut op3_response = Op3Response { rows: vec![] };
    if let Ok(resp) = reqwest_wasm::get(
        "https://op3.dev/api/1/redirect-logs?start=-24h&format=json&token=preview07ce&limit=10",
    )
    .await
    {
        op3_response = match resp.json::<Op3Response>().await {
            Ok(result) => result,
            Err(_) => Op3Response { rows: vec![] },
        };
    }
    let mut continent_counts: HashMap<Continent, usize> = HashMap::new();

    for row in op3_response.rows {
        match continent_counts.get(&row.continent) {
            Some(count) => continent_counts.insert(row.continent, count + 1),
            None => continent_counts.insert(row.continent, 1),
        };
    }

    let mut continent_counts: Vec<(Continent, usize)> = continent_counts.into_iter().collect();
    continent_counts.sort_by(|a, b| b.1.cmp(&a.1));
    let continent_count_views: View<G> = View::new_fragment(
        continent_counts
            .into_iter()
            .map(|(continent, count)| view! { cx, li { (continent) ": " (count)} })
            .collect(),
    );
    view! {cx,
        (continent_count_views)
    }
}

#[component]
pub fn PlotOp3<G: Html>(cx: Scope<'_>) -> View<G> {
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

    ul {
        Suspense(fallback=view! { cx, "Loading..." }) {
            Continents {}
        }
    }

    table(class="charts-css bar show-labels", style="height: 150px;") {
        tbody {
            tr {
                th(scope="row") { "North America" }
                td(style="--size: calc( 10 / 12 )") {
                    span(class="data") {
                        "10"
                    }
                }
            }
            tr {
                th(scope="row") { "South America" }
                td(style="--size: calc( 2 / 12 )") {
                    span(class="data") {
                        "2"
                    }
                }
            }
        }
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
