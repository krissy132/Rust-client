//! Move-generation completeness tests.
//!
//! `possible_moves` returning *some* valid moves is necessary but not sufficient —
//! a robust engine must enumerate **all legal Set moves** with no gaps and no duplicates.
//! ChatGPT's risk assessment P1.4 was that we currently only validate `_a_` move rather
//! than comparisons against expected complete move sets.
//!
//! Strategy: hand-constructed `GameState` fixtures with a known reference move set
//! (represented as collections of (PieceShape, sorted board-cells)). Then assert that
//! `possible_moves` produces exactly that set.

use std::collections::BTreeSet;

use socha::internal::{possible_moves, GameState};
use socha::neutral::{Color, Coordinates, Move, PieceShape};

/// Canonical key for a move: (kind, sorted absolute board coordinates).
/// Two moves occupying the same cells with the same shape are considered equivalent,
/// regardless of which `(rotation, is_flipped, position)` triple describes the piece.
fn move_key(mv: &Move) -> (PieceShape, BTreeSet<(i32, i32)>) {
    match mv {
        Move::Set { piece } => {
            let cells: BTreeSet<(i32, i32)> = piece
                .coordinates()
                .into_iter()
                .map(|c| (c.x, c.y))
                .collect();
            (piece.kind, cells)
        }
        Move::Skip { color } => {
            // Skip moves get a sentinel empty shape with the color embedded in a dummy cell.
            let _ = color;
            (PieceShape::Mono, BTreeSet::new())
        }
    }
}

fn moves_as_set(moves: &[Move]) -> BTreeSet<(PieceShape, BTreeSet<(i32, i32)>)> {
    moves.iter().map(move_key).collect()
}

#[test]
fn first_move_for_blue_yields_all_start_piece_border_placements() {
    // On a fresh board with start_piece=PENTO_L, BLUE's first move must enumerate every
    // legal border placement of every variant of PENTO_L. We cross-check by recomputing
    // the variant enumeration independently via `PieceShape::variants()` + manual
    // try-add — this catches both under-enumeration (missing variant or coordinate) and
    // over-enumeration (impossible positions counted).
    use socha::internal::BOARD_LENGTH;
    use socha::neutral::Piece;
    use socha::internal::validate_set_move;

    let state = GameState::new(PieceShape::PentoL);
    let generated = possible_moves(&state);
    assert!(!generated.is_empty(), "first move should have options");

    // Independent expected set: for each variant, for each board position that keeps the
    // piece fully in-bounds, accept it iff validate_set_move says OK.
    let mut expected: BTreeSet<(PieceShape, BTreeSet<(i32, i32)>)> = BTreeSet::new();
    for variant in PieceShape::PentoL.variants() {
        let mut shape = variant.shape.clone();
        shape.sort();
        for y in 0..BOARD_LENGTH as i32 {
            for x in 0..BOARD_LENGTH as i32 {
                let piece = Piece::new(
                    Color::Blue,
                    PieceShape::PentoL,
                    variant.rotation,
                    variant.is_flipped,
                    Coordinates::new(x, y),
                );
                if validate_set_move(&state, &piece).is_ok() {
                    let cells: BTreeSet<(i32, i32)> =
                        piece.coordinates().into_iter().map(|c| (c.x, c.y)).collect();
                    expected.insert((PieceShape::PentoL, cells));
                }
            }
        }
    }
    assert!(!expected.is_empty(), "expected set should not be empty");

    let gen_set = moves_as_set(&generated);
    if gen_set != expected {
        let missing: Vec<_> = expected.difference(&gen_set).cloned().collect();
        let extra: Vec<_> = gen_set.difference(&expected).cloned().collect();
        panic!(
            "move set mismatch.\n  missing (in expected, not in generated): {missing:?}\n  \
             extra   (in generated, not in expected): {extra:?}"
        );
    }
}

#[test]
fn first_move_for_each_color_uses_start_piece() {
    // Every first move for every colour must place the start_piece (not any other shape).
    // This protects against a regression where `is_first_move_for` is checked incorrectly
    // and a wrong shape is allowed.
    use socha::neutral::PieceShape;
    let mut state = GameState::new(PieceShape::PentoT);
    for c in Color::ALL {
        if state.current_color() != c {
            // advance manually until c is current
            while state.current_color() != c {
                if !state.advance() {
                    break;
                }
            }
        }
        if !state.is_first_move_for(c) {
            continue;
        }
        let moves = possible_moves(&state);
        assert!(!moves.is_empty(), "first move for {c:?} should have options");
        for mv in &moves {
            match mv {
                Move::Set { piece } => {
                    assert_eq!(
                        piece.kind,
                        PieceShape::PentoT,
                        "{c:?}'s first move must use the start piece, got {piece:?}"
                    );
                }
                Move::Skip { .. } => {
                    panic!("{c:?}'s first move must not be a Skip");
                }
            }
        }
    }
}

