extern crate serde;
extern crate serde_json;

use error::Error;
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct Unit {
    #[serde(rename = "lifeMax")]
    life_max: f32,
}

type UnitData = HashMap<String, Unit>;

fn load_unit_data(path: &Path) -> Result<UnitData, Error> {
    let f = File::open(path)?;
    let unit_data = serde_json::from_reader(f)?;
    Ok(unit_data)
}

pub struct GameData {
    unit_data: UnitData,
}

pub fn load(path: &Path) -> Result<GameData, Error> {
    let unit_data = load_unit_data(&path.join("units.json"))?;
    Ok(GameData {
        unit_data: unit_data,
    })
}
