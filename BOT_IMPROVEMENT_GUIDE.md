# Guide to Building a Better 2027 Blokus Bot

This repository contains a fully‑functional client library (`socha`) that hides all networking and XML details.  All you need is a struct that implements the `IClientHandler` trait.  The following steps walk you through a typical development workflow.

---
## 1. Prerequisites
- **Rust toolchain** – install via `rustup` (latest stable).  The code is written for the 2021 edition.
- **Cargo** – the standard Rust package manager (`cargo --version`).
- **A running server** – the official Software‑Challenge 2027 server listens on `HOST:PORT` (default in the example is `localhost:13050`).  You can also use a local mock server for testing.

---
## 2. Compile & run the reference bot
```sh
# Build the library and the example binary
cargo run --example random_bot
```
The bot joins a free room, picks a random legal move each turn and exits when the game ends.  Study the source at `src/examples/random_bot.rs` – it is the minimal starting point.

---
## 3. Create your own bot
1. **Create a new binary (or example)** inside the crate:
   ```sh
   cargo new --bin my_bot
   # move the generated folder under the repository or add it to Cargo.toml as a workspace member
   ```
   In `my_bot/src/main.rs` add the `socha` dependency (the crate is part of the same workspace, so you can write `use socha::*`).
2. **Define a state holder** – typically you store the current `GameState` plus any auxiliary data (search tree, heuristics, cache, …):
   ```rust
   use socha::internal::GameState;
   use socha::neutral::{Move, Team};

   struct MyBot {
       state: GameState,
       // add your own fields here (e.g. a Monte‑Carlo tree, opening book, …)
   }
   ```
3. **Implement the `IClientHandler` trait** (the only required methods are `calculate_move` and `on_gamestate_update`):
   ```rust
   use socha::i_client_handler::handler_trait::IClientHandler;

   impl IClientHandler for MyBot {
       fn calculate_move(&mut self) -> Move {
           // *** YOUR MOVE LOGIC ***
           // The current board is available in `self.state`.
           // A simple fallback is `internal::sensible_moves(&self.state)[0]`.
           // Return either `Move::Set { piece }` or `Move::Skip { color }`.
           unimplemented!();
       }

       fn on_gamestate_update(&mut self, state: GameState) {
           // keep the latest state – required for `calculate_move`
           self.state = state;
       }

       fn on_welcome_message(&mut self, team: Team) {
           println!("I am playing for team {team}");
       }
   }
   ```
4. **Start the client** – the same helper used by the example:
   ```rust
   use socha::i_client_handler::start_iclient;
   use std::time::Duration;

   fn main() -> Result<(), socha::error::ComError> {
       // optional file logging (writes the raw XML traffic)
       let _ = simple_logging::log_to_file("com.log", log::LevelFilter::Info);

       let mut bot = MyBot { state: GameState::default() };
       start_iclient(
           "localhost:13050", // address of the server
           None,                // reservation code for prepared rooms (optional)
           &mut bot,
           Duration::from_millis(2), // sleep time for the I/O thread
           Duration::from_secs_f64(1.0), // timeout for a move request
       )?;
       Ok(())
   }
   ```

---
## 4. Using the engine helpers
`socha::internal` offers a rich toolbox:
- **Move generation** – `possible_moves(&state)` (all legal moves), `sensible_moves(&state)` (adds a mandatory `Skip` when no move exists).
- **Validation** – `validate_move(&state, &mv)` returns a `Result<(), BlokusMoveMistake>`; useful while debugging your algorithm.
- **Scoring** – `state.points_for_color(color)` and `state.points_for_team(team)` give the final score components.
- **Board utilities** – `Board::valid_fields(color)`, `Board::colored_fields(color)`, `Board::borders_on_color`, `Board::corners_on_color`.

You can build sophisticated heuristics (e.g., minimise the opponent's available fields, maximise own colour clusters, favour early placement of large pentominos, etc.) by inspecting `state.board` directly.

---
## 5. Admin commands (optional)
If you need to run an *admin* client (create/observe rooms, pause, step, cancel) the library already contains XML builders in `outgoing.rs` and a thin wrapper in `socha_com::ComHandler`.  Example usage:
```rust
use socha::socha_com::ComHandler;
let mut com = ComHandler::connect_to_server("localhost:13050")?; // no join message
com.send_admin_authenticate("my_secret")?;
com.send_admin_prepare(false, &[
    socha::socha_com::PrepareSlot::new("slot‑A".into(), true, false),
    socha::socha_com::PrepareSlot::new("slot‑B".into(), false, true),
])?;
```
The normal client (`start_iclient`) does not expose these admin calls directly; you can extend the `SendAdminCommand` enum in `i_client_handler/mod.rs` and forward the corresponding XML through the `out_tx` channel.

---
## 6. Debugging & logging
- **Raw traffic** – `simple_logging::log_to_file("com.log", LevelFilter::Info)` writes every inbound/outbound XML line to `com.log`.  Inspect it with `cat com.log`.
- **Move validation** – wrap your move generation with `validate_move` in a `debug_assert!` block to catch illegal moves early.
- **Unit tests** – the repository ships a comprehensive test suite (`cargo test`).  Add new tests for your algorithm to keep it regression‑safe.

---
## 7. Common workflow (CLI commands)
| Goal | Command |
|------|----------|
| Build the library | `cargo build --release` |
| Run the reference bot | `cargo run --example random_bot` |
| Run your bot (binary `my_bot`) | `cargo run --bin my_bot` |
| Execute all tests | `cargo test` |
| Check formatting | `cargo fmt -- --check` |
| Lint (Clippy) | `cargo clippy -- -D warnings` |

---
## 8. Tips for a “better” bot
1. **Search depth** – generate all moves (`possible_moves`) and evaluate the resulting `GameState` with a static evaluation (e.g., remaining area, number of pieces left, corner‑reachability).
2. **Monte‑Carlo Tree Search (MCTS)** – treat each legal move as a child node, rollout random moves until the end of the game, back‑propagate the result.  The library already provides fast move generation and scoring, which makes MCTS feasible.
3. **Opening book** – pre‑compute a small set of high‑quality opening placements for each colour and start the game with the best one.
4. **Parallel evaluation** – the move list can be split across threads (Rayon works nicely) because `GameState` is `Clone` and all helper functions are pure.
5. **Avoid early skips** – `validate_skip_move` forbids skipping on the first turn, so your algorithm should always try a real piece on the first move.
6. **Mono‑bonus** – placing the `Mono` piece as the *very last* piece of a colour yields a +5 bonus (`state.last_move_mono`).  Planning for that can swing the final score.

---
## 9. Packaging & distribution
If you want to ship your bot as a stand‑alone binary, add it as a binary target in `Cargo.toml`:
```toml
[[bin]]
name = "my_bot"
path = "src/main.rs"
```
Then `cargo build --release` will produce a static executable in `target/release/` ready to be uploaded to the competition server.

---
**Happy coding – may your moves be legal and your scores high!**