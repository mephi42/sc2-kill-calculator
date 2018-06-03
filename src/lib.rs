#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use game_data::{Game, GameData, Unit, Weapon, WeaponData, WeaponEffect};
use rocket::State;
use rocket_contrib::Json;
use std::path::Path;

pub mod error;
mod game_data;
pub mod rest;

static VERSIONS: [&str; 1] = [
    "v4.3.2.65384",
];

#[get("/versions")]
fn versions(game_data: State<GameData>) -> Result<Json<rest::VersionsResponse>, error::Error> {
    let mut versions: Vec<String> = game_data.keys().map(|x| x.clone()).collect();
    versions.sort();
    Ok(Json(rest::VersionsResponse {
        versions,
    }))
}

fn get_game<'a>(game_data: &'a GameData, version: &str) -> Result<&'a Game, error::Error> {
    match game_data.get(version) {
        Some(x) => Ok(x),
        None => Err(error::Error::WebApp(String::from("Unsupported version"))),
    }
}

#[get("/versions/<version>")]
fn version(version: String, game_data: State<GameData>) -> Result<Json<rest::VersionResponse>, error::Error> {
    let game = get_game(&game_data, &version)?;
    let mut races = game.races.clone();
    races.sort();
    Ok(Json(rest::VersionResponse {
        races,
    }))
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

fn get_weapon<'a>(attacker: &Unit, defender: &Unit, weapons: &'a WeaponData) -> Option<&'a Weapon> {
    match attacker.weapons.get(0) {
        Some(x) => weapons.get(x),
        None => None,
    }
}

fn get_dmg(effect: &WeaponEffect) -> f32 {
    let self_dmg_amount: f32 = effect.dmg_amount.unwrap_or_else(|| 0.0);
    let children_dmg_amount: f32 = effect.set_effects
        .iter().flat_map(|x| x.iter().map(|x| get_dmg(x))).sum();
    let total_dmg_amount: f32 = self_dmg_amount + children_dmg_amount;
    total_dmg_amount.max(1.0)
}

fn get_attacks_nr(effect: &WeaponEffect) -> i32 {
    effect.persistent_count.unwrap_or_else(|| 1)
}

fn calculate_kill(attacker: &Unit, defender: &Unit, weapons: &WeaponData) -> rest::KillCalculation {
    match get_weapon(attacker, defender, weapons) {
        Some(weapon) => {
            let damage_dealt = get_dmg(&weapon.effect);
            let attacks_nr = get_attacks_nr(&weapon.effect);
            let shield_defense = 0.0;  // TODO: upgrades
            let armor_defense = defender.life_armor;  // TODO: upgrades
            let damage_dealt_shields = damage_dealt - shield_defense;
            let damade_dealt_armor = damage_dealt - armor_defense;
            let hits_shields = (defender.shields_max / damage_dealt_shields).ceil();
            let shields_spill = (damage_dealt_shields * hits_shields - defender.shields_max - armor_defense).max(0.0);
            let hits_life = ((defender.life_max - shields_spill) / damade_dealt_armor).ceil();
            rest::KillCalculation {
                can_hit: true,
                hits: ((hits_shields + hits_life) / attacks_nr as f32).ceil() as i32,
            }
        },
        None => rest::KillCalculation {
            can_hit: false,
            hits: 0,
        },
    }
}

#[post("/versions/<version>/matchup", data = "<request>")]
fn matchup(version: String, request: Json<rest::MatchupRequest>, game_data: State<GameData>) -> Result<Json<rest::MatchupResponse>, error::Error> {
    let game = get_game(&game_data, &version)?;
    let attacker_units = get_units(&game, &request.attacker_race)?;
    let defender_units = get_units(&game, &request.defender_race)?;
    Ok(Json(rest::MatchupResponse {
        attackers: attacker_units.iter().map(|x| rest_unit(x)).collect(),
        defenders: defender_units.iter().map(|x| rest_unit(x)).collect(),
        attacker_upgrades: Vec::new(),
        defender_upgrades: Vec::new(),
        kill_calculations: attacker_units.iter().map(|a| {
            defender_units.iter().map(|d| {
                calculate_kill(a, d, &game.weapon_data)
            }).collect()
        }).collect(),
    }))
}

pub fn rocket() -> Result<rocket::Rocket, error::Error> {
    let game_data = game_data::load(Path::new("sc2-gamedata"), &VERSIONS)?;
    Ok(rocket::ignite()
        .mount("/", routes![versions, version, matchup])
        .manage(game_data))
}
