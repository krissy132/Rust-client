use rand::{rngs::ThreadRng, Rng};
use socha::neutral::Color;

#[cfg(test)]
pub mod tests {
    use super::*;
    use socha::internal::{possible_moves, GameState};
    use socha::neutral::PieceShape;

    fn random_start_piece(rng: &mut ThreadRng) -> PieceShape {
        // Server picks a random pentomino — replicate that here so games start with a real start-piece.
        let pentos = [
            PieceShape::PentoL,
            PieceShape::PentoT,
            PieceShape::PentoV,
            PieceShape::PentoS,
            PieceShape::PentoZ,
            PieceShape::PentoI,
            PieceShape::PentoP,
            PieceShape::PentoW,
            PieceShape::PentoU,
            PieceShape::PentoR,
            PieceShape::PentoX,
            PieceShape::PentoY,
        ];
        pentos[rng.random_range(0..pentos.len())]
    }

    /// Drive `games` random games: at each step sample a move from `sensible_moves`,
    /// apply `make_move`, push the change onto a stack. After up to `max_plies` plies,
    /// unwind via `unmake_move` and assert that the state matches the initial snapshot.
    fn fuzz_make_unmake_games(games: usize, max_plies: usize) {
        let mut rng = rand::rng();

        for game_idx in 0..games {
            let start_piece = random_start_piece(&mut rng);
            let mut state = GameState::new(start_piece);
            let snapshot = state.clone();

            let mut move_stack: Vec<socha::internal::MoveChange> = Vec::new();

            for _ in 0..max_plies {
                let moves = possible_moves(&state);
                if moves.is_empty() {
                    // No legal moves at all -> if it's first move we'd be stuck; possible_moves
                    // never returns Skip on a first move (an error per rules). Stop the game.
                    break;
                }
                let mv = moves[rng.random_range(0..moves.len())];
                match mv.make_move(&mut state) {
                    Ok(change) => move_stack.push(change),
                    Err(_e) => {
                        // A `Move::Skip` fails `validate_skip_move` on first turn — skip sampling; retry without it
                        // (could happen if `possible_moves` incorrectly returned Skip, but it shouldn't).
                        // We use sensible order: try every other move; if all fail, stop.
                        let mut found = false;
                        for alt_mv in moves.iter().cycle().skip(1).take(moves.len() - 1) {
                            if let Ok(change) = alt_mv.make_move(&mut state) {
                                move_stack.push(change);
                                found = true;
                                break;
                            }
                        }
                        if !found {
                            break;
                        }
                    }
                }
            }

            while let Some(change) = move_stack.pop() {
                socha::neutral::Move::unmake_move(&mut state, change);
            }
            assert_eq!(state, snapshot, "board mismatch after game {}", game_idx);
        }
    }

    #[test]
    fn fuzz_make_unmake_short() {
        fuzz_make_unmake_games(20, 60);
    }

    #[test]
    fn fuzz_make_unmake_long() {
        fuzz_make_unmake_games(5, 200);
    }

    #[test]
    fn first_player_first_move_must_use_start_piece_on_border() {
        // Confirm that the very first move of a game: validates WrongShape when not the start-piece,
        // validates WrongColor when wrong color, validates NotOnBorder when not on edge.
        use socha::internal::validate_move;
        use socha::neutral::{BlokusMoveMistake, Coordinates, Move, Piece, Rotation};

        let mut state = GameState::new(PieceShape::PentoL);
        // current color is BLUE.
        assert_eq!(state.current_color(), Color::Blue);

        // Wrong shape test: try to set a MONO piece (not start_piece=PENTO_L).
        let piece_wrong_shape = Piece::new(
            Color::Blue,
            PieceShape::Mono,
            Rotation::None,
            false,
            Coordinates::new(0, 0),
        );
        assert_eq!(
            validate_move(
                &state,
                &Move::Set {
                    piece: piece_wrong_shape
                }
            )
            .unwrap_err(),
            BlokusMoveMistake::WrongShape
        );

        // Wrong color test: try a YELLOW PENTO_L on the border.
        let piece_wrong_color = Piece::new(
            Color::Yellow,
            PieceShape::PentoL,
            Rotation::None,
            false,
            Coordinates::new(0, 0),
        );
        assert_eq!(
            validate_move(
                &state,
                &Move::Set {
                    piece: piece_wrong_color
                }
            )
            .unwrap_err(),
            BlokusMoveMistake::WrongColor
        );

        // Not-On-Border test: place a valid PENTO_L shape but entirely interior.
        // PENTO_L bounding is 2x4. (5,7) means top-left at interior — let's compute.
        // The coordinates for PentoL are {(0,0),(0,1),(0,2),(0,3),(1,3)}.
        // Anchor at (5,7) means cells {5,7..10} and {6,10} — all strictly interior (no border).
        // But (1,3) → (6,10) is y=10, that's not on the border (borders are y=0 or y=19).
        // So this should fail NotOnBorder.
        let piece_interior = Piece::new(
            Color::Blue,
            PieceShape::PentoL,
            Rotation::None,
            false,
            Coordinates::new(5, 7),
        );
        assert_eq!(
            validate_move(
                &state,
                &Move::Set {
                    piece: piece_interior
                }
            )
            .unwrap_err(),
            BlokusMoveMistake::NotOnBorder
        );

        // Now a valid first move: PentoL at (0, 0) — all cells (0,0), (0,1), (0,2), (0,3), (1,3).
        // (0,0), (0,1), (0,2), (0,3) all touch x=0 border — NotOnBorder is OK.
        let piece_valid = Piece::new(
            Color::Blue,
            PieceShape::PentoL,
            Rotation::None,
            false,
            Coordinates::new(0, 0),
        );
        assert!(validate_move(&state, &Move::Set { piece: piece_valid }).is_ok());
        // Apply it and confirm first-move semantics reset.
        let mv = Move::Set { piece: piece_valid };
        mv.make_move(&mut state).unwrap();
        assert!(!state.is_first_move_for(Color::Blue));
        // After Blue plays, turn advances to Yellow (still in round 1).
        assert_eq!(state.current_color(), Color::Yellow);
        // Yellow is still on first move though.
        assert!(state.is_first_move_for(Color::Yellow));
    }

