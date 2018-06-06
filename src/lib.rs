#![feature(plugin)]
#![plugin(rocket_codegen)]

#[macro_use]
extern crate log;
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

struct DamageInstance {
    timestamp: f32,
    dmg_amount: f32,
}

fn get_damage_instances(effect: &WeaponEffect, mut timestamp: f32) -> Vec<DamageInstance> {
    let mut damage_instances = Vec::new();
    let persistent_count = match effect.persistent_count {
        Some(persistent_count) => persistent_count,
        None => 1,
    };
    let default_persistent_periods = vec!(0.);
    let persistent_periods = match effect.persistent_periods {
        Some(ref persistent_periods) => &persistent_periods,
        None => &default_persistent_periods,
    };
    for i in 0..(persistent_count as usize) {
        match effect.dmg_amount {
            Some(0.) => {}
            Some(dmg_amount) =>
                damage_instances.push(DamageInstance { timestamp, dmg_amount }),
            None => {}
        }
        for set_effects in effect.set_effects.iter() {
            for set_effect in set_effects {
                damage_instances.extend(get_damage_instances(&set_effect, timestamp))
            }
        }
        timestamp += persistent_periods[i % persistent_periods.len()]
    }
    damage_instances
}

fn calculate_kill(attacker: &Unit, defender: &Unit, weapons: &WeaponData) -> rest::KillCalculation {
    match get_weapon(attacker, defender, weapons) {
        Some(weapon) => {
            let instances = get_damage_instances(&weapon.effect, 0.);
            if instances.is_empty() {
                rest::KillCalculation {
                    can_hit: false,
                    hits: 0,
                }
            } else {
                let shield_defense = 0.;  // TODO: upgrades
                let armor_defense = defender.life_armor;  // TODO: upgrades
                let mut i = 0;
                let mut life = defender.life_max;
                let mut shields = defender.shields_max;
                let mut prev_timestamp = 0.;
                loop {
                    let instance = &instances[i % instances.len()];
                    let damage_dealt = instance.dmg_amount;
                    let timestamp = (i / instances.len()) as f32 / weapon.period + instance.timestamp;

                    // https://liquipedia.net/starcraft2/Zerg_Regeneration
                    if life < defender.life_max {
                        life += (timestamp - prev_timestamp) * defender.life_regen_rate;
                    }

                    // https://liquipedia.net/starcraft2/Damage_Calculation
                    life -= if shields == 0. {
                        (damage_dealt - armor_defense).max(0.5)
                    } else {
                        let damage_dealt_shields = (damage_dealt - shield_defense).max(0.5);
                        let spill = damage_dealt_shields - shields;
                        if spill < 0. {
                            shields = -spill;
                            0.
                        } else {
                            shields = 0.;
                            (spill - armor_defense).max(0.)
                        }
                    };

                    i += 1;
                    prev_timestamp = timestamp;
                    if i >= 999 {
                        warn!("{} vs {} took too long", attacker.name, defender.name);
                        break;
                    }
                    if life < 1. {
                        break;
                    }
                };
                rest::KillCalculation {
                    can_hit: true,
                    hits: ((i + instances.len() - 1) / instances.len()) as i32,
                }
            }
        }
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
