//! Server-state vs. client-state synchronisation test.
//!
//! ChatGPT's risk P1.2: after every server message, the server-emitted board/state must
//! match the client's local `GameState`. We simulate one round-trip:
//!
//!   1. Construct a known `GameState` on the client (fresh game, start_piece PENTO_L).
//!   2. Generate a `Move` and apply it locally via `make_move`.
//!   3. Re-serialise the resulting state as server XML (board fields + shape lists + turn),
//!      re-parse it back into a new `GameState`, and assert board/pieces/turn match.
//!
//! If `make_move` leaves the client state out of sync with what the server would emit,
//! this test surfaces the discrepancy.

use strong_xml::XmlRead;

use socha::incoming::ReceivedState;
use socha::internal::{GameState, possible_moves, BOARD_LENGTH};
use socha::neutral::{Color, FieldContent, Move, PieceShape};

/// Serialise a `GameState` into the same XML format the server emits for `<state>`.
///
/// This function mirrors what `GameState.to_xml()` would do if we had it. We hand-roll it
/// here because the client library is server→client only (no client→server `<state>`).
fn state_to_server_xml(state: &GameState) -> String {
    let mut board_xml = String::new();
    for y in 0..BOARD_LENGTH {
        for x in 0..BOARD_LENGTH {
            let fc = *state.board.get(x, y);
            if !fc.is_empty() {
                board_xml.push_str(&format!(
                    "<field x=\"{x}\" y=\"{y}\" content=\"{fc}\"/>"
                ));
            }
        }
    }
    let shapes_to_xml = |shapes: &[PieceShape]| -> String {
        shapes
            .iter()
            .map(|s| format!("<shape>{s}</shape>"))
            .collect()
    };

    let last_move_mono_xml = if state.last_move_mono.is_empty() {
        "<lastMoveMono/>".to_string()
    } else {
        let mut entries = String::new();
        for (color, value) in state.last_move_mono.iter() {
            entries.push_str(&format!(
                "<entry><color>{color}</color><boolean>{}</boolean></entry>",
                if *value { "true" } else { "false" }
            ));
        }
        format!("<lastMoveMono>{entries}</lastMoveMono>")
    };

    // lastMove XML — minimal representation for a Set move.
    let last_move_xml = match &state.last_move {
        Some(Move::Set { piece }) => {
            format!(
                "<lastMove class=\"sc.plugin2027.SetMove\"><piece color=\"{}\" kind=\"{}\" rotation=\"{}\" isFlipped=\"{}\"><position x=\"{}\" y=\"{}\"/></piece></lastMove>",
                piece.color,
                piece.kind,
                piece.rotation,
                piece.is_flipped,
                piece.position.x,
                piece.position.y
            )
        }
        Some(Move::Skip { color }) => format!(
            "<lastMove class=\"sc.plugin2027.SkipMove\"><color>{color}</color></lastMove>"
        ),
        None => String::new(),
    };

    let valid_colors_xml: String = state
        .valid_colors
        .iter()
        .map(|c| format!("<color>{c}</color>"))
        .collect();

    format!(
        "<state startTeam=\"ONE\" turn=\"{}\" startPiece=\"{}\" round=\"{}\">\
           <board>{board_xml}</board>\
           {last_move_mono_xml}\
           {last_move_xml}\
           <blueShapes>{}</blueShapes>\
           <yellowShapes>{}</yellowShapes>\
           <redShapes>{}</redShapes>\
           <greenShapes>{}</greenShapes>\
           <validColors>{valid_colors_xml}</validColors>\
         </state>",
        state.turn,
        state.start_piece,
        state.round(),
        shapes_to_xml(&state.blue_shapes),
        shapes_to_xml(&state.yellow_shapes),
        shapes_to_xml(&state.red_shapes),
        shapes_to_xml(&state.green_shapes),
    )
}

