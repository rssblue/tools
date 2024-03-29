use crate::components::utils;
use crate::hyper_header::{ByteRangeSpec, Range};
use chrono::{DateTime, Duration, Utc};
use itertools::Itertools;
use rand::Rng;
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use sycamore::prelude::*;
use sycamore::suspense::{use_transition, Suspense};
use wasm_bindgen::JsValue;

const OP3_PREFIX: &str = "https://op3.dev/e";

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
                td(title=format!("{:.2}%", percent), style=format!("--size: {}", percent/max_percent)) {
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
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
    limit: usize,
    token: String,
) -> Result<Vec<Row>, String> {
    let client = reqwest_wasm::Client::new();

    let mut request_url = format!("https://op3.dev/api/1/redirect-logs?format=json&limit={limit}&url={url}&_from=rssblue-plot-op3");
    if let Some(start_time) = start_time {
        request_url += &format!(
            "&start={}",
            start_time.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
        );
    }
    if let Some(end_time) = end_time {
        request_url += &format!(
            "&end={}",
            end_time.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
        );
    }

    let resp = client
        .get(request_url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("could not fetch the request ({e})"))?;

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
            let msg = "Invalid OP3 API request";
            if let Some(message) = op3_response.message {
                Err(format!("{} (“{}”)", msg, message))
            } else {
                Err(msg.to_string())
            }
        }
        reqwest_wasm::StatusCode::FORBIDDEN => Err(
            "Forbidden access to OP3, try changing the authentication token in the settings"
                .to_string(),
        ),
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

async fn get_results(
    url: String,
    token: String,
    num_days: Option<usize>,
    period_num_minutes: Option<usize>,
    num_periods: Option<usize>,
    limit: usize,
) -> Result<Vec<Row>, String> {
    let start_time = num_days.map(|num_days| Utc::now() - Duration::days(num_days as i64));
    let end_time = Utc::now();

    let period_duration =
        period_num_minutes.map(|period_num_minutes| Duration::minutes(period_num_minutes as i64));

    let periods = match (start_time, num_periods, period_duration) {
        (Some(start_time), Some(num_periods), Some(period_duration)) => {
            random_periods(num_periods, period_duration, start_time, end_time)
        }
        _ => vec![],
    };

    let mut rows = Vec::new();
    if periods.is_empty() {
        let result = fetch_op3(url, start_time, Some(end_time), limit, token).await;
        match result {
            Ok(rows_) => rows = rows_,
            Err(e) => return Err(e),
        };
    } else {
        // Fetch OP3 for each period concurrently and combine.
        let results =
            futures::future::join_all(
                periods.iter().map(|(start, end)| {
                    let url = url.clone();
                    let token = token.clone();
                    async move {
                        fetch_op3(url.to_string(), Some(*start), Some(*end), limit, token).await
                    }
                }),
            )
            .await;
        for result in results {
            match result {
                Ok(rows_) => rows.extend(rows_),
                Err(e) => {
                    return Err(e);
                }
            }
        }
    };

    Ok(rows)
}

