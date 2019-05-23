mod game;
mod non_nan;
mod player;
mod stats;
mod types;

use crate::game::*;
use crate::player::*;
use json5;
use std::collections::BTreeMap;
use std::env;


fn run_sim(config: SimConfig, rules: GameRules) {
    let mut game_results: Vec<GameResult> = Vec::new();
    let mut players: Vec<Box<player::PlayerStrategy>> =
        load_strategies(&config.player_configs, config.num_players);

    for _ in 0..config.num_runs {
        let game = game::generate_start_state(&config, &rules);
        players.iter_mut().for_each(|player| player.reset());

        let game_result = game::play(game, &rules, &mut players);
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
    let args: Vec<String> = env::args().collect();
    let config_arg = if args.len() > 1 { &args[1] } else { "{}" };
    let rules_arg = if args.len() > 2 { &args[2] } else { "{}" };

    let config: SimConfig = json5::from_str(config_arg).unwrap();
    let rules: GameRules = json5::from_str(rules_arg).unwrap();
    run_sim(config, rules);
}
