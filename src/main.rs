#![feature(plugin)]
#![plugin(rocket_codegen)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::path::Path;

mod error;
mod game_data;
mod rest;

static VERSIONS: [&str; 1] = [
    "v4.2.3.63785",
];

#[get("/")]
fn versions() -> String {
    serde_json::to_string(&rest::VersionsResponse {
        versions: VERSIONS.into_iter().map(|x| String::from(*x)).collect(),
    }).expect("This should never happen")
}

fn main() {
    let game_data = VERSIONS.into_iter().map(|version| {
        game_data::load(Path::new(version)).expect("Could not load game data")
    });
    rocket::ignite()
        .mount("/versions", routes![versions])
        .launch();
}
