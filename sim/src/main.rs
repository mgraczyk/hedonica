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
// This first simulator simplifies the game by making all actions synchronous.
// Each turn is modeled as a sequence of messages passed to and from the lead.
mod non_nan;
mod stats;

use json5;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
//use rand::distributions::{Distribution, Uniform};

use non_nan::NonNan;
use rand::prelude::*;

type Preferences = BTreeMap<String, f64>;

#[derive(Serialize, Deserialize)]
pub struct Good {
    category: String,
}

#[derive(Serialize, Deserialize)]
pub struct Trade {
    proposor: i32,
    acceptor: i32,
    from_proposor: Vec<Good>,
    to_acceptor: Vec<Good>,
}

#[derive(Serialize, Deserialize)]
pub struct PlayerState {
    preferences: Preferences,
    pub num_goods: BTreeMap<String, f64>,
}

impl PlayerState {
    pub fn score(&self) -> f64 {
        self.num_goods
            .iter()
            .map(|(category, count)| count * self.preferences[category])
            .sum()
    }
}

#[derive(Serialize, Deserialize)]
pub struct SharedState {
    deck: Vec<Good>,
}

#[derive(Serialize, Deserialize)]
pub struct GameState {
    shared_state: SharedState,

    players: Vec<PlayerState>,

    // It is this player's turn.
    lead: usize,

    // Starts at 0, increments each time the lead changes.
    pub current_turn: i32,

    current_trades: Vec<Trade>,
    past_trades: Vec<Vec<Trade>>,
}

impl GameState {
    fn lead_player(&self) -> &PlayerState {
        &self.players[self.lead]
    }

    fn start_lead_turn(&mut self) {
        *self.players[self.lead]
            .num_goods
            .get_mut(&self.shared_state.deck.pop().unwrap().category)
            .unwrap() += 1.;
    }

    fn end_lead_turn(&mut self) {
        self.lead = (self.lead + 1) % self.players.len();
        self.current_turn += 1;
        self.past_trades
            .push(std::mem::replace(&mut self.current_trades, Vec::new()));
    }
}

#[derive(Serialize, Deserialize)]
pub struct GameRules {
    max_turns: i32,
    victory_threshold: f64,
    start_money: f64,
    deck_size: usize,
}

pub trait PlayerStrategy {}

#[derive(Serialize, Deserialize)]
pub struct SimConfig {
    deck_shuffle_seed: u64,
    preferences_seed: u64,
    num_players: usize,
    num_runs: i32,
}

#[derive(Serialize, Deserialize)]
pub struct GameResult {
    turns: i32,
    winner: usize,
    scores: Vec<f64>,
}

impl GameResult {
    pub fn from_state(game: GameState) -> GameResult {
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
            let mut num_goods: BTreeMap<String, f64> = preferences
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
                .for_each(|(category, v)| {
                    map.insert(String::from(*category), *v as f64);
                    return;
                });
            map
        });
    }
    result
}

fn generate_start_state(config: &SimConfig, rules: &GameRules) -> GameState {
    let preferences_deck = generate_preferences_deck(config);

    GameState {
        players: generate_players(config, rules, preferences_deck),
        shared_state: SharedState {
            deck: generate_deck(config, rules),
        },
        lead: 0,
        current_turn: 0,
        current_trades: Vec::new(),
        past_trades: Vec::new(),
    }
}

const GAME_RULES: &str = r#"
    {
        "victory_threshold": 50,
        "max_turns": 1000,
        "start_money": 10,
        "deck_size": 500,
    }"#;

const SIM_CONFIG: &str = r#"
    {
        "deck_shuffle_seed": 0,
        "preferences_seed": 43222,
        "num_runs": 16000,
        "num_players": 2,
    }"#;

fn play(mut game: GameState, rules: &GameRules) -> GameResult {
    assert!(game.players.len() > 1);

    'turns: while game.current_turn < rules.max_turns {
        game.start_lead_turn();
        'trades: loop {
            if game.lead_player().score() >= rules.victory_threshold {
                break 'turns;
            }
            break 'trades;
        }
        game.end_lead_turn();
    }

    GameResult::from_state(game)
}

fn run_sim(rules: GameRules, config: SimConfig) {
    let mut game_results: Vec<GameResult> = Vec::new();

    for run in 0..config.num_runs {
        let game = generate_start_state(&config, &rules);
        if run == config.num_runs - 1 {
            println!("{}", serde_json::to_string_pretty(&game.players).unwrap());
        }

        let game_result = play(game, &rules);
        game_results.push(game_result);
    }

    let mut wins_by_player: BTreeMap<usize, i32> = BTreeMap::new();
    game_results
        .iter()
        .for_each(|g| *wins_by_player.entry(g.winner).or_insert(0) += 1);
    println!("{}", serde_json::to_string_pretty(&wins_by_player).unwrap());

    let turn_stats: stats::Stats = game_results.iter().map(|g| g.turns as f64).collect();
    println!(
        "turn stats\n{}",
        serde_json::to_string_pretty(&turn_stats).unwrap()
    );
}

fn main() {
    let config: SimConfig = json5::from_str(SIM_CONFIG).unwrap();
    let rules: GameRules = json5::from_str(GAME_RULES).unwrap();
    run_sim(rules, config);
}
