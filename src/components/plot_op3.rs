use crate::components::utils;
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
    country: isocountry::CountryCode,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Op3Response {
    rows: Vec<Row>,
}

fn plot_bars<G: Html>(cx: Scope<'_>, data: &HashMap<String, usize>) -> View<G> {
    // Convert to vector of tuples.
    let data: Vec<(String, usize)> = data.iter().map(|(k, v)| (k.clone(), *v)).collect();

    // Sort by count.
    let mut data = data;
    data.sort_by(|(_, a), (_, b)| b.cmp(a));

    let num_points = data.len();

    let max_count = match data.iter().map(|(_, count)| count).max() {
        Some(max) => *max,
        None => 1,
    };

    let views: View<G> = View::new_fragment(
        data.into_iter()
            .map(|(name, count)| {
                view! { cx, tr {
                    th(scope="row") { (name) }
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
    table(id="my-table", class="charts-css bar show-labels labels-align-start", style=format!("height: {}px;", num_points*50)) {
        tbody {
            (views)
        }
    }
    }
}

#[component]
pub async fn Geography<G: Html>(cx: Scope<'_>) -> View<G> {
    let response = match fetch_op3().await {
        Ok(response) => response,
        Err(e) => {
            return view! { cx,
                utils::Warning(warning=format!("Error: {e}"))
            };
        }
    };

    if response.rows.is_empty() {
        return view! { cx,
            utils::Warning(warning="No data found for the URL.".to_string())
        };
    }

    // Get continent and country counts.
    let mut continent_counts: HashMap<String, usize> = HashMap::new();
    let mut country_counts: HashMap<String, usize> = HashMap::new();
    for row in response.rows {
        *continent_counts
            .entry(row.continent.to_string())
            .or_insert(0) += 1;
        *country_counts
            .entry(row.country.name().to_string())
            .or_insert(0) += 1;
    }

    // Use country abbreviations.
    if let Some(count) = country_counts.remove("United States of America") {
        country_counts.insert("USA".to_string(), count);
    }
    if let Some(count) =
        country_counts.remove("United Kingdom of Great Britain and Northern Ireland")
    {
        country_counts.insert("UK".to_string(), count);
    }

    view! { cx,
        div {
            h2 { "Continents" }
            (plot_bars(cx, &continent_counts))
            h2 { "Countries" }
            (plot_bars(cx, &country_counts))
        }
    }
}

async fn fetch_op3() -> Result<Op3Response, String> {
    let resp = reqwest_wasm::get(
        "https://op3.dev/api/1/redirect-logs?start=-24h&format=json&token=preview07ce&limit=100",
    )
    .await
    .map_err(|_| "could not fetch the request")?;

    resp.json::<Op3Response>()
        .await
        .map_err(|_| "could not deserialize".to_string())
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
        Geography {}
    }
    }
}
