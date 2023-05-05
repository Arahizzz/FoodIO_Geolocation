use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Position {
    pub lat: f64,
    pub lon: f64
}

#[derive(Serialize, Deserialize)]
pub struct Distance {
    pub km: f64
}