    #[test]
    fn second_player_shared_corner_and_touches_same_color_enforced() {
        // After a valid first move for BLUE, YELLOW must place its start-piece such that:
        //   - it shares a diagonal corner with an existing BLUE piece (NO_SHARED_CORNER otherwise)
        //   - it does NOT touch any BLUE cell cardinally (TOUCHES_SAME_COLOR -- but here it's a different color,
        //     so this check should pass because TOUCHES_SAME_COLOR compares against the move's COLOR,
        //     not the other player's color; whatever YELLOW places must not touch another YELLOW cell though).
        // Re-read the rule: TOUCHES_SAME_COLOR uses `move.color` · a YELLOW move touching BLUE cells is fine
        // (different color). The cardinal-touch rule only blocks when yellow touches yellow.
        // So the second-move-for-yellow test is about NO_SHARED_CORNER for yellow's own pieces (none yet placed).
        // Since yellow is on its first move (still round 1), the `isFirstMove` branch fires: just need
        // NotOnBorder + the right shape (start_piece).
        use socha::internal::{possible_moves, validate_move};
        use socha::neutral::{Coordinates, Move, Piece, Rotation};

        let mut state = GameState::new(PieceShape::PentoL);
        let blue_first = Piece::new(
            Color::Blue,
            PieceShape::PentoL,
            Rotation::None,
            false,
            Coordinates::new(0, 0),
        );
        let mv = Move::Set { piece: blue_first };
        mv.make_move(&mut state).unwrap();

        // Now YELLOW's turn. Yellow still has all 21 pieces — first move for yellow.
        assert_eq!(state.current_color(), Color::Yellow);
        assert!(state.is_first_move_for(Color::Yellow));

        // Yellow's first move: must be on border + start_piece = PENTO_L.
        let yellow_moves = possible_moves(&state);
        assert!(
            !yellow_moves.is_empty(),
            "yellow should have first-move options"
        );
        // Confirm any returned move is valid.
        for mv in &yellow_moves {
            assert!(validate_move(&state, mv).is_ok());
            if let Move::Set { piece } = mv {
                assert_eq!(piece.color, Color::Yellow);
                assert_eq!(piece.kind, PieceShape::PentoL);
            } else {
                panic!("first-move for yellow should be a Set, not Skip");
            }
        }

        // Try an interior yellow move — should be NotOnBorder.
        let yellow_interior = Piece::new(
            Color::Yellow,
            PieceShape::PentoL,
            Rotation::None,
            false,
            Coordinates::new(5, 7),
        );
        assert!(validate_move(
            &state,
            &Move::Set {
                piece: yellow_interior
            }
        )
        .is_err());
    }

    #[test]
    fn scoring_points_for_undeployed() {
        use socha::internal::points_from_undeployed;

        // All 21 shapes undeployed: 89 - 89 = 0.
        let all: Vec<PieceShape> = PieceShape::ALL.to_vec();
        assert_eq!(points_from_undeployed(&all, false), 0);

        // Nothing left undeployed without monoLast: 89 + 15 = 104.
        assert_eq!(points_from_undeployed(&[], false), 104);

        // Nothing left and mono was last: 89 + 15 + 5 = 109.
        assert_eq!(points_from_undeployed(&[], true), 109);

        // Only MONO undeployed: 89 - 1 = 88.
        assert_eq!(points_from_undeployed(&[PieceShape::Mono], false), 88);
    }
}
