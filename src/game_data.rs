extern crate serde;
extern crate serde_json;

use error::Error;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::File;
use std::iter::FromIterator;
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct Unit {
    name: String,

    race: String,

    #[serde(rename = "lifeMax")]
    life_max: f32,
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
}

fn load_game(path: &Path) -> Result<Game, Error> {
    let unit_data = load_unit_data(&path.join("units.json"))?;
    let races = {
        let uniq: HashSet<&String> = unit_data.values().map(|x| &x.race).collect();
        Vec::from_iter(uniq.into_iter().map(|x| x.clone()))
    };
    Ok(Game {
        races,
        unit_data,
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
