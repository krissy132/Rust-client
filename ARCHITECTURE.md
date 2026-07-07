# Architecture Overview

## Project layout
```
socha-master-2027/
├─ Cargo.toml                # crate metadata & dependencies
├─ src/
│  ├─ lib.rs               # re‑exports public modules
│  ├─ error.rs             # communication‑layer error types
│  ├─ i_client_handler/
│  │   ├─ mod.rs           # public API – `start_iclient` & command enums
│  │   └─ handler_trait.rs # `IClientHandler` trait – entry point for bots
│  ├─ incoming.rs          # XML deserialization of every server message
│  ├─ outgoing.rs          # XML builders for *join*, *move* and admin commands
│  ├─ internal.rs          # core game model, board, validation, move generation & scoring
│  ├─ neutral.rs           # domain enums (Color, Team, PieceShape, …) and helpers
│  └─ socha_com.rs         # low‑level TCP handling, buffering, message framing
├─ examples/
│  └─ random_bot.rs        # tiny reference bot (random legal moves)
└─ tests/
   ├─ make_unmake_move.rs   # unit tests for board‑state changes
   └─ server_message_parsing.rs # parsing tests for inbound XML
```

## Core modules
| Module | Responsibility |
|--------|-----------------|
| `error.rs` | Defines `ComError`, `SendErr`, `ReceiveErr` and finer‑grained enums – the public error surface of the library. |
| `i_client_handler` | Provides the **bot contract** (`IClientHandler`) and the orchestration helper `start_iclient`. It also defines the `SendCommnad` / `SendAdminCommand` enums that the runtime uses to forward moves to the server. |
| `incoming.rs` | Strong‑XML structs that mirror every XML element the server can send (`<room>`, `<joined>`, `<left>`, `<data class="…">`). The structs are annotated for automatic deserialization. |
| `outgoing.rs` | Helper functions that serialise the same protocol back to the server (`make_join_xml`, `make_move_xml`, admin XML builders). Most structs are also `#[derive(XmlRead, XmlWrite)]`. |
| `internal.rs` | **Game engine** – immutable data model (`Board`, `GameState`), move validation (`validate_move`), move generation (`possible_moves`, `sensible_moves`) and scoring (`points_for_color`). This is a faithful Rust port of the original Kotlin sources. |
| `neutral.rs` | Domain‑specific enums (`Color`, `Team`, `PieceShape`, `Rotation`, `Move`, `BlokusMoveMistake`, `ScoreAggregation`) and lightweight geometry helpers (`Coordinates`). |
| `socha_com.rs` | Low‑level TCP client (`ComHandler`). It performs non‑blocking reads, buffers partial XML, extracts `<comMessage>` elements and converts them into the high‑level `ComMessage` enum from `internal.rs`. |

## Communication flow
1. **Connect** – `ComHandler::join` builds a `<join>` (or `<joinPrepared>`) XML, writes `<protocol>` then the join XML to the server.
2. **Receive** – The handler continuously reads from the TCP stream, appends to an internal string buffer and repeatedly attempts to parse a complete `<comMessage>...</comMessage>` using `strong_xml`. Successful parses are turned into `internal::ComMessage` variants (`Joined`, `Left`, `Room(..)`, `Admin(..)`).
3. **Dispatch** – `i_client_handler::start_iclient` spawns a reader thread that feeds the parsed messages into three cross‑beam channels (raw, watch, outgoing). The main loop consumes the channel, maps the high‑level `RoomMessage` into trait callbacks (`on_game_joined`, `on_welcome_message`, `on_gamestate_update`, `on_game_result`). When a `MoveRequest` arrives it calls `IClientHandler::calculate_move`, wraps the result in `SendCommnad::Move` and sends it back through the outgoing channel.
4. **Admin** – The same channel can be used to send admin commands (`Authenticate`, `Observe`, `Pause`, …). The runtime currently wires only `SendCommnad::Move`; admin commands can be emitted manually by writing to the channel.

## Build & test
```sh
# Build the library (and the example binary)
cargo build

# Run the reference bot against a local server
cargo run --example random_bot

# Run the unit‑test suite
cargo test
```

The crate has no external runtime dependencies other than the standard library, `strong-xml` (XML (de)serialization), `crossbeam-channel` (thread communication), `log`/`simple‑logging` (optional logging) and `rand` (used only by the example).

---
*The architecture mirrors the original Kotlin client while exposing a small, idiomatic Rust API for bot developers.*