#[test]
fn normal_moves_cover_all_undeployed_shape_variants_with_corner_constraint() {
    // Construct a mid-game state where BLUE has placed exactly one PENTO_L at (0,0) and
    // has 20 remaining shapes. Verify every generated move:
    //   - uses one of those 20 shapes
    //   - shares a corner with an existing BLUE piece
    //   - does not overlap or touch cardinal side
    // Then independently enumerate all legal positions for every shape+variant+anchor
    // triplet and verify the move generator finds exactly that set.
    use socha::internal::{validate_set_move, BOARD_LENGTH};
    use socha::neutral::{Piece, Rotation};

    let mut state = GameState::new(PieceShape::PentoL);
    // Place the first BLUE PENTO_L at (0,0).
    let first = Piece::new(
        Color::Blue,
        PieceShape::PentoL,
        Rotation::None,
        false,
        Coordinates::new(0, 0),
    );
    Move::Set { piece: first }
        .make_move(&mut state)
        .expect("first move should validate");

    // Skip Yellow/Red/Green turns by emptying their piece lists — `possible_moves_normal`
    // will return no Set-moves for them; we manually advance past their turns to BLUE.
    state.advance(); // yellow's turn

    // For YELLOW/RED/GREEN's first move they must use the start piece on the border — for
    // our purposes we instead fast-forward BLUE's next turn by manually placing a piece
    // for each of Y/R/G so BLUE gets another turn. The simplest path: discard Y/R/G to
    // make their turns empty, but the engine requires them to play their start piece first.
    //
    // Easier path: take BLUE's second move at turn=4 (since turn=4 mod 4 == BLUE).
    // To get there without messing with intermediate colours, we set their `*_shapes`
    // to contain only one shape (MONO) and `valid_colors` only BLUE+YELLOW. Then YELLOW
    // is forced to skip after first move (its first move uses start piece, then runs out),
    // RED/GREEN are removed from valid_colors.
    state.valid_colors = vec![Color::Blue, Color::Yellow];
    // Re-init turn to 0 to keep things predictable.
    state.turn = 0;
    // Re-place the blue first piece.
    state.board = Default::default();
    state.blue_shapes = PieceShape::ALL.to_vec();
    state.yellow_shapes = vec![PieceShape::Mono];
    state.last_move_mono.clear();
    let first = Piece::new(
        Color::Blue,
        PieceShape::PentoL,
        Rotation::None,
        false,
        Coordinates::new(0, 0),
    );
    Move::Set { piece: first }.make_move(&mut state).expect("blue first");

    // YELLOW turn: must place start_piece on border (still first move for yellow). Place a MONO.
    // But yellow's undeployed list has only MONO while start_piece=PENTO_L — `validate_set_move`
    // requires start_piece on first move, so MONO is rejected. Skip logic: `possible_start_moves`
    // is what we'd use, but it returns nothing. For testing, instead we make yellow also start
    // with start_piece=PENTO_L by setting yellow_shapes back to all_shapes once.
    state.yellow_shapes = PieceShape::ALL.to_vec();
    // YELLOW at bottom-right corner. PENTO_L variants occupy bbox (2x4 or 4x2); the anchor
    // (18,16) with Rotation::None lands cells {(18,16),(18,17),(18,18),(18,19),(19,19)}
    // — all in-bounds and on the right/bottom border.
    let yellow_first = Piece::new(
        Color::Yellow,
        PieceShape::PentoL,
        Rotation::None,
        false,
        Coordinates::new(18, 16),
    );
    Move::Set { piece: yellow_first }
        .make_move(&mut state)
        .expect("yellow first");

    // Now it's BLUE's turn again (not first-move-for-blue, since one piece placed).
    assert_eq!(state.current_color(), Color::Blue);
    assert!(!state.is_first_move_for(Color::Blue));

    let generated = possible_moves(&state);
    assert!(!generated.is_empty(), "blue should have follow-up moves");

    // Build the expected set independently: for each undeployed shape, for each variant,
    // for each anchor coordinate, validate via `validate_set_move`.
    let mut expected: BTreeSet<(PieceShape, BTreeSet<(i32, i32)>)> = BTreeSet::new();
    let color = Color::Blue;
    for &kind in state.undeployed(color) {
        for variant in kind.variants() {
            for y in 0..BOARD_LENGTH as i32 {
                for x in 0..BOARD_LENGTH as i32 {
                    let piece = Piece::new(
                        color,
                        kind,
                        variant.rotation,
                        variant.is_flipped,
                        Coordinates::new(x, y),
                    );
                    if validate_set_move(&state, &piece).is_ok() {
                        let cells: BTreeSet<(i32, i32)> =
                            piece.coordinates().into_iter().map(|c| (c.x, c.y)).collect();
                        expected.insert((kind, cells));
                    }
                }
            }
        }
    }
    assert!(!expected.is_empty(), "expected set should not be empty");

    let gen_set = moves_as_set(&generated);
    if gen_set != expected {
        let missing: Vec<_> = expected.difference(&gen_set).cloned().collect();
        let extra: Vec<_> = gen_set.difference(&expected).cloned().collect();
        // Print counts first for triage.
        eprintln!(
            "expected: {} moves, generated: {} moves; missing {}, extra {}",
            expected.len(),
            gen_set.len(),
            missing.len(),
            extra.len()
        );
        // Limit noise: print up to 3 samples each.
        if !missing.is_empty() {
            eprintln!("missing samples: {:?}", &missing[..missing.len().min(3)]);
        }
        if !extra.is_empty() {
            eprintln!("extra samples: {:?}", &extra[..extra.len().min(3)]);
        }
        panic!("normal-move enumeration mismatch");
    }
}

#[test]
fn possible_moves_no_duplicates() {
    // The generator must not emit duplicate moves. Two Set-moves that occupy the same set of
    // cells with the same shape are equivalent for game-state purposes — so even if the
    // (rotation, flip, position) triple differs slightly due to alignment fuzz, they should
    // not both appear in the output list. We accept either: they don't appear as duplicates
    // (preferred), or — failing that — we at least surface this behaviour as a test failure.
    let state = GameState::new(PieceShape::PentoL);
    let moves = possible_moves(&state);
    let set = moves_as_set(&moves);
    assert_eq!(
        set.len(),
        moves.len(),
        "{} duplicate moves emitted (set={}, list={})",
        moves.len() - set.len(),
        set.len(),
        moves.len()
    );
}
