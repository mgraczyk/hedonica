use serde::{Deserialize, Serialize, Serializer};
use std::collections::HashMap;

pub type Preferences = HashMap<String, f64>;
pub type GoodsSet = HashMap<String, f64>;
pub type PlayerId = usize;

#[derive(Deserialize, Clone)]
pub struct Good {
    pub category: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Trade {
    pub proposer: PlayerId,
    pub accepter: PlayerId,
    pub from_proposor: GoodsSet,
    pub from_acceptor: GoodsSet,
}

impl Serialize for Good {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.category)
    }
}