#[component(inline_props)]
pub async fn Geography<'a, G: Html>(cx: Scope<'a>, url: String, token: String) -> View<G> {
    const REQUEST_LIMIT: usize = 1000;
    const LOWER_REQUEST_LIMIT: usize = 250;

    let url = format!("https://op3.dev/e{}", url);
    let mut url = match url::Url::parse(url.as_str()) {
        Ok(url) => url,
        Err(_) => {
            return view! {cx,
            utils::Alert(type_=utils::AlertType::Danger, msg=format!("Could not parse the URL."))
            }
        }
    };
    url.set_query(None);

    // Set 'op3-url' query parameter.
    if let Some(window) = web_sys::window() {
        if let Ok(href) = window.location().href() {
            if let Ok(web_url) = web_sys::Url::new(&href) {
                web_url.search_params().set("op3-url", url.as_ref());
                let new_url = web_url.href();
                if let Ok(history) = window.history() {
                    if history
                        .push_state_with_url(&JsValue::NULL, "", Some(&new_url))
                        .is_err()
                    {
                        web_sys::console::error_1(&"Error setting query parameter".into());
                    }
                }
            }
        }
    }

    let mut num_periods = None;
    let mut num_days = None;
    let mut period_num_minutes = None;

    let mut rows = Vec::new();
    let config = vec![
        (None, None, None, REQUEST_LIMIT),    // Since beginning.
        (Some(7), None, None, REQUEST_LIMIT), // Last 7 days.
        (Some(7), Some(30), Some(100), LOWER_REQUEST_LIMIT), // Random periods in the last 7 days.
    ];
    for (num_days_, period_num_minutes_, num_periods_, limit) in config {
        num_days = num_days_;
        period_num_minutes = period_num_minutes_;
        num_periods = num_periods_;

        rows = match get_results(
            url.to_string(),
            token.to_owned(),
            num_days,
            period_num_minutes,
            num_periods,
            limit,
        )
        .await
        {
            Ok(rows) => rows,
            Err(e) => {
                return view! {cx,
                utils::Alert(type_=utils::AlertType::Danger, msg=format!("Error: {}.", e))
                }
            }
        };

        if rows.len() < REQUEST_LIMIT {
            break;
        }
    }

    let num_original_rows = rows.len();
    rows = filter_rows(rows);
    let num_filtered_rows = rows.len();
    let num_filtered_out = num_original_rows - num_filtered_rows;

    if rows.is_empty() {
        return view! { cx,
            utils::Alert(type_=utils::AlertType::Warning, msg=format!("No data found for the URL. If the file is new (or has been only recently routed through OP3), reliable data might take some time to show up."))
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

    // Improve country display names.
    for (simple_name, name) in simplified_country_names() {
        if let Some(count) = country_counts.remove(simple_name) {
            country_counts.insert(name.to_string(), count);
        }
    }

    let msg = match (num_periods, num_days, period_num_minutes) {
        (None, None, None) => {
            "Below you can find data since the beginning of the file being routed through OP3."
                .to_string()
        }
        (None, Some(num_days), _) => {
            format!("Below you can find data over the last {num_days} days.")
        }
        (Some(num_periods), Some(num_days), Some(period_num_minutes)) => {
            format!("Below you can find data from {num_periods} randomly sampled {period_num_minutes}-minute blocks over the last {num_days} days.")
        }
        // Shouldn't happen.
        _ => "".to_string(),
    };
    let extra_msg = match num_periods {
        None | Some(1) => "",
        Some(_) => " These are indicative of but not equivalent to the total number of downloads because we are randomly sampling only a fraction of all requests, and there are also limits on how many requests are returned by OP3."
    };

    let info: View<G> = View::new_fragment(vec![view! {cx,
        (msg)

        br {}
        br {}

        "Data are from " strong{ (num_filtered_rows) " file request"
            (if num_filtered_rows == 1 { "" } else { "s" })
        } " (" (num_filtered_out) " "
        (if num_filtered_out == 1 { "has" } else { "have" })
        " been filtered out)." (extra_msg)

        br {}
        br {}

        details {
            summary {
                "Filtering methodology"
            }
            ul {
                li { "Only GET requests are kept." }
                li { "For partial requests, only those that are at least 1 MB and start at byte 0 are kept." }
                li { "Only requests with unique (hashed) IP addresses are kept." }
            }
        }
    }]);

    view! { cx,
    div(class="my-6") {
        utils::AlertHTML(type_=utils::AlertType::Info, msg=info)
    }

    h2 { "Continents" }
    (plot_bars(cx, &continent_counts))
        h2 { "Countries" }
    (plot_bars(cx, &country_counts))
    }
}

fn simplified_country_names() -> Vec<(&'static str, &'static str)> {
    vec![
        ("United States of America", "USA"),
        ("United Kingdom of Great Britain and Northern Ireland", "UK"),
        ("Russian Federation", "Russia"),
        ("Moldova (Republic of)", "Moldova"),
        ("Korea (Republic of)", "South Korea"),
        ("Venezuela (Bolivarian Republic of)", "Venezuela"),
        ("Iran (Islamic Republic of)", "Iran"),
        ("Boliivia (Plurinational State of)", "Bolivia"),
        ("Tanzania, United Republic of", "Tanzania"),
        ("Virgin Islands (U.S.)", "Virgin Islands (USA)"),
        ("Taiwan, Province of China", "Taiwan"),
        ("United Arab Emirates", "UAE"),
    ]
}

#[component]
pub fn PlotOp3<G: Html>(cx: Scope<'_>) -> View<G> {
    let mut op3_url = String::new();
    // Get 'op3-url' query parameter.
    if let Some(window) = web_sys::window() {
        if let Ok(href) = window.location().href() {
            if let Ok(url) = web_sys::Url::new(&href) {
                if let Some(param) = url.search_params().get("op3-url") {
                    op3_url = param;
                }
            }
        }
    }

    let url_str = create_signal(cx, String::new());
    let fetching_data = create_signal(cx, false);
    let show_data = create_signal(cx, false);
    let input_cls = create_signal(cx, String::new());
    let token = create_signal(cx, String::new());
    let settings_open = create_signal(cx, false);
    let wrong_query_op3_url = create_signal(cx, false);

    // Initialize OP3 token.
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            if let Ok(Some(token_)) = storage.get_item("op3-token") {
                token.set(token_);
            } else {
                token.set("preview07ce".to_string());
            }
        }
    }

    create_effect(cx, move || {
        utils::change_dialog_state(*settings_open.get());
        if !*settings_open.get() {
            if let Some(window) = web_sys::window() {
                if let Ok(Some(storage)) = window.local_storage() {
                    storage
                        .set_item("op3-token", &token.get())
                        .expect("Failed to save API token to local storage");
                }
            }
        }
    });

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
            wrong_query_op3_url.set(false);
            show_data.set(true);
        }
    });

    create_effect(cx, move || fetching_data.set(transition.is_pending()));

    // Check if URL start with correct prefix.
    if !op3_url.is_empty() && !op3_url.starts_with(OP3_PREFIX) {
        wrong_query_op3_url.set(true);
    }
    if !*wrong_query_op3_url.get() {
        // Strip prefix from URL.
        if let Some(url) = op3_url.strip_prefix(OP3_PREFIX) {
            url_str.set(url.to_string());
            update(true);
        }
    }

    view! { cx,
    div(class="flex flex-row items-center") {
        crate::components::ToolsBreadcrumbs(title="Plot OP3")
            button(
                class="ml-auto text-gray-400 hover:text-gray-600",
                dangerously_set_inner_html=utils::Icon::Settings.to_string().replace("{{ class }}", "h-5 stroke-2").as_str(),
                on:click=|_| settings_open.set(true),
                ) {}
    }
    h1(class="mb-3") { "Plot OP3" }
    h2(class="mt-3 text-gray-500") { "Visualize requests for a podcast media file." }
    p(class="mt-7") {
        utils::Link(url="https://op3.dev".to_string(), text="OP3".to_string(), new_tab=true)
        " is an open-source analytics service. A podcaster can route requests to their show's media files through OP3, and the service will record all those requests. Consider "
        utils::Link(url="https://github.com/skymethod/op3#commitment-to-sustainable-development".to_string(), text="supporting OP3".to_string(), new_tab=true)
        "!"
    }

    p(class="mb-7") {
        "This tool allows to visualize the countries of origin for your typical listeners. It is being actively developed and we welcome all feedback! "
        utils::Link(url="https://github.com/rssblue/tools/issues".to_string(), text="Let us know".to_string(), new_tab=true)
        " what kinds of data you would like to see visualized."
    }

    (if *wrong_query_op3_url.get() {
        view!{ cx,
        div(class="my-4") {
            utils::Alert(type_=utils::AlertType::Danger, msg=format!("URL query parameter <code class='font-mono'>op3-url</code> should start with “{OP3_PREFIX}”."))
        }}
    } else {
        view!{ cx, }
    })

    dialog(id="settings") {
        h2(class="mt-0") { "Settings" }

        label(for="token") {
            "OP3 API token"
                small(class="text-gray-500") {
                    " (get one "
                        utils::Link(url="https://op3.dev/api/keys".to_string(), text="here".to_string(), new_tab=true)
                    ")"
                }
        }
        input(
            id="token",
            type="text",
            class="input-text font-mono",
            bind:value=token,
            )

            button(
                class="btn btn-primary w-full mt-4",
                type="button",
                tabindex="-1",
                on:click=|_| settings_open.set(false),
                ) {
                "Save"
            }

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
                    class="block w-full border border-gray-300 pl-3 pr-1 rounded-tl-xl md:rounded-l-xl w-auto flex items-center bg-gray-100 text-gray-800",
                    disabled=true,
                    ) {
                    (OP3_PREFIX)
                }
                input(
                    class=format!("input-text-base rounded-tr-xl md:rounded-none md:rounded-r-none pl-1 text-ellipsis z-10 {}", input_cls.get()),
                    spellcheck=false,
                    autofocus=true,
                    type="url",
                    id="url",
                    placeholder="/example.com/episode.mp3",
                    autocomplete="off",
                    disabled=*fetching_data.get(),
                    bind:value=url_str,
                    )
                }
                    button(
                        class=format!("btn-base btn-primary rounded-b-xl md:rounded-r-xl md:rounded-l-none col-span-4 md:col-span-1"),
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
                Geography(url=url_str.get().to_string(), token=token.get().to_string())
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
