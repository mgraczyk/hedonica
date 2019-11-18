// Rules of the game.
// R1. Players draw preferences cards which give them utility functions,
//     a mapping between "goods" and points.
// R2. Players receive an initial cash payment.
// R3. Players take turns in a pre-defined order until some player earns enough points to win.
// R4. On his turn, a player draws a good and trades with as many other players as he would like.
//     When he is done trading, his turn ends and the next player's turn begins.
//
// Variables in the game.
// victory_threshold - The number of points needed to win the game.
// start_money - The amount of money that players start with.
// { categories } - The set of categories of goods.
// { preferences } - The set of preferences that a player can be given.
//                   Each preference is a map from category to point value.
// { goods } - The set of goods a player can draw from the deck.
//             Each good has a category.
//
// Goals of simulation:
//  G0. The game should be fun to play and easy to learn.
//  G1. Ensure that the total game time is reasonable and has low variance.
//  G2. Ensure that the distribution of preferences is "fair", in that
//      the subset of preferences selected by the players do not typically
//      conspire to advantage any one player by too much.
//  G3. Ensure that there are no lame strategies that are easy to execute
//      and greatly outperform many other simple strategies.
//  G4. Ensure that there are no "dominant" strategies. We do not want every player
//      to be forced to execute the same strategy.
//
// We aim to find values of the variables that will achieve these goals, and to find convincing
// evidence that the goals have been achieved.
//
// Glossary:
//   Lead: The player whose turn it currently is.
//   Deck: The set of goods that has not yet been taken by any player.
//         In the board game, this is a deck of cards.
//
//
// This first simulator simplifies the game by making all actions synchronous,
// and by restricting the structure of trading.
// Trading is modeled as proposed trades that can be either accepted or rejected.
//
// A player must always be able to fulfill all his outstanding proposals, and no player
// can accept any proposals that he cannot fulfill.
//
// Proposals must be accepted or rejected when they are received.
// The flow is like this.
//
//  T1. The lead creates a set of proposed trades and broadcasts them.
//  T2. Each non-lead accepts or rejects any trades directed at him.
//  T3. Each non-lead prepares a set of trade proposal, which are gathered and broadcast to all
//      playes
use crate::non_nan::NonNan;
use crate::player;

use crate::player::*;
use crate::types::*;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{thread, time};

#[derive(Serialize, Deserialize, Clone)]
pub struct PlayerState {
    preferences: Preferences,
    pub num_goods: GoodsSet,
}

