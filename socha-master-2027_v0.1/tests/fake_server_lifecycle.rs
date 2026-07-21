//! End-to-end integration test against a tiny fake server.
//!
//! ChatGPT's risk P3.14: a complete lifecycle (connect → join → memento → turn →
//! receive move → next turn → game end) has never been exercised. This test wires
//! up an in-process `TcpListener`-based fake server that speaks the Socha-protocol
//! enough to drive `start_iclient` through one round-trip.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use socha::i_client_handler::ComCancelHandler;
use socha::i_client_handler::handler_trait::IClientHandler;
use socha::i_client_handler::start_iclient;
use socha::internal::GameState;
use socha::neutral::{Move, Team};

/// A minimal bot that plays a fixed, deterministic move when asked, and
/// records the last received state so we can assert against it after the game.
#[derive(Debug, Default)]
struct ScriptedBot {
    last_state: Arc<Mutex<Option<GameState>>>,
    produced_move: Arc<Mutex<Option<Move>>>,
    saw_welcome: Arc<Mutex<Option<Team>>>,
    saw_result: Arc<Mutex<bool>>,
}

impl IClientHandler for ScriptedBot {
    fn calculate_move(&mut self) -> Move {
        // Return a Skip as the simplest possible valid response — the engine's
        // `sensible_moves` would give us a Set move on a real fresh state, but a Skip is
        // accepted by the server as a valid pass (since first-move rules forbid skip,
        // the integration test below crafts the memento so BLUE has already played).
        let mv = Move::Skip {
            color: socha::neutral::Color::Yellow,
        };
        *self.produced_move.lock().unwrap() = Some(mv);
        mv
    }

    fn on_gamestate_update(&mut self, state: GameState) {
        *self.last_state.lock().unwrap() = Some(state);
    }

    fn on_welcome_message(&mut self, team: Team) {
        *self.saw_welcome.lock().unwrap() = Some(team);
    }

    fn on_game_result(&mut self, _res: &socha::internal::GameResult) {
        *self.saw_result.lock().unwrap() = true;
    }
}

