use crate::components::hyper_header::{ByteRangeSpec, Range};
use crate::components::utils;
use chrono::{DateTime, Duration, Utc};
use itertools::Itertools;
use rand::Rng;
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use sycamore::prelude::*;
use sycamore::suspense::{use_transition, Suspense};

#[derive(Debug, Deserialize, Eq, Clone, Default)]
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
    #[default]
    Unknown,
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
            Self::Unknown => write!(f, "Unknown continent"),
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

#[derive(Debug, Deserialize, PartialEq, Eq)]
enum Method {
    #[serde(rename = "GET")]
    Get,
    #[serde(rename = "POST")]
    Post,
    #[serde(rename = "PUT")]
    Put,
    #[serde(rename = "DELETE")]
    Delete,
    #[serde(rename = "HEAD")]
    Head,
    #[serde(rename = "PATCH")]
    Patch,
    #[serde(rename = "OPTIONS")]
    Options,
}

pub fn deserialize_option_country<'de, D>(
    deserializer: D,
) -> Result<Option<isocountry::CountryCode>, D::Error>
where
    D: Deserializer<'de>,
{
    match isocountry::CountryCode::deserialize(deserializer) {
        Ok(s) => Ok(Some(s)),
        Err(_) => Ok(None),
    }
}

#[derive(Debug, Deserialize, PartialEq)]
struct Row {
    #[serde(default)]
    continent: Option<Continent>,
    #[serde(default, deserialize_with = "deserialize_option_country")]
    country: Option<isocountry::CountryCode>,
    #[serde(rename = "hashedIpAddress")]
    hashed_ip_address: String,
    method: Method,
    range: Option<Range>,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Op3Response {
    #[serde(default)]
    rows: Vec<Row>,
    message: Option<String>,
}

fn plot_bars<G: Html>(cx: Scope<'_>, data: &HashMap<String, usize>) -> View<G> {
    // Convert to vector of tuples.
    let data: Vec<(String, usize)> = data.iter().map(|(k, v)| (k.clone(), *v)).collect();

    // Sort by count.
    let mut data = data;
    data.sort_by(|(_, a), (_, b)| b.cmp(a));

    let num_points = data.len();

    // Get percentages.
    let total: usize = data.iter().map(|(_, v)| v).sum();
    let data: Vec<(String, f64)> = data
        .iter()
        .map(|(k, v)| (k.clone(), 100.0 * *v as f64 / total as f64))
        .collect();

    // Get max percentage without unwrap.
    let max_percent = data
        .iter()
        .map(|(_, v)| v)
        .fold(0.0, |acc, v| if *v > acc { *v } else { acc });

    let views: View<G> = View::new_fragment(
        data.into_iter()
            .map(|(name, percent)| {
                view! { cx, tr {
                th(scope="row") { (name) }
                td(title=format!("{:.0}%", percent), style=format!("--size: {}", percent/max_percent)) {
                    span(class="data") {
                       (if percent >= 5.0 {
                           // format as integer
                           format!("{:.0}%", percent)
                       } else {
                           "".to_string()
                       })
                    }
                } }
                }
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

async fn fetch_op3(
    url: String,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
) -> Result<Vec<Row>, String> {
    let resp = reqwest_wasm::get(
        format!("https://op3.dev/api/1/redirect-logs?format=json&token=preview07ce&limit=250&url=https://op3.dev/e/{}&start={}&end={}&_from=rssblue-plot-op3",
            url,
            start_time.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            end_time.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                ),
    )
    .await
    .map_err(|_| "could not fetch the request")?;

    let status = resp.status();
    let body = resp
        .text()
        .await
        .map_err(|_| "could not read OP3 response")?;

    let op3_response = serde_json::from_str::<Op3Response>(&body);
    let op3_response = match op3_response {
        Ok(response) => response,
        Err(e) => {
            return Err(format!(
                "could not parse OP3 response: “{e}”<br><details>
    <summary>OP3 response</summary>
    <pre><code>{body}</code></pre>
</details>"
            ))
        }
    };

    let rows = op3_response.rows;

    match status {
        reqwest_wasm::StatusCode::OK => Ok(rows),
        reqwest_wasm::StatusCode::BAD_REQUEST => {
            let msg = "invalid OP3 API request";
            if let Some(message) = op3_response.message {
                Err(format!("{} (“{}”)", msg, message))
            } else {
                Err(msg.to_string())
            }
        }
        _ => Err("unknown error".to_string()),
    }
}

fn filter_rows(rows: Vec<Row>) -> Vec<Row> {
    let mut rows = rows;

    // Only keep GET requests.
    rows.retain(|row| row.method == Method::Get);

    // Filter out rows with insufficient range.
    rows.retain(|row| match row.range.clone() {
        Some(r) => match r {
            Range::Bytes(range_specs) => {
                if range_specs.is_empty() {
                    false
                } else {
                    // Only check the first range.
                    match range_specs[0] {
                        ByteRangeSpec::FromTo(from, to) => {
                            if from != 0 {
                                return false;
                            }
                            // Only keep if more than at 1 MB.
                            if to - from < 1_000_000 {
                                return false;
                            }
                            true
                        }
                        ByteRangeSpec::AllFrom(from) => {
                            if from != 0 {
                                return false;
                            }
                            true
                        }
                        ByteRangeSpec::Last(_) => false,
                    }
                }
            }
            Range::Unregistered(_, _) => false,
        },
        None => true,
    });

    // Filter duplicate IP addresses.
    rows = rows
        .into_iter()
        .unique_by(|row| row.hashed_ip_address.clone())
        .collect();

    rows
}

#[component(inline_props)]
pub async fn Geography<'a, G: Html>(cx: Scope<'a>, url: String) -> View<G> {
    let NUM_DAYS = 7;
    let PERIOD_NUM_MINUTES = 30;
    let NUM_PERIODS = 100;

    let start_time = Utc::now() - Duration::days(NUM_DAYS);
    let end_time = Utc::now();
    let period_duration = Duration::minutes(PERIOD_NUM_MINUTES);
    let periods = random_periods(NUM_PERIODS, period_duration, start_time, end_time);

    // Fetch OP3 for each period concurrently and combine.
    let results = futures::future::join_all(periods.iter().map(|(start, end)| {
        let url = url.clone();
        async move { fetch_op3(url, *start, *end).await }
    }))
    .await;
    let mut rows = Vec::new();
    for result in results {
        match result {
            Ok(rows_) => rows.extend(rows_),
            Err(e) => {
                return view! {cx,
                utils::Warning(warning=format!("Error: {e}"))

                };
            }
        }
    }

    let num_original_rows = rows.len();
    rows = filter_rows(rows);
    let num_filtered_rows = rows.len();

    if rows.is_empty() {
        return view! { cx,
            utils::Warning(warning=format!("No data found for the URL. {periods:?}"))
        };
    }

    // Get continent and country counts.
    let mut continent_counts: HashMap<String, usize> = HashMap::new();
    let mut country_counts: HashMap<String, usize> = HashMap::new();
    for row in rows {
        if let Some(continent) = row.continent {
            *continent_counts.entry(continent.to_string()).or_insert(0) += 1;
        }
        if let Some(country) = row.country {
            *country_counts.entry(country.to_string()).or_insert(0) += 1;
        }
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
    if let Some(count) = country_counts.remove("Russian Federation") {
        country_counts.insert("Russia".to_string(), count);
    }
    if let Some(count) = country_counts.remove("Moldova (Republic of)") {
        country_counts.insert("Moldova".to_string(), count);
    }
    if let Some(count) = country_counts.remove("Korea (Republic of)") {
        country_counts.insert("South Korea".to_string(), count);
    }
    if let Some(count) = country_counts.remove("Venezuela (Bolivarian Republic of)") {
        country_counts.insert("Venezuela".to_string(), count);
    }
    if let Some(count) = country_counts.remove("Iran (Islamic Republic of)") {
        country_counts.insert("Iran".to_string(), count);
    }

    view! { cx,
    div(class="my-6") {
        utils::Info(
            info=format!("Below you can find data from {} randomly sampled {}-minute blocks over the last {} days.<br><br>Data are from <strong>{} file requests</strong> ({} have been filtered out). These are indicative of but not equivalent to the total number of downloads because we are using <em>random</em> sampling and there are limits on how many requests are returned by OP3.", NUM_PERIODS, PERIOD_NUM_MINUTES, NUM_DAYS, num_filtered_rows, num_original_rows-num_filtered_rows )
            )
    }

    h2 { "Continents" }
    (plot_bars(cx, &continent_counts))
        h2 { "Countries" }
    (plot_bars(cx, &country_counts))
    }
}

#[component]
pub fn PlotOp3<G: Html>(cx: Scope<'_>) -> View<G> {
    let url_str = create_signal(cx, String::new());
    let fetching_data = create_signal(cx, false);
    let show_data = create_signal(cx, false);
    let input_cls = create_signal(cx, String::new());

    let transition = use_transition(cx);
    let update = move |x| transition.start(move || fetching_data.set(x), || ());

    create_effect(cx, move || {
        if *fetching_data.get() {
            input_cls.set("bg-gray-100".to_string());
        } else {
            input_cls.set("".to_string());
        }
    });

    create_effect(cx, move || {
        if *fetching_data.get() {
            show_data.set(true);
        }
    });

    create_effect(cx, move || fetching_data.set(transition.is_pending()));

    view! { cx,
    h1(class="mb-3") { "Plot OP3 🚧" }
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

    div(class="my-4") {
        utils::Warning(warning="This tool is being actively developed and is not yet ready for production; however, any feedback is welcome! <a href='https://github.com/rssblue/tools/issues' class='link' target='_blank' rel='noopenener'>Let us know</a> what kinds of data you would like to see visualized.".to_string())
    }

    form(class="mb-4") {
        // Prevent submission with "Enter".
        button(
            type="submit",
            disabled=true,
            style="display: none",
            aria-hidden="true"
            ){}
        div{
            label(for="url") { "Media file's URL" }
            div(class="grid grid-cols-4") {
                div(class="flex flex-row col-span-4 md:col-span-3") {
                div(
                    class="block w-full border border-gray-300 pl-3 pr-1 rounded-tl-lg md:rounded-l-lg w-auto flex items-center bg-gray-100 text-gray-800",
                    disabled=true,
                    ) {
                    "https://op3.dev/e/"
                }
                input(
                    class=format!("input-text-base rounded-tr-lg md:rounded-none md:rounded-r-none pl-1 text-ellipsis {}", input_cls.get()),
                    spellcheck=false,
                    autofocus=true,
                    type="url",
                    id="url",
                    placeholder="example.com/episode.mp3",
                    autocomplete="off",
                    disabled=*fetching_data.get(),
                    bind:value=url_str,
                    )
                }
                    button(
                        class=format!("btn-base btn-primary rounded-b-lg md:rounded-r-lg md:rounded-l-none col-span-4 md:col-span-1"),
                        type="button",
                        on:click=move |_| update(true),
                        disabled=*fetching_data.get(),
                        ) {
                        (if *fetching_data.get() {
                            "Loading..."
                        } else {
                            "Fetch data"
                        })
                    }
            }

        }
    }

    (if *show_data.get() {
        view!{cx,
            Suspense(fallback=view! { cx, }) {
                Geography(url=url_str.get().to_string())
            }
        }
    } else {
        view! { cx,
            }
    })

    }
}

// Generate n random numbers that add up to 1.
fn random_weights(n: usize) -> Vec<f64> {
    let mut rng = rand::thread_rng();
    let mut weights: Vec<f64> = (0..n).map(|_| rng.gen::<f64>()).collect();
    let sum: f64 = weights.iter().sum();
    weights.iter_mut().for_each(|x| *x /= sum);
    weights
}

// Generate n non-overlapping periods.
fn random_periods(
    num_periods: usize,
    one_period_duration: Duration,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
) -> Vec<(DateTime<Utc>, DateTime<Utc>)> {
    let total_duration = end_time - start_time;

    let all_period_durations = one_period_duration * num_periods as i32;

    let num_gaps = num_periods + 1;
    let all_gap_durations = total_duration - all_period_durations;

    // Generate gap durations.
    let weights = random_weights(num_gaps);
    let gap_durations = weights
        .iter()
        .map(|x| {
            chrono::Duration::milliseconds((x * all_gap_durations.num_milliseconds() as f64) as i64)
        })
        .collect::<Vec<Duration>>();

    // Generate period intervals.
    let mut periods: Vec<(DateTime<Utc>, DateTime<Utc>)> = Vec::new();
    let mut start = start_time + gap_durations[0];
    for gap_duration in gap_durations[1..].iter() {
        let end = start + one_period_duration;
        periods.push((start, end));
        start = end + *gap_duration;
    }

    periods
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::prelude::*;

    #[test]
    fn test_random_periods() {
        // If no space left for gaps, then periods just span the whole duration.
        assert_eq!(
            random_periods(
                1,
                Duration::minutes(5),
                Utc.ymd(2020, 1, 1).and_hms(0, 0, 0),
                Utc.ymd(2020, 1, 1).and_hms(0, 5, 0)
            ),
            vec![(
                Utc.ymd(2020, 1, 1).and_hms(0, 0, 0),
                Utc.ymd(2020, 1, 1).and_hms(0, 5, 0)
            )]
        );
        assert_eq!(
            random_periods(
                3,
                Duration::minutes(5),
                Utc.ymd(2020, 1, 1).and_hms(0, 0, 0),
                Utc.ymd(2020, 1, 1).and_hms(0, 15, 0)
            ),
            vec![
                (
                    Utc.ymd(2020, 1, 1).and_hms(0, 0, 0),
                    Utc.ymd(2020, 1, 1).and_hms(0, 5, 0)
                ),
                (
                    Utc.ymd(2020, 1, 1).and_hms(0, 5, 0),
                    Utc.ymd(2020, 1, 1).and_hms(0, 10, 0)
                ),
                (
                    Utc.ymd(2020, 1, 1).and_hms(0, 10, 0),
                    Utc.ymd(2020, 1, 1).and_hms(0, 15, 0)
                )
            ]
        );
    }
}
