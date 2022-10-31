use rocket::fs::{relative, FileServer};
use rocket::get;
use rocket::launch;
use rocket::response::content::RawHtml;

static INDEX_FILE: &str = include_str!("../app/dist/index.html");

#[get("/")]
async fn index() -> RawHtml<String> {
    RawHtml(INDEX_FILE.replace("<title></title>", "<title>RSS Blue Tools</title>"))
}

#[get("/podcast-guid")]
async fn podcast_guid() -> RawHtml<String> {
    RawHtml(INDEX_FILE.replace(
        "<title></title>",
        "<title>Podcast GUID | RSS Blue Tools</title>",
    ))
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", rocket::routes![index, podcast_guid])
        .mount("/", FileServer::from(relative!("/app/dist")))
}
