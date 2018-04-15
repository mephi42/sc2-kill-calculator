#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use game_data::{Game, GameData, Unit};
use rocket::State;
use rocket_contrib::Json;
use std::path::Path;

mod error;
mod game_data;
mod rest;

static VERSIONS: [&str; 1] = [
    "v4.2.3.63785",
];

#[get("/versions")]
fn versions(game_data: State<GameData>) -> Result<String, error::Error> {
    let mut versions: Vec<String> = game_data.keys().map(|x| x.clone()).collect();
    versions.sort();
    let result = serde_json::to_string(&rest::VersionsResponse {
        versions,
    })?;
    Ok(result)
}

fn get_game<'a>(game_data: &'a GameData, version: &str) -> Result<&'a Game, error::Error> {
    match game_data.get(version) {
        Some(x) => Ok(x),
        None => Err(error::Error::WebApp(String::from("No such version"))),
    }
}

#[get("/versions/<version>")]
fn version(version: String, game_data: State<GameData>) -> Result<String, error::Error> {
    let game = get_game(&game_data, &version)?;
    let races = game.races.clone();
    let result = serde_json::to_string(&rest::VersionResponse {
        races,
    })?;
    Ok(result)
}

fn get_units<'a>(game: &'a Game, race: &str) -> Result<Vec<&'a Unit>, error::Error> {
    let units: Vec<&'a Unit> = game.unit_data.values().filter(|x| {
        !x.invulnerable && !x.ability_commands.is_empty() && x.race == race
    }).collect();
    if units.is_empty() {
        Err(error::Error::WebApp(String::from("No such race")))
    } else {
        Ok(units)
    }
}

fn rest_unit(unit: &Unit) -> rest::Unit {
    rest::Unit {
        name: unit.name.clone(),
    }
}

#[post("/versions/<version>/matchup", data = "<request>")]
fn matchup(version: String, request: Json<rest::MatchupRequest>, game_data: State<GameData>) -> Result<String, error::Error> {
    let game = get_game(&game_data, &version)?;
    let attacker_units = get_units(&game, &request.attacker_race)?;
    let defender_units = get_units(&game, &request.defender_race)?;
    let result = serde_json::to_string(&rest::MatchupResponse {
        attackers: attacker_units.iter().map(|x| rest_unit(x)).collect(),
        defenders: defender_units.iter().map(|x| rest_unit(x)).collect(),
        kill_calculations: Vec::new(),
    })?;
    Ok(result)
}

fn main_impl() -> Result<(), error::Error> {
    let game_data = game_data::load(Path::new("."), &VERSIONS)?;
    rocket::ignite()
        .mount("/", routes![versions, version, matchup])
        .manage(game_data)
        .launch();
    Ok(())
}

fn main() {
    main_impl().expect("An error occurred")
}
