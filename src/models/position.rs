use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Position {
    lat: f64,
    lon: f64
}

#[derive(Serialize, Deserialize)]
pub struct Distance {
    km: f64
}