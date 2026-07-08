//! Random-bot example for `socha-master-2027`.
//! 
//! Joins a server at `localhost:13050`, plays random legal moves each turn, and exits when 
//! the game ends. Run with:
//! 
//! ```sh
//! cargo run --example bot1
//! ```
//! 
//! This version uses a comprehensive set of socha API functions and internal access
//! to inspect game state and identify potential bugs in move generation, validation,
//! and game result handling.

use log::LevelFilter;
use rand::Rng;
use socha::i_client_handler::handler_trait::IClientHandler;
use socha::i_client_handler::start_iclient;
use socha::internal::GameState;
use socha::neutral::{Color, Move, Team};

#[derive(Debug, Default)]
pub struct RandomBot {
    game_state: GameState,
}

impl IClientHandler for RandomBot {
    fn calculate_move(&mut self) -> Move {
        self.debug_game_state();
        let mut rng = rand::rng();
        let moves = socha::internal::possible_moves(&self.game_state);
        self.log_move_statistics(&moves);
        if moves.is_empty() {
            return Move::Skip {
                color: self.game_state.current_color(),
            };
        }
        let selected = moves[rng.random_range(0..moves.len())];
        self.log_selected_move(&selected);
        selected
    }
    
    fn on_gamestate_update(&mut self, state: GameState) {
        self.game_state = state;
        // Debug: Log when game state updates
        self.log_state_update();
    }
    
    fn on_welcome_message(&mut self, team: Team) {
        println!("welcome — playing as team {team}");
        // Debug: Log team assignment
        self.log_welcome_message(team);
    }
}
impl RandomBot {
    /// Comprehensive game state debugging to identify potential bugs
    fn debug_game_state(&self) {
        let s = &self.game_state;
        println!("=== GAME STATE DEBUG ===");
        println!("Turn: {}, Round: {}", s.turn, s.round());
        println!("Current color: {:?}, Team: {:?}", s.current_color(), s.current_team());
        println!("Valid colors: {:?}", s.valid_colors);
        
        // Check is_over() vs game_result() consistency
        let is_state_over = s.is_over();
        let result = s.game_result();
        println!("is_over(): {}, game_result(): {:?}", is_state_over, result);
        
        if is_state_over && result.is_none() {
            println!("⚠️  POTENTIAL BUG: is_over() true but game_result() None!");
        }
        
        // Debug piece counts per color
        println!("Piece counts - Blue: {}, Yellow: {}, Red: {}, Green: {}", 
                 s.blue_shapes.len(), s.yellow_shapes.len(), 
                 s.red_shapes.len(), s.green_shapes.len());
        
        // Check for colors with empty undeployed list
        for &color in Color::ALL.iter() {
            let undeployed = s.undeployed(color);
            if undeployed.is_empty() {
                println!("⚠️  Color {:?} has no pieces left - should be removed from valid_colors", color);
            }
        }
        
        // Debug: Board occupancy
        let occupied = s.board.colored_fields(Color::Blue).len();
        println!("Blue pieces on board: {}", occupied);
        
        // Debug: Move generation statistics
        let possible = socha::internal::possible_moves(s);
        let sensible = socha::internal::sensible_moves(s);
        println!("Move generation - possible: {}, sensible: {}", possible.len(), sensible.len());
        
        if possible.len() != sensible.len() {
            println!("⚠️  WARNING: possible_moves != sensible_moves! Possible empty should cause Skip");
        }
        
        // Debug: Current color first move status
        let first_move = s.is_first_move_for(s.current_color());
        println!("Current color ({:?}) first move: {}", s.current_color(), first_move);
        
        println!("=== END DEBUG ===");
    }
    
    /// Log move statistics for analysis
    fn log_move_statistics(&self, moves: &[Move]) {
        let mut set_moves = 0;
        let mut skip_moves = 0;
        
        for mv in moves {
            match mv {
                Move::Set { .. } => set_moves += 1,
                Move::Skip { .. } => skip_moves += 1,
            }
        }
        
        if moves.len() > 10 {
            println!("Move stats: {} Set moves, {} Skip moves", set_moves, skip_moves);
        }
    }
    
    /// Log selected move for analysis
    fn log_selected_move(&self, mv: &Move) {
        match mv {
            Move::Set { piece } => {
                println!("Selected Set move: {:?} at {:?}", 
                         piece.kind, piece.position);
            }
            Move::Skip { color } => {
                println!("Selected Skip move for {:?}", color);
            }
        }
    }
    
    /// Log game state updates
    fn log_state_update(&self) {
        let s = &self.game_state;
        println!("[STATE UPDATE] Turn: {}, Round: {}, Current color: {:?}", 
                 s.turn, s.round(), s.current_color());
    }
    
    /// Log welcome message
    fn log_welcome_message(&self, team: Team) {
        println!("[WELCOME] Joined team: {:?}", team);
    }
}
fn main() -> Result<(), socha::error::ComError> {
    // Optional log file.
    let _ = simple_logging::log_to_file("bot1.log", LevelFilter::Info);
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
