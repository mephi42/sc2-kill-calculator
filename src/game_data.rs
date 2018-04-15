extern crate serde;
extern crate serde_json;

use error::Error;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::File;
use std::iter::FromIterator;
use std::path::Path;

#[derive(Deserialize)]
pub struct WeaponEffectImpact {
    #[serde(rename = "dmgAmount")]
    pub dmg_amount: Option<f32>,
}

#[derive(Deserialize)]
pub struct WeaponEffect {
    #[serde(rename = "dmgAmount")]
    pub dmg_amount: Option<f32>,
    pub impact: Option<WeaponEffectImpact>,
    #[serde(rename = "persistentCount")]
    pub persistent_count: Option<i32>,
    #[serde(rename = "setEffects")]
    pub set_effects: Option<Vec<WeaponEffect>>,
}

#[derive(Deserialize)]
pub struct Weapon {
    pub disabled: bool,
    pub effect: WeaponEffect,
    #[serde(rename = "filterRequires")]
    pub filter_requires: Vec<String>,
    pub name: String,
    pub period: f32,
}

pub type WeaponData = HashMap<String, Weapon>;

fn load_weapon_data(path: &Path) -> Result<WeaponData, Error> {
    let f = File::open(path)?;
    let weapon_data = serde_json::from_reader(f)?;
    Ok(weapon_data)
}

#[derive(Deserialize)]
pub struct Unit {
    #[serde(rename = "abilityCommands")]
    pub ability_commands: Vec<String>,
    pub invulnerable: bool,
    #[serde(rename = "lifeMax")]
    pub life_max: f32,
    pub name: String,
    pub race: String,
    pub weapons: Vec<String>,
}

type UnitData = HashMap<String, Unit>;

fn load_unit_data(path: &Path) -> Result<UnitData, Error> {
    let f = File::open(path)?;
    let unit_data = serde_json::from_reader(f)?;
    Ok(unit_data)
}

pub struct Game {
    pub races: Vec<String>,
    pub unit_data: UnitData,
    pub weapon_data: WeaponData,
}

fn load_game(path: &Path) -> Result<Game, Error> {
    let unit_data = load_unit_data(&path.join("units.json"))?;
    let weapon_data = load_weapon_data(&path.join("weapons.json"))?;
    let races = {
        let uniq: HashSet<&String> = unit_data.values().map(|x| &x.race).collect();
        Vec::from_iter(uniq.into_iter().map(|x| x.clone()))
    };
    Ok(Game {
        races,
        unit_data,
        weapon_data,
    })
}

pub type GameData = HashMap<String, Game>;

pub fn load(path: &Path, versions: &[&str]) -> Result<GameData, Error> {
    let mut game_data = HashMap::new();
    for version in versions {
        let game = load_game(&path.join(version))?;
        game_data.insert(String::from(*version), game);
    }
    Ok(game_data)
}