impl PlayerState {
    fn score(&self) -> f64 {
        self.num_goods
            .iter()
            .map(|(category, count)| count * self.preferences[category])
            .sum()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GameState {
    deck: Vec<Good>,

    pub players: Vec<PlayerState>,

    // It is this player's turn.
    pub lead: PlayerId,

    // Starts at 0, increments each time the lead changes.
    pub current_turn: i32,

    // Starts at 0, increments each time trades are proposed.
    // The lead proposes trades on even rounds.
    pub current_round: i32,
    pub current_trade_proposals: HashMap<PlayerId, Trade>,

    current_trades: Vec<Trade>,
    past_trades: HashMap<i32, Vec<Trade>>,
}

//fn diff_vector<T>(before: Vec<T>, after: Vec<T>) {
//for i in 0..min(before.len(), after.len()) {
//}
//}

//fn diff_game_state(before: &GameState, after: &GameState) -> HashMap<String, String> {
//let mut result = HashMap::new();
//let insert_if = |k, v| {
//if v { result.insert(k, v) };
//};

//insert_if("diff", diff_vector(before.deck, after.deck));

//return result
//}

impl GameState {
    pub fn lead_player_state(&self) -> &PlayerState {
        &self.players[self.lead]
    }

    pub fn player_state(&self, player_id: PlayerId) -> &PlayerState {
        &self.players[player_id]
    }

    fn start_lead_turn(&mut self) {
        *self.players[self.lead]
            .num_goods
            .get_mut(&self.deck.pop().unwrap().category)
            .unwrap() += 1.;
    }

    fn end_lead_turn(&mut self) {
        self.lead = (self.lead + 1) % self.players.len();
        assert_eq!(self.current_trade_proposals.len(), 0);
        if self.current_trades.len() > 0 {
            self.past_trades.insert(
                self.current_turn,
                std::mem::replace(&mut self.current_trades, Vec::new()),
            );
        }

        self.current_turn += 1;
        self.current_round = 0;
    }

    fn end_round(&mut self, trade_acceptances: Vec<bool>) {
        // Move goods for accepted trades.
        let players = &mut self.players;
        let accepted_trades = trade_acceptances
            .into_iter()
            .zip(std::mem::replace(
                &mut self.current_trade_proposals,
                HashMap::new(),
            ))
            .filter(|(accepted, (_, ___))| *accepted)
            .map(|(_, (__, trade))| {
                trade.from_proposor.iter().for_each(|(category, &amount)| {
                    if amount > 0.0 {
                        assert!(players[trade.proposer].num_goods[category] > amount);
                    } else {
                        assert!(players[trade.accepter].num_goods[category] > -amount);
                    }
                    *players[trade.proposer].num_goods.get_mut(category).unwrap() -= amount;
                    *players[trade.accepter].num_goods.get_mut(category).unwrap() += amount;
                });
                trade
            });

        self.current_trades.extend(accepted_trades);
        self.current_round += 1;
    }
}

#[derive(Serialize, Deserialize)]
pub struct GameRules {
    #[serde(default = "default_victory_threshold")]
    victory_threshold: f64,
    #[serde(default = "default_start_money")]
    start_money: f64,
    #[serde(default = "default_deck_size")]
    deck_size: usize,
    #[serde(default = "default_max_turns")]
    max_turns: i32,
}

fn default_victory_threshold() -> f64 {
    50.
}
fn default_start_money() -> f64 {
    10.
}
fn default_deck_size() -> usize {
    500
}
fn default_max_turns() -> i32 {
    1000
}

#[derive(Serialize, Deserialize)]
pub struct GameResult {
    pub turns: i32,
    pub winner: PlayerId,
    pub scores: Vec<f64>,
}

impl GameResult {
    fn from_state(game: GameState) -> GameResult {
        let scores: Vec<f64> = game.players.iter().map(PlayerState::score).collect();
        let winner = (0..game.players.len())
            .max_by_key(|pi| NonNan::new(scores[*pi]).unwrap())
            .unwrap();
        GameResult {
            winner,
            scores,
            turns: game.current_turn,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SimConfig {
    #[serde(default)]
    pub deck_shuffle_seed: u64,

    #[serde(default = "default_preferences_seed")]
    pub preferences_seed: u64,

    #[serde(default = "default_num_players")]
    pub num_players: usize,

    #[serde(default = "default_num_runs")]
    pub num_runs: i32,

    #[serde(default)]
    pub player_configs: Vec<PlayerConfig>,

    #[serde(default = "default_turn_pause_millis")]
    pub turn_pause_millis: u64,

    #[serde(default)]
    pub hide_game_state: bool,
}

fn default_preferences_seed() -> u64 {
    1
}
fn default_num_players() -> usize {
    2
}
fn default_num_runs() -> i32 {
    100
}
fn default_turn_pause_millis() -> u64 {
    500
}

const CATEGORIES: &[&str] = &["money", "cars", "clothing", "food", "art", "travel"];

fn generate_deck(config: &SimConfig, rules: &GameRules) -> Vec<Good> {
    let mut rng: StdRng = match config.deck_shuffle_seed {
        0 => SeedableRng::from_rng(rand::thread_rng()).unwrap(),
        _ => SeedableRng::seed_from_u64(config.deck_shuffle_seed),
    };

    let mut result: Vec<Good> = CATEGORIES[1..]
        .iter()
        .map(|category| {
            (0..((rules.deck_size / CATEGORIES[1..].len()) as usize)).map(move |_: usize| Good {
                category: String::from(*category),
            })
        })
        .flatten()
        .collect();
    result.shuffle(&mut rng);
    result
}

fn generate_players(
    config: &SimConfig,
    rules: &GameRules,
    mut preferences_deck: Vec<Preferences>,
) -> Vec<PlayerState> {
    // TODO(mgraczyk): Correct for advantage in going first.
    //                 This doesn't quite work.
    //                 With two players, we have to give p1 $2 extra.
    //                 With more, it becomes hard to give integer numbers.
    const OFFSET: [f64; 11] = [0., 2., 0., 0., 0., 0., 1., 1., 1., 1., 1.];

    (0..config.num_players)
        .map(|player_num| {
            let preferences = preferences_deck.pop().unwrap();
            let mut num_goods: HashMap<String, f64> = preferences
                .iter()
                .map(|(category, _)| (category.clone(), 0.))
                .collect();
            num_goods.insert(
                String::from("money"),
                rules.start_money + OFFSET[player_num] * (player_num as f64),
            );
            PlayerState {
                preferences,
                num_goods,
            }
        })
        .collect()
}

fn generate_preferences_deck(config: &SimConfig) -> Vec<Preferences> {
    let mut rng: StdRng = match config.preferences_seed {
        0 => SeedableRng::from_rng(rand::thread_rng()).unwrap(),
        _ => SeedableRng::seed_from_u64(config.preferences_seed),
    };

    let mut result = Vec::new();
    let mut values = [1, 2, 2, 5, 10];

    for _ in 0..config.num_players {
        result.push({
            values.shuffle(&mut rng);

            let mut map = Preferences::new();
            map.insert(String::from("money"), 1.);
            CATEGORIES[1..]
                .iter()
                .zip(values.iter())
                .for_each(|(category, &v)| {
                    map.insert(String::from(*category), v as f64);
                    return;
                });
            map
        });
    }
    result
}

pub fn generate_start_state(config: &SimConfig, rules: &GameRules) -> GameState {
    let preferences_deck = generate_preferences_deck(config);

    GameState {
        players: generate_players(config, rules, preferences_deck),
        deck: generate_deck(config, rules),
        lead: 0,
        current_turn: 0,
        current_round: 0,
        current_trade_proposals: HashMap::new(),
        current_trades: Vec::new(),
        past_trades: HashMap::new(),
    }
}

pub fn play(
    config: &SimConfig,
    rules: &GameRules,
    mut game: GameState,
    players: &mut Vec<Box<dyn player::PlayerStrategy>>,
) -> GameResult {
    'turns: while game.current_turn < rules.max_turns && game.deck.len() > 0 {
        game.start_lead_turn();
        'rounds: loop {
            if config.turn_pause_millis > 0 {
                thread::sleep(time::Duration::from_millis(config.turn_pause_millis));
            }

            if !config.hide_game_state {
                println!("{}", serde_json::to_string_pretty(&game).unwrap());
            }
            if game.lead_player_state().score() >= rules.victory_threshold {
                break 'turns;
            }

            game.current_trade_proposals = if game.current_round % 2 == 0 {
                players[game.lead].propose_trades_as_lead(&game)
            } else {
                let mut trades = HashMap::new();
                for (player_id, player) in players.iter_mut().enumerate() {
                    if player_id == game.lead {
                        continue;
                    }
                    if let Some(trade) = player.propose_trade_as_non_lead(&game) {
                        trades.insert(player_id, trade);
                    }
                }
                trades
            };

            if game.current_round > 0
                && game.current_round % 2 == 0
                && game.current_trade_proposals.len() == 0
            {
                break 'rounds;
            }

            let trade_acceptances = if game.current_round % 2 == 0 {
                game.current_trade_proposals
                    .iter()
                    .map(|(&player_id, trade)| {
                        players[player_id].accept_trades_as_non_lead(&game, trade)
                    })
                    .filter(|&do_trade| do_trade)
                    .collect()
            } else {
                players[game.lead].accept_trades_as_lead(&game)
            };

            game.end_round(trade_acceptances);
        }
        game.end_lead_turn();
    }

    GameResult::from_state(game)
}
