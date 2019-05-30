use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type Preferences = HashMap<String, f64>;
pub type GoodsSet = HashMap<String, f64>;
pub type PlayerId = usize;

#[derive(Serialize, Deserialize)]
pub struct Good {
    pub category: String,
}

#[derive(Serialize, Deserialize)]
pub struct Trade {
    pub proposer: PlayerId,
    pub accepter: PlayerId,
    pub from_proposor: GoodsSet,
}
