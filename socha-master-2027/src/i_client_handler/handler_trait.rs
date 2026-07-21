use crate::i_client_handler::ComCancelHandler;
use crate::internal::{GameResult, GameState, PreparedRoom};
use crate::neutral::{Move, Team};

/// The callback interface the user has to implement to drive a 2027 Blokus client.
///
/// The default implementations cope with most events; the only required methods are
/// `calculate_move` and `on_gamestate_update`.
pub trait IClientHandler {
    /// Called when the server requests a move from this client.
    fn calculate_move(&mut self) -> Move;

    /// Called whenever the server sends a fresh `GameState`.
    ///
    /// Use this to save the state into your struct so `calculate_move` can use it.
    fn on_gamestate_update(&mut self, state: GameState);

    /// Called when the welcome message arrives — `team` is your assigned team (ONE/TWO).
    fn on_welcome_message(&mut self, team: Team);

    /// Called when the client has successfully joined a room.
    fn on_game_joined(&mut self, room_id: &str) {
        println!("joined game with id: {}", room_id);
    }

    /// Called when the client has left the room.
    fn on_game_left(&mut self) {
        println!("client is not in game room anymore");
    }

    /// Called when the result of the current game has been received.
    /// Default prints and exits — override to keep the process alive across games.
    fn on_game_result(&mut self, res: &GameResult) {
        println!("game over, exiting");
        println!("final result: \n{:#?}", res);
        std::process::exit(0);
    }

    /// Is run while the enemy is calculating their move. The handler should
    /// poll `cancel_handler.is_cancelled()` and return as soon as it returns `true`.
    #[allow(unused_variables)]
    fn while_waiting(&mut self, cancel_handler: ComCancelHandler) {}

    // ___ ADMIN ___
    /// Called when a game is prepared.
    #[allow(unused_variables)]
    fn on_game_prepared(&mut self, prepared: &PreparedRoom) {}

    /// ADMIN: when a game was created.
    fn on_create_game(&mut self) {}

    /// ADMIN: called when the client starts observing a room.
    #[allow(unused_variables)]
    fn on_observed(&mut self, room_id: &str) {}
}