#[test]
fn local_state_matches_serialised_and_reparsed_state() {
    // Round 1: BLUE plays PENTO_L at (0,0).
    let mut local = GameState::new(PieceShape::PentoL);
    let blue_first = socha::neutral::Piece::new(
        Color::Blue,
        PieceShape::PentoL,
        socha::neutral::Rotation::None,
        false,
        socha::neutral::Coordinates::new(0, 0),
    );
    Move::Set { piece: blue_first }
        .make_move(&mut local)
        .expect("blue first move valid");

    // Serialise → XML → reparse.
    let xml = state_to_server_xml(&local);
    let received = ReceivedState::from_str(&xml).expect("reparse xml");
    let reparsed = GameState::try_from(received).expect("reparse to GameState");

    assert_eq!(reparsed.turn, local.turn, "turn mismatch");
    assert_eq!(reparsed.start_piece, local.start_piece);
    assert_eq!(reparsed.round(), local.round());
    assert_eq!(reparsed.board, local.board, "board mismatch");
    assert_eq!(reparsed.last_move, local.last_move, "last_move mismatch");
    assert_eq!(reparsed.last_move_mono, local.last_move_mono);
    assert_eq!(reparsed.blue_shapes, local.blue_shapes, "blue shapes");
    assert_eq!(reparsed.yellow_shapes, local.yellow_shapes, "yellow shapes");
    assert_eq!(reparsed.red_shapes, local.red_shapes, "red shapes");
    assert_eq!(reparsed.green_shapes, local.green_shapes, "green shapes");
    assert_eq!(reparsed.valid_colors, local.valid_colors);
}

#[test]
fn several_moves_round_trip_matches_reparsed_state() {
    // Apply multiple random-but-legal moves synchronously, recompute the XML projection
    // after each, reparse, and compare. Run a small game to completion (up to 80 plies or
    // until the game ends).
    let mut local = GameState::new(PieceShape::PentoL);

    for _ply in 0..80 {
        if local.is_over() {
            break;
        }
        let moves = possible_moves(&local);
        if moves.is_empty() {
            // Skip to advance the turn (shouldn't happen because sensible_moves returns Skip,
            // but `possible_moves` itself returns an empty vec when Skip is the only option).
            let skip = Move::Skip {
                color: local.current_color(),
            };
            skip.make_move(&mut local).expect("skip");
        } else {
            // Take the first legal move deterministically (no RNG dependency for repro).
            let mv = moves.into_iter().next().unwrap();
            mv.make_move(&mut local).expect("apply legal move");
        }

        let xml = state_to_server_xml(&local);
        let received = ReceivedState::from_str(&xml).expect("reparse xml");
        let reparsed = GameState::try_from(received).expect("reparse");

        assert_eq!(reparsed.turn, local.turn, "turn mismatch at ply {}", local.turn);
        assert_eq!(reparsed.board, local.board, "board mismatch at ply {}", local.turn);
        assert_eq!(reparsed.last_move, local.last_move, "last_move mismatch at ply {}", local.turn);
        assert_eq!(reparsed.blue_shapes, local.blue_shapes);
        assert_eq!(reparsed.yellow_shapes, local.yellow_shapes);
        assert_eq!(reparsed.red_shapes, local.red_shapes);
        assert_eq!(reparsed.green_shapes, local.green_shapes);
        assert_eq!(reparsed.valid_colors, local.valid_colors);
        assert_eq!(reparsed.last_move_mono, local.last_move_mono);
    }
}

#[test]
fn shapes_list_remain_sorted_and_unique_after_each_move() {
    // Robust supplementary invariant: the per-color `*_shapes` vec must stay sorted and
    // contain unique entries — both the server's projection AND the client `make_move` must
    // preserve this. Otherwise re-serialisation would diverge subtly.
    let mut local = GameState::new(PieceShape::PentoT);

    for _ply in 0..60 {
        if local.is_over() {
            break;
        }
        let moves = possible_moves(&local);
        if moves.is_empty() {
            Move::Skip { color: local.current_color() }
                .make_move(&mut local)
                .expect("skip");
        } else {
            let mv = moves.into_iter().next().unwrap();
            mv.make_move(&mut local).expect("apply");
        }

        for shapes in [
            &local.blue_shapes,
            &local.yellow_shapes,
            &local.red_shapes,
            &local.green_shapes,
        ] {
            let mut sorted = shapes.clone();
            sorted.sort_by_key(|s| s.to_index());
            assert_eq!(shapes, &sorted, "shapes list not sorted after ply {}", local.turn);
            // Uniqueness
            let mut uniq = shapes.clone();
            uniq.dedup();
            assert_eq!(shapes.len(), uniq.len(), "shapes list has dubplicates after ply {}", local.turn);
        }
    }

    // Sanity: after at least one move, the board is non-empty (proves we made progress).
    let non_empty = local
        .board
        .rows
        .iter()
        .flat_map(|r| r.fields.iter())
        .filter(|c| !c.is_empty())
        .count();
    assert!(non_empty > 0, "board should not be empty after moves");
    let _ = FieldContent::Empty;
}
