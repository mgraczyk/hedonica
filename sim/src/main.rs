extern crate clap;

mod game;
mod non_nan;
mod player;
mod stats;
mod types;

use crate::game::*;
use crate::player::*;
use clap::{App, Arg};
use json5;
use std::collections::BTreeMap;

fn run_sim(config: SimConfig, rules: GameRules) {
    let mut game_results: Vec<GameResult> = Vec::new();
    let mut players: Vec<Box<dyn PlayerStrategy>> =
        load_strategies(&config.player_configs, config.num_players);

    for _ in 0..config.num_runs {
        let game = game::generate_start_state(&config, &rules);
        players.iter_mut().for_each(|player| player.reset());

        let game_result = game::play(&config, &rules, game, &mut players);
        game_results.push(game_result);
    }

    let mut wins_by_player: BTreeMap<usize, i32> = BTreeMap::new();
    game_results
        .iter()
        .for_each(|g| *wins_by_player.entry(g.winner).or_insert(0) += 1);
    println!("{}", serde_json::to_string_pretty(&wins_by_player).unwrap());

    let turn_stats: stats::Stats = game_results.iter().map(|g| g.turns as f64).collect();
    println!(
        "{}",
        serde_json::to_string_pretty(&turn_stats).unwrap()
    );
    println!("\n");
}

fn main() {
    let default_sim_config =
        serde_json::to_string_pretty(&json5::from_str::<SimConfig>("{}").unwrap()).unwrap();
    let default_game_rules =
        serde_json::to_string_pretty(&json5::from_str::<GameRules>("{}").unwrap()).unwrap();

    let matches = App::new("Hedonica Simulator")
        .version("0.1")
        .author("Michael Graczyk <michael@mgraczyk.com>")
        .about("Simulates the Hedonica board game")
        .arg(
            Arg::with_name("sim-config")
                .long("sim-config")
                .help("JSON of sim config")
                .default_value(&default_sim_config)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("game-rules")
                .long("game-rules")
                .help("JSON of game rules")
                .default_value(&default_game_rules)
                .takes_value(true),
        )
        .get_matches();

    let config: SimConfig = json5::from_str(matches.value_of("sim-config").unwrap()).expect("Could not parse sim config");
    let rules: GameRules = json5::from_str(matches.value_of("game-rules").unwrap()).expect("Could not parse game rules");
    run_sim(config, rules);
}
