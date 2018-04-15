#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use rocket::State;
use std::path::Path;

mod error;
mod game_data;
mod rest;

static VERSIONS: [&str; 1] = [
    "v4.2.3.63785",
];

#[get("/versions")]
fn versions(game_data: State<game_data::GameData>) -> Result<String, error::Error> {
    let mut versions: Vec<String> = game_data.keys().map(|x| x.clone()).collect();
    versions.sort();
    let result = serde_json::to_string(&rest::VersionsResponse {
        versions,
    })?;
    Ok(result)
}

#[get("/versions/<version>")]
fn version(version: String, game_data: State<game_data::GameData>) -> Result<String, error::Error> {
    let game = match game_data.get(&version) {
        Some(x) => x,
        None => return Err(error::Error::WebApp(String::from("No such version"))),
    };
    let races = game.races.clone();
    let result = serde_json::to_string(&rest::VersionResponse {
        races,
    })?;
    Ok(result)
}

fn main_impl() -> Result<(), error::Error> {
    let game_data = game_data::load(Path::new("."), &VERSIONS)?;
    rocket::ignite()
        .mount("/", routes![versions, version])
        .manage(game_data)
        .launch();
    Ok(())
}

fn main() {
    main_impl().expect("An error occurred")
}
