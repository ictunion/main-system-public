use rocket::{Build, Rocket};

mod registration;

#[get("/status")]
fn status_api() -> &'static str {
    "OK"
}

pub fn build() -> Rocket<Build> {
    rocket::build()
        .mount("/", routes![status_api])
        .mount("/registration", registration::routes())
        .register("/registration", catchers![rocket_validation::validation_catcher])
}
