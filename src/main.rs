#[macro_use]
extern crate serde_derive;

use std::path::Path;

mod error;
mod game_data;

fn main() {
    let game_data = game_data::load(Path::new("v4.2.3.63785")).expect("Could not load game data");
}
