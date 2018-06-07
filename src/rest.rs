#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct VersionsResponse {
    pub versions: Vec<String>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct VersionResponse {
    pub races: Vec<String>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct MatchupRequest {
    #[serde(rename = "attacker-race")]
    pub attacker_race: String,
    #[serde(rename = "defender-race")]
    pub defender_race: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Unit {
    pub name: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Upgrade {}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct KillCalculation {
    #[serde(rename = "can-hit")]
    pub can_hit: bool,
    pub hits: i32,
    pub time: f32,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct MatchupResponse {
    pub attackers: Vec<Unit>,
    pub defenders: Vec<Unit>,
    #[serde(rename = "attacker-upgrades")]
    pub attacker_upgrades: Vec<Upgrade>,
    #[serde(rename = "defender-upgrades")]
    pub defender_upgrades: Vec<Upgrade>,
    #[serde(rename = "kill-calculations")]
    pub kill_calculations: Vec<Vec<KillCalculation>>,
}
