use crate::player;
use crate::player::*;
use ctor::ctor;

struct PlayerNoTrades {}
impl PlayerStrategy for PlayerNoTrades {
    fn init(&mut self, _player_id: PlayerId, _value: &serde_json::Value) {}
    fn reset(&mut self) {}

    fn propose_trades_as_lead(&mut self) -> HashMap<PlayerId, Trade> {
        HashMap::new()
    }
    fn propose_trade_as_non_lead(&mut self) -> Option<Trade> {
        None
    }

    fn maybe_accept_trades(&mut self, trades: HashMap<PlayerId, Trade>) -> Vec<Trade> {
        return vec![];
    }
}

fn create() -> Box<PlayerStrategy> {
    Box::new(PlayerNoTrades {})
}

#[ctor]
fn init() {
    player::register_strategy(&"PLAYER_NO_TRADES", create)
}