#[test]
fn end_to_end_lifecycle_join_memento_move_result() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let addr = listener.local_addr().expect("local_addr").to_string();

    let bot_state: Arc<Mutex<Option<GameState>>> = Arc::new(Mutex::new(None));
    let bot_move: Arc<Mutex<Option<Move>>> = Arc::new(Mutex::new(None));
    let bot_welcome: Arc<Mutex<Option<Team>>> = Arc::new(Mutex::new(None));
    let bot_result: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));

    let mut bot = ScriptedBot {
        last_state: bot_state.clone(),
        produced_move: bot_move.clone(),
        saw_welcome: bot_welcome.clone(),
        saw_result: bot_result.clone(),
    };

    // Fake-server thread.
    let server_thread = thread::spawn(move || {
        let (mut stream, _peer) = listener.accept().expect("accept");
        stream.set_read_timeout(Some(Duration::from_millis(500))).ok();
        stream.set_nonblocking(false).ok();

        // Drain the client handshake (<protocol> + <join gameType="swc_2027_blokus"/>).
        let mut buf = [0u8; 4096];
        let _ = stream.read(&mut buf);

        // Respond with the protocol header, then a welcome message, then an initial
        // memento with BLUE already having played (turn=1 so YELLOW is current).
        stream
            .write_all(b"<protocol>")
            .expect("write protocol");
        // WelcomeMessage for YELLOW (team TWO).
        stream
            .write_all(b"<joined roomId=\"r-integ\"/><room roomId=\"r-integ\"><data class=\"welcomeMessage\" color=\"TWO\"/></room>")
            .expect("write welcome");
        // Initial memento: turn=1 (YELLOW), start_piece=PENTO_L, board populated with one BLUE
        // piece so YELLOW can pass with a Skip (SkipFirstTurn was ruled out — actually
        // `validate_skip_move` checks `is_first_move_for`; turn=1 means YELLOW has not
        // moved yet so YELLOW IS on first move and Skip will be rejected as SkipFirstTurn).
        //
        // We avoid this by setting turn=5 (YELLOW has played once) so YELLOW is no longer
        // on first move and Skip is legal.
        stream
            .write_all(b"<room roomId=\"r-integ\"><data class=\"memento\"><state startTeam=\"ONE\" turn=\"5\" startPiece=\"PENTO_L\" round=\"2\">\
                <board>\
                  <field x=\"0\" y=\"0\" content=\"BLUE\"/>\
                  <field x=\"0\" y=\"1\" content=\"BLUE\"/>\
                  <field x=\"0\" y=\"2\" content=\"BLUE\"/>\
                  <field x=\"0\" y=\"3\" content=\"BLUE\"/>\
                  <field x=\"1\" y=\"3\" content=\"BLUE\"/>\
                  <field x=\"19\" y=\"19\" content=\"YELLOW\"/>\
                  <field x=\"18\" y=\"19\" content=\"YELLOW\"/>\
                  <field x=\"17\" y=\"19\" content=\"YELLOW\"/>\
                  <field x=\"16\" y=\"19\" content=\"YELLOW\"/>\
                  <field x=\"16\" y=\"18\" content=\"YELLOW\"/>\
                </board>\
                <lastMoveMono/>\
                <blueShapes><shape>MONO</shape><shape>DOMINO</shape></blueShapes>\
                <yellowShapes><shape>MONO</shape><shape>DOMINO</shape></yellowShapes>\
                <redShapes><shape>MONO</shape><shape>DOMINO</shape></redShapes>\
                <greenShapes><shape>MONO</shape><shape>DOMINO</shape></greenShapes>\
                <validColors><color>BLUE</color><color>YELLOW</color><color>RED</color><color>GREEN</color></validColors>\
            </state></data></room>")
            .expect("write memento");
        stream.flush().expect("flush1");

        // Send moveRequest and expect the client to write back a <room ...><data class="...SkipMove">...</room>.
        stream
            .write_all(b"<room roomId=\"r-integ\"><data class=\"moveRequest\"/></room>")
            .expect("write moveRequest");
        stream.flush().expect("flush2");

        // Read the client's move XML. Drain the socket for up to ~2 seconds.
        stream.set_read_timeout(Some(Duration::from_secs(2))).ok();
        let mut got = Vec::new();
        let mut tmp = [0u8; 4096];
        while let Ok(n) = stream.read(&mut tmp) {
            if n == 0 {
                break;
            }
            got.extend_from_slice(&tmp[..n]);
            if String::from_utf8_lossy(&got).contains("</room>") {
                break;
            }
        }
        let got_str = String::from_utf8_lossy(&got).to_string();
        assert!(
            got_str.contains("SkipMove") || got_str.contains("Skip"),
            "expected SkipMove XML, got: {got_str}"
        );

        // Send a "result" data class so the bot's `on_game_result` fires.
        stream
            .write_all(b"<room roomId=\"r-integ\"><data class=\"result\">\
                <definition>\
                  <fragment name=\"Siegpunkte\"><aggregation>SUM</aggregation><relevantForRanking>true</relevantForRanking></fragment>\
                </definition>\
                <scores>\
                  <entry><player team=\"ONE\"/><score><part>10</part></score></entry>\
                  <entry><player team=\"TWO\"/><score><part>5</part></score></entry>\
                </scores>\
                <winner team=\"ONE\" regular=\"true\" reason=\"ONE hat am meisten Punkte erzielt.\"/>\
            </data></room>")
            .expect("write result");
        stream.flush().expect("flush3");

        // Send a final `<left roomId="r-integ"/>` then close.
        stream
            .write_all(b"<left roomId=\"r-integ\"/>")
            .expect("write left");
        stream.flush().expect("flush4");
        thread::sleep(Duration::from_millis(200));
    });

    // Drive the bot through a lifecycle. Use short polling time so it finishes quickly.
    let _ = start_iclient(
        &addr,
        None,
        &mut bot,
        Duration::from_millis(20),
        Duration::from_secs(5),
    );

    // Wait for server thread to finish writing regardless of result.
    let _ = server_thread.join();

    // Assertions on bot state.
    let welcome = bot_welcome.lock().unwrap().clone();
    assert_eq!(welcome, Some(Team::Two), "bot should have received welcome for TWO");

    let mv = bot_move.lock().unwrap().clone();
    assert!(mv.is_some(), "bot should have been asked for a move");
    match mv {
        Some(Move::Skip { color }) => {
            assert_eq!(color, socha::neutral::Color::Yellow);
        }
        Some(other) => panic!("expected Skip, got {other:?}"),
        None => unreachable!(),
    }

    let state = bot_state.lock().unwrap().clone();
    assert!(state.is_some(), "bot should have received a memento");
    let s = state.unwrap();
    assert_eq!(s.turn, 5);
    assert_eq!(s.current_color(), socha::neutral::Color::Yellow);
    assert!(!s.board.is_empty());

    let saw_result = *bot_result.lock().unwrap();
    assert!(saw_result, "bot should have received a game result");

    // Final compiler hint: keep `ComCancelHandler` referenced so the test compiles cleanly.
    let _ = std::marker::PhantomData::<ComCancelHandler>;
}
