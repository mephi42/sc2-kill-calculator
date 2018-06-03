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

#[derive(Deserialize)]
pub struct EffectArrayEntry {
    pub operation: String,
    #[serde(rename = "referenceType")]
    pub reference_type: String,
    #[serde(rename = "referenceId")]
    pub reference_id: String,
    #[serde(rename = "referenceAttribute")]
    pub reference_attribute: String,
    pub value: String,
}

#[derive(Deserialize)]
pub struct Upgrade {
    name: String,
    race: String,
    #[serde(rename = "effectArray")]
    effect_array: Vec<EffectArrayEntry>,
}

type UpgradeData = HashMap<String, Upgrade>;

pub struct Game {
    pub races: Vec<String>,
    pub unit_data: UnitData,
    pub weapon_data: WeaponData,
    pub upgrade_data: UpgradeData,
}

fn load_json<T>(path: &Path) -> Result<T, Error> where for<'de> T: serde::Deserialize<'de> {
    let f = File::open(path)?;
    let weapon_data = serde_json::from_reader(f)?;
    Ok(weapon_data)
}

fn load_game(path: &Path) -> Result<Game, Error> {
    let unit_data: UnitData = load_json(&path.join("units.json"))?;
    let weapon_data: WeaponData = load_json(&path.join("weapons.json"))?;
    let upgrade_data: UpgradeData = load_json(&path.join("upgrades.json"))?;
    let races = {
        let uniq: HashSet<&String> = unit_data.values().map(|x| &x.race).collect();
        Vec::from_iter(uniq.into_iter().map(|x| x.clone()))
    };
    Ok(Game {
        races,
        unit_data,
        weapon_data,
        upgrade_data,
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
