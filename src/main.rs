use rocket::fs::NamedFile;
use rocket::fs::{relative, FileServer};
use rocket::get;
use rocket::launch;
use std::path::Path;

#[get("/")]
async fn index() -> Option<NamedFile> {
    common().await
}

#[get("/podcast-guid")]
async fn podcast_guid() -> Option<NamedFile> {
    common().await
}

async fn common() -> Option<NamedFile> {
    let path = Path::new(relative!("/app/dist-html/index.html"));

    NamedFile::open(path).await.ok()
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", rocket::routes![index, podcast_guid])
        .mount("/", FileServer::from(relative!("/app/dist")))
}
