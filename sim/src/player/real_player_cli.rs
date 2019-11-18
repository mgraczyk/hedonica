use ctor::ctor;
use dialoguer::{Checkboxes, Confirmation};

use crate::game::GameState;
use crate::player;
use crate::player::*;
use crate::types::GoodsSet;

fn print_table_state(my_id: PlayerId, game_state: &GameState) {
    // TODO: Show my point values.
    println!(
        "\nHere's the table right now ({}, {}):",
        game_state.current_turn, game_state.current_round
    );

    for (i, player) in game_state.players.iter().enumerate() {
        println!(
            "Player {} {}: {}",
            i,
            if i == game_state.lead {
                "[lead]"
            } else if i == my_id {
                "[ you]"
            } else {
                "      "
            },
            serde_json::to_string_pretty(&player.num_goods).unwrap(),
        );
        println!("");
    }

    println!("");
}

fn ask_yes_no_question(prompt: &str) -> bool {
    Confirmation::new().with_text(prompt).interact().unwrap()
}

fn ask_goods_list(prompt: &str, goods: &GoodsSet) -> GoodsSet {
    let mut dialog = Checkboxes::new();
    dialog.with_prompt(prompt);

    let mut prompt_items = Vec::<&str>::new();
    goods.iter().for_each(|(category, &count)| {
        for _ in 0..(count as u64) {
            dialog.item(category);
            prompt_items.push(category);
        }
    });

    let mut result = GoodsSet::new();
    if let Ok(selected) = dialog.interact() {
        selected.into_iter().for_each(|i| {
            *result.entry(prompt_items[i].to_string()).or_insert(0.0) += 1.0;
        })
    }

    result
}

struct RealPlayerCLI {
    my_id: PlayerId,
}

impl PlayerStrategy for RealPlayerCLI {
    fn init(&mut self, player_id: PlayerId, _value: &serde_json::Value) {
        self.my_id = player_id;
    }

    fn reset(&mut self) {
        self.my_id = 0;
    }

    fn propose_trades_as_lead(&mut self, _game_state: &GameState) -> HashMap<PlayerId, Trade> {
        HashMap::new()
    }

    fn propose_trade_as_non_lead(&mut self, game_state: &GameState) -> Option<Trade> {
        print_table_state(self.my_id, game_state);

        if !ask_yes_no_question(&format!(
            "Do you want to trade with player {}?",
            game_state.lead
        )) {
            return None;
        }

        let from_acceptor = ask_goods_list(
            "Which goods do you want?",
            &game_state.lead_player_state().num_goods,
        );
        let from_proposor = ask_goods_list(
            "Which goods will you give?",
            &game_state.player_state(self.my_id).num_goods,
        );

        if from_acceptor.len() == 0 && from_proposor.len() == 0 {
            return None;
        }

        Some(Trade {
            proposer: self.my_id,
            accepter: game_state.lead,
            from_proposor: from_proposor,
            from_acceptor: from_acceptor,
        })
    }

    fn accept_trades_as_lead(&mut self, _game_state: &GameState) -> Vec<bool> {
        vec![false; _game_state.current_trade_proposals.len()]
    }

    fn accept_trades_as_non_lead(&mut self, _game_state: &GameState, _trade: &Trade) -> bool {
        ask_yes_no_question("Do you want to make the trade? [y/n]")
    }
}

fn create() -> Box<dyn PlayerStrategy> {
    Box::new(RealPlayerCLI { my_id: 0 })
}

#[ctor]
fn init() {
    player::register_strategy(&"RealPlayerCLI", create)
}
