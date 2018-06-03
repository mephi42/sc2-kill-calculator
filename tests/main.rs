#[macro_use]
extern crate lazy_static;
extern crate rocket;
extern crate rocket_contrib;
extern crate sc2_kill_calculator;
extern crate serde;
extern crate serde_json;

#[cfg(test)]
mod test {
    use rocket::local::Client;
    use sc2_kill_calculator::rocket;
    use sc2_kill_calculator::error;
    use sc2_kill_calculator::rest;
    use serde;
    use serde_json;

    struct Fixture {
        client: Client,
    }

    impl Fixture {
        fn get<T>(&self, uri: &str) -> T where for<'de> T: serde::Deserialize<'de> {
            let mut response = self.client.get(uri).dispatch();
            let body = response.body().expect("dispatch() failed");
            serde_json::from_reader(body.into_inner()).expect("from_reader() failed")
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

    #[test]
    fn versions() {
        let response: rest::VersionsResponse = FIXTURE.get("/versions");
        assert!(response.versions.iter().find(|x| *x == VERSION).is_some());
    }
}
