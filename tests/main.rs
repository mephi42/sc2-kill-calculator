#[macro_use]
extern crate lazy_static;
extern crate rocket;
extern crate sc2_kill_calculator;
extern crate serde;
extern crate serde_json;

#[cfg(test)]
mod test {
    use rocket::http::ContentType;
    use rocket::local::Client;
    use rocket::local::LocalResponse;
    use sc2_kill_calculator::error;
    use sc2_kill_calculator::rest;
    use sc2_kill_calculator::rocket;
    use serde;
    use serde_json;

    struct Fixture {
        client: Client,
    }

    fn parse<'c, T>(mut response: LocalResponse<'c>) -> T where for<'de> T: serde::Deserialize<'de> {
        let body = response.body().expect("dispatch() failed");
        serde_json::from_reader(body.into_inner()).expect("from_reader() failed")
    }

    fn kill_calculation<'a>(matchup: &'a rest::MatchupResponse, attacker: &str, defender: &str) -> &'a rest::KillCalculation {
        let attacker_idx = matchup.attackers.iter().position(|x| x.name == attacker)
            .expect(&format!("attacker {attacker} not found", attacker = attacker));
        let defender_idx = matchup.defenders.iter().position(|x| x.name == defender)
            .expect(&format!("defender {defender} not found", defender = defender));
        matchup.kill_calculations
            .get(attacker_idx).expect(&format!("attacker index {attacker_idx} is out of bounds", attacker_idx = attacker_idx))
            .get(defender_idx).expect(&format!("defender index {defender_idx} is out of bounds", defender_idx = defender_idx))
    }

    impl Fixture {
        fn get<T>(&self, uri: &str) -> T where for<'de> T: serde::Deserialize<'de> {
            parse(self.client.get(uri).dispatch())
        }

        fn post<T, U>(&self, uri: &str, request: &U) -> T where for<'de> T: serde::Deserialize<'de>, U: serde::Serialize {
            let body = serde_json::to_string(request).expect("to_string() failed");
            parse(self.client.post(uri).header(ContentType::JSON).body(body).dispatch())
        }

        fn get_versions(&self) -> rest::VersionsResponse {
            self.get("/versions")
        }

        fn get_version(&self, version: &str) -> rest::VersionResponse {
            self.get(&format!("/versions/{version}", version = version))
        }

        fn get_matchup(&self, version: &str, request: &rest::MatchupRequest) -> rest::MatchupResponse {
            self.post(&format!("/versions/{version}/matchup", version = version), request)
        }
    }

    fn fixture() -> Result<Fixture, error::Error> {
        let rocket = rocket().expect("rocket() failed");
        let client = Client::new(rocket).expect("Client::new() failed");
        Ok(Fixture {
            client,
        })
    }

    lazy_static! {
        static ref FIXTURE: Fixture = fixture().expect("fixture() failed");
    }

    static VERSION: &str = "v4.3.2.65384";
    static PROTOSS: &str = "protoss";
    static TERRAN: &str = "terran";
    static ZERG: &str = "zerg";
    static ZEALOT: &str = "Zealot";
    static MARINE: &str = "Marine";
    static ZERGLING: &str = "Zergling";

    #[test]
    fn versions() {
        assert!(FIXTURE.get_versions().versions.iter().find(|x| *x == VERSION).is_some());
    }

    #[test]
    fn version() {
        assert_eq!(vec!["neutral", PROTOSS, TERRAN, ZERG], FIXTURE.get_version(VERSION).races);
    }

    #[test]
    fn p_v_p() {
        let matchup = FIXTURE.get_matchup(VERSION, &rest::MatchupRequest {
            attacker_race: String::from(PROTOSS),
            defender_race: String::from(PROTOSS),
        });
        assert_eq!(rest::KillCalculation {
            can_hit: true,
            hits: 11,
        }, *kill_calculation(&matchup, ZEALOT, ZEALOT))
    }

    #[test]
    fn p_v_t() {
        let matchup = FIXTURE.get_matchup(VERSION, &rest::MatchupRequest {
            attacker_race: String::from(PROTOSS),
            defender_race: String::from(TERRAN),
        });
    }

    #[test]
    fn p_v_z() {
        let matchup = FIXTURE.get_matchup(VERSION, &rest::MatchupRequest {
            attacker_race: String::from(PROTOSS),
            defender_race: String::from(ZERG),
        });
    }

    #[test]
    fn t_v_p() {
        let matchup = FIXTURE.get_matchup(VERSION, &rest::MatchupRequest {
            attacker_race: String::from(TERRAN),
            defender_race: String::from(PROTOSS),
        });
    }

    #[test]
    fn t_v_t() {
        let matchup = FIXTURE.get_matchup(VERSION, &rest::MatchupRequest {
            attacker_race: String::from(TERRAN),
            defender_race: String::from(TERRAN),
        });
    }

    #[test]
    fn t_v_z() {
        let matchup = FIXTURE.get_matchup(VERSION, &rest::MatchupRequest {
            attacker_race: String::from(TERRAN),
            defender_race: String::from(ZERG),
        });
    }

    #[test]
    fn z_v_t() {
        let matchup = FIXTURE.get_matchup(VERSION, &rest::MatchupRequest {
            attacker_race: String::from(ZERG),
            defender_race: String::from(TERRAN),
        });
    }

    #[test]
    fn z_v_p() {
        let matchup = FIXTURE.get_matchup(VERSION, &rest::MatchupRequest {
            attacker_race: String::from(ZERG),
            defender_race: String::from(PROTOSS),
        });
    }

    #[test]
    fn z_v_z() {
        let matchup = FIXTURE.get_matchup(VERSION, &rest::MatchupRequest {
            attacker_race: String::from(ZERG),
            defender_race: String::from(ZERG),
        });
    }
}
