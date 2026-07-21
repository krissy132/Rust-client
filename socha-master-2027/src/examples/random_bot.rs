//! Random-bot example for `socha-master-2027`.
//!
//! Joins a server at `localhost:13050`, plays random legal moves each turn, and exits when the
//! game ends. Run with:
//!
//! ```sh
//! cargo run --example random_bot
//! ```

use log::LevelFilter;
use rand::Rng;
use socha::i_client_handler::handler_trait::IClientHandler;
use socha::i_client_handler::start_iclient;
use socha::internal::GameState;
use socha::neutral::{Move, Team};

#[derive(Debug, Default)]
pub struct RandomBot {
    game_state: GameState,
}

impl IClientHandler for RandomBot {
    fn calculate_move(&mut self) -> Move {
        let mut rng = rand::rng();
        let moves = socha::internal::sensible_moves(&self.game_state);
        if moves.is_empty() {
            // Should never happen because `sensible_moves` always returns at least a Skip,
            // but be defensive — pick a Skip of the current color.
            return Move::Skip {
                color: self.game_state.current_color(),
            };
        }
        moves[rng.random_range(0..moves.len())]
    }

    fn on_gamestate_update(&mut self, state: GameState) {
        self.game_state = state;
    }

    fn on_welcome_message(&mut self, team: Team) {
        println!("welcome — playing as team {team}");
    }
}

fn main() -> Result<(), socha::error::ComError> {
    // Optional log file.
    let _ = simple_logging::log_to_file("com.log", LevelFilter::Info);
    let mut handler = RandomBot::default();
    start_iclient(
        "localhost:13050",
        None,
        &mut handler,
        std::time::Duration::from_millis(2),
        std::time::Duration::from_secs_f64(1.0),
    )?;
    Ok(())
}
