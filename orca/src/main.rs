#[macro_use]
extern crate rocket;

mod api;
mod db;
mod server;
mod generate;

#[rocket::main]
async fn main() {
    let db_pool = db::connect(db::Config {
        connection_url: "postgres://orca@localhost/ictunion",
        max_connections: 5
    }).await.unwrap();

    let _ = api::build()
        .manage(db_pool)
        .launch()
        .await
        .unwrap();
}
