extern crate serde_json;

use std::io;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Json(serde_json::Error),
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io(error)
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::Json(error)
    }
}
