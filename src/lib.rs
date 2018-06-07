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
    time: f32,
    dmg_amount: f32,
}

fn get_or_0(o: &Option<f32>) -> f32 {
    match o {
        Some(x) => *x,
        None => 0.,
    }
}

fn get_bonus_damage(effect: &WeaponEffect, defender: &Unit) -> f32 {
    match effect.dmg_attribute_bonuses {
        Some(ref dmg_attribute_bonuses) =>
            (if defender.armored { get_or_0(&dmg_attribute_bonuses.armored) } else { 0. }) +
                (if defender.biological { get_or_0(&dmg_attribute_bonuses.biological) } else { 0. }) +
                (if defender.light { get_or_0(&dmg_attribute_bonuses.light) } else { 0. }) +
                (if defender.massive { get_or_0(&dmg_attribute_bonuses.massive) } else { 0. }) +
                (if defender.mechanical { get_or_0(&dmg_attribute_bonuses.mechanical) } else { 0. }),
        None => 0.,
    }
}

fn get_damage_instances(effect: &WeaponEffect, defender: &Unit) -> Vec<DamageInstance> {
    fn go(effect: &WeaponEffect, defender: &Unit, mut time: f32, damage_instances: &mut Vec<DamageInstance>) {
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
            let dmg_amount = get_or_0(&effect.dmg_amount) + get_bonus_damage(effect, defender);
            if dmg_amount > 0.01 {
                damage_instances.push(DamageInstance { time, dmg_amount });
            }
            for set_effects in effect.set_effects.iter() {
                for set_effect in set_effects {
                    go(&set_effect, defender, time, damage_instances)
                }
            }
            time += persistent_periods[i % persistent_periods.len()]
        }
    }
    let mut damage_instances = Vec::new();
    go(effect, defender, 0., &mut damage_instances);
    damage_instances.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
    damage_instances
}

static MIN_DMG: f32 = 0.5;

fn calculate_kill(attacker: &Unit, defender: &Unit, weapons: &WeaponData) -> rest::KillCalculation {
    match get_weapon(attacker, defender, weapons) {
        Some(weapon) => {
            let instances = get_damage_instances(&weapon.effect, &defender);
            if instances.is_empty() {
                rest::KillCalculation {
                    can_hit: false,
                    hits: 0,
                    time: 0.,
                }
            } else {
                let shield_defense = 0.;  // TODO: upgrades
                let armor_defense = defender.life_armor;  // TODO: upgrades
                let mut i = 0;
                let mut life = defender.life_max;
                let mut shields = defender.shields_max;
                let mut time;
                let mut prev_time = 0.;
                loop {
                    let instance = &instances[i % instances.len()];
                    let damage_dealt = instance.dmg_amount;
                    time = (i / instances.len()) as f32 * weapon.period + instance.time;

                    // https://liquipedia.net/starcraft2/Zerg_Regeneration
                    life = (life + (time - prev_time) * defender.life_regen_rate).min(defender.life_max);

                    // https://liquipedia.net/starcraft2/Damage_Calculation
                    life -= if shields == 0. {
                        (damage_dealt - armor_defense).max(MIN_DMG)
                    } else {
                        let damage_dealt_shields = (damage_dealt - shield_defense).max(MIN_DMG);
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
                    prev_time = time;
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
                    time: (time * 100.).round() / 100.,
                }
            }
        }
        None => rest::KillCalculation {
            can_hit: false,
            hits: 0,
            time: 0.,
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
