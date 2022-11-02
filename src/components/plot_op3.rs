use serde::Deserialize;
use std::collections::HashMap;
use sycamore::prelude::*;
use sycamore::suspense::Suspense;

#[derive(Debug, Deserialize, Eq, Clone)]
enum Continent {
    #[serde(rename = "AF")]
    Africa,
    #[serde(rename = "AN")]
    Antarctica,
    #[serde(rename = "AS")]
    Asia,
    #[serde(rename = "EU")]
    Europe,
    #[serde(rename = "NA")]
    NorthAmerica,
    #[serde(rename = "OC")]
    Oceania,
    #[serde(rename = "SA")]
    SouthAmerica,
}

impl std::fmt::Display for Continent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Africa => write!(f, "Africa"),
            Self::Antarctica => write!(f, "Antarctica"),
            Self::Asia => write!(f, "Asia"),
            Self::Europe => write!(f, "Europe"),
            Self::NorthAmerica => write!(f, "North America"),
            Self::Oceania => write!(f, "Oceania"),
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
    let resp = match reqwest_wasm::get(
        "https://op3.dev/api/1/redirect-logs?start=-24h&format=json&token=preview07ce&limit=100",
    )
    .await
    {
        Ok(resp) => resp,
        Err(_) => return view! {cx, "Could not fetch the request."},
    };

    let op3_response = match resp.json::<Op3Response>().await {
        Ok(result) => result,
        Err(_) => return view! {cx, "Could not deserialize."},
    };

    let num_rows = op3_response.rows.len();
    let mut continent_counts: HashMap<Continent, usize> = HashMap::new();

    for row in op3_response.rows {
        match continent_counts.get(&row.continent) {
            Some(count) => continent_counts.insert(row.continent, count + 1),
            None => continent_counts.insert(row.continent, 1),
        };
    }
    let max_count = match continent_counts.clone().into_values().max() {
        Some(max_count) => max_count,
        None => num_rows,
    };

    let mut continent_counts: Vec<(Continent, usize)> = continent_counts.into_iter().collect();
    let num_continents = continent_counts.len();
    continent_counts.sort_by(|a, b| b.1.cmp(&a.1));
    let continent_count_views: View<G> = View::new_fragment(
        continent_counts
            .into_iter()
            .map(|(continent, count)| {
                view! { cx, tr {
                    th(scope="row") { (continent) }
                    td(style=format!("--size: calc( {count} / {max_count} )")) {
                        span(class="data") {
                            (count)
                        }
                    }
                } }
            })
            .collect(),
    );

    view! {cx,
    table(id="my-table", class="charts-css bar show-labels labels-align-start", style=format!("height: {}px;", num_continents*50)) {
        tbody {
            (continent_count_views)
        }
    }
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

    Suspense(fallback=view! { cx, "Loading..." }) {
        Continents {}
    }
    }
}
