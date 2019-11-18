use crate::game::GameState;
use crate::player;
use crate::player::*;
use ctor::ctor;

struct PlayerNoTrades {}
impl PlayerStrategy for PlayerNoTrades {
    fn init(&mut self, _player_id: PlayerId, _value: &serde_json::Value) {}

    fn reset(&mut self) {}

    fn propose_trades_as_lead(&mut self, _game_state: &GameState) -> HashMap<PlayerId, Trade> {
        HashMap::new()
    }

    fn propose_trade_as_non_lead(&mut self, _game_state: &GameState) -> Option<Trade> {
        None
    }

    fn accept_trades_as_lead(&mut self, _game_state: &GameState) -> Vec<bool> {
        vec![false; _game_state.current_trade_proposals.len()]
    }

    fn accept_trades_as_non_lead(&mut self, _game_state: &GameState, _trade: &Trade) -> bool {
        false
    }
}

fn create() -> Box<dyn PlayerStrategy> {
    Box::new(PlayerNoTrades {})
}

#[ctor]
fn init() {
    player::register_strategy(&"PlayerNoTrades", create)
}
