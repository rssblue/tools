use rocket::fs::{relative, FileServer};
use rocket::get;
use rocket::launch;
use rocket::request::Request;
use rocket::response::content::RawHtml;
use rocket::{catch, catchers};

static INDEX_FILE: &str = include_str!("../app/dist/index.html");

#[get("/")]
fn index() -> RawHtml<String> {
    RawHtml(INDEX_FILE.replace("{{ title }}", "RSS Blue Tools"))
}

#[get("/podcast-guid")]
fn podcast_guid() -> RawHtml<String> {
    RawHtml(INDEX_FILE.replace("{{ title }}", "Podcast GUID | RSS Blue Tools"))
}

#[catch(404)]
fn not_found(_: &Request) -> RawHtml<String> {
    RawHtml(INDEX_FILE.replace("{{ title }}", "Page Not Found | RSS Blue Tools"))
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .register("/", catchers![not_found])
        .mount("/", rocket::routes![index, podcast_guid])
        .mount("/", FileServer::from(relative!("/app/dist")))
}
