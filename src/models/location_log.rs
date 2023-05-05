use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, PartialEq)]
pub struct LocationLog {
    pub lat: f64,
    pub lon: f64,
}

impl LocationLog {
    pub fn new(lat: f64, lon: f64) -> Self {
        Self { lat, lon }
    }
}

