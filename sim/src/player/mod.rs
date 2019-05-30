mod rand_no_trades;

extern crate lazy_static;
use crate::types::*;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::sync::Mutex;

type StrategyConstructor = fn() -> Box<PlayerStrategy>;

lazy_static! {
    static ref REGISTRY: Mutex<HashMap<String, StrategyConstructor>> = Mutex::new(HashMap::new());
}
const _DEFAULT_PLAYER_TYPE: &str = "PLAYER_NO_TRADES";

#[derive(Serialize, Deserialize)]
pub struct PlayerConfig {
    player_type: String,

    #[serde(default)]
    config: serde_json::Value,
}

pub trait PlayerStrategy {
    // Initialize the player from the given config.
    fn init(&mut self, player_id: PlayerId, value: &serde_json::Value);

    // Reset the player to the most recent init() state.
    fn reset(&mut self);

    fn propose_trades_as_lead(&mut self) -> HashMap<PlayerId, Trade>;
    fn propose_trade_as_non_lead(&mut self) -> Option<Trade>;

    fn accept_trades_as_lead(&mut self, trades: HashMap<PlayerId, Trade>) -> Vec<Trade>;
    fn accept_trades_as_non_lead(&mut self, trade: Trade) -> Option<Trade>;
}

pub fn register_strategy(player_type: &str, constructor: StrategyConstructor) {
    REGISTRY
        .lock()
        .unwrap()
        .insert(player_type.to_string(), constructor);
}

pub fn load_strategies(
    configs: &Vec<PlayerConfig>,
    num_players: usize,
) -> Vec<Box<PlayerStrategy>> {
    let mut strategies: Vec<Box<PlayerStrategy>> = Vec::new();

    assert!(configs.len() <= num_players);
    for i in 0..num_players {
        strategies.push(if i < configs.len() {
            let config = &configs[i];
            let mut strategy = REGISTRY.lock().unwrap()[&config.player_type]();
            strategy.init(i, &config.config);
            strategy
        } else {
            // default
            REGISTRY.lock().unwrap()[_DEFAULT_PLAYER_TYPE]()
        })
    }

    strategies
}
