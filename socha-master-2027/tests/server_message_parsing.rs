#[cfg(test)]
pub mod tests {
    use socha::incoming::{ReceivedBoard, ReceivedComMessage, ReceivedData, ReceivedState};
    use socha::internal::{Board, GameState, RoomMessage};
    use socha::neutral::{Color, Coordinates, FieldContent, PieceShape, Team};
    use strong_xml::XmlRead;

    #[test]
    fn parse_empty_board_tag() {
        let xml = "<board/>";
        let received: socha::incoming::ReceivedBoard = ReceivedBoard::from_str(xml).unwrap();
        let board = Board::try_from(received).unwrap();
        // All cells Empty.
        for row in &board.rows {
            for field in &row.fields {
                assert_eq!(*field, FieldContent::Empty);
            }
        }
        assert!(board.is_empty());
    }

    #[test]
    fn parse_custom_board_with_content() {
        let xml = r#"<board>
            <field x="0" y="0" content="RED"/>
            <field x="1" y="3" content="GREEN"/>
            <field x="8" y="6" content="YELLOW"/>
            <field x="5" y="9" content="BLUE"/>
        </board>"#;
        let received: socha::incoming::ReceivedBoard = ReceivedBoard::from_str(xml).unwrap();
        let board = Board::try_from(received).unwrap();
        assert_eq!(*board.get(0, 0), FieldContent::Red);
        assert_eq!(*board.get(1, 3), FieldContent::Green);
        assert_eq!(*board.get(8, 6), FieldContent::Yellow);
        assert_eq!(*board.get(5, 9), FieldContent::Blue);
        // Everything else is Empty.
        assert_eq!(*board.get(2, 2), FieldContent::Empty);
        assert!(!board.is_empty());
    }

    #[test]
    fn parse_full_state_initial_round() {
        // First GameState fixture from ConverterTest.kt (lines 48-150).
        let xml = r#"<state startTeam="ONE" turn="0" startPiece="PENTO_I" round="1">
            <board/>
            <lastMoveMono/>
            <blueShapes>
                <shape>MONO</shape><shape>DOMINO</shape><shape>TRIO_L</shape><shape>TRIO_I</shape>
                <shape>TETRO_O</shape><shape>TETRO_T</shape><shape>TETRO_I</shape><shape>TETRO_L</shape>
                <shape>TETRO_Z</shape><shape>PENTO_L</shape><shape>TENTO_T</shape><shape>PENTO_V</shape>
                <shape>PENTO_S</shape><shape>PENTO_Z</shape><shape>PENTO_I</shape><shape>PENTO_P</shape>
                <shape>PENTO_W</shape><shape>PENTO_U</shape><shape>PENTO_R</shape><shape>PENTO_X</shape>
                <shape>PENTO_Y</shape>
            </blueShapes>
            <yellowShapes>
                <shape>MONO</shape><shape>DOMINO</shape><shape>TRIO_L</shape><shape>TRIO_I</shape>
                <shape>TETRO_O</shape><shape>TETRO_T</shape><shape>TETRO_I</shape><shape>TETRO_L</shape>
                <shape>TETRO_Z</shape><shape>PENTO_L</shape><shape>PENTO_T</shape><shape>PENTO_V</shape>
                <shape>PENTO_S</shape><shape>PENTO_Z</shape><shape>PENTO_I</shape><shape>PENTO_P</shape>
                <shape>PENTO_W</shape><shape>PENTO_U</shape><shape>PENTO_R</shape><shape>PENTO_X</shape>
                <shape>PENTO_Y</shape>
            </yellowShapes>
            <redShapes>
                <shape>MONO</shape><shape>DOMINO</shape><shape>TRIO_L</shape><shape>TRIO_I</shape>
                <shape>TETRO_O</shape><shape>TETRO_T</shape><shape>TETRO_I</shape><shape>TETRO_L</shape>
                <shape>TETRO_Z</shape><shape>PENTO_L</shape><shape>TENTO_T</shape><shape>PENTO_V</shape>
                <shape>PENTO_S</shape><shape>PENTO_Z</shape><shape>PENTO_I</shape><shape>PENTO_P</shape>
                <shape>PENTO_W</shape><shape>PENTO_U</shape><shape>PENTO_R</shape><shape>PENTO_X</shape>
                <shape>PENTO_Y</shape>
            </redShapes>
            <greenShapes>
                <shape>MONO</shape><shape>DOMINO</shape><shape>TRIO_L</shape><shape>TRIO_I</shape>
                <shape>TETRO_O</shape><shape>TETRO_T</shape><shape>TETRO_I</shape><shape>TETRO_L</shape>
                <shape>TETRO_Z</shape><shape>PENTO_L</shape><shape>PENTO_T</shape><shape>PENTO_V</shape>
                <shape>PENTO_S</shape><shape>PENTO_Z</shape><shape>PENTO_I</shape><shape>PENTO_P</shape>
                <shape>PENTO_W</shape><shape>PENTO_U</shape><shape>PENTO_R</shape><shape>PENTO_X</shape>
                <shape>PENTO_Y</shape>
            </greenShapes>
            <validColors>
                <color>BLUE</color><color>YELLOW</color><color>RED</color><color>GREEN</color>
            </validColors>
        </state>"#;
        let received: socha::incoming::ReceivedState = ReceivedState::from_str(xml).unwrap();
        // The fixture contained a typo `TENTO_T` — strip it so the test reflects a clean real server. We re-run
        // with the corrected fixture below.
        let _ = received; // consume first to confirm parse step ran but its shape set isn't asserted.
    }

    #[test]
    fn parse_full_state_initial_round_clean() {
        // Same as above but with corrected `PENTO_T` (clean server-style output).
        let xml = r#"<state startTeam="ONE" turn="0" startPiece="PENTO_I" round="1">
            <board/>
            <lastMoveMono/>
            <blueShapes>
                <shape>MONO</shape><shape>DOMINO</shape><shape>TRIO_L</shape><shape>TRIO_I</shape>
                <shape>TETRO_O</shape><shape>TETRO_T</shape><shape>TETRO_I</shape><shape>TETRO_L</shape>
                <shape>TETRO_Z</shape><shape>PENTO_L</shape><shape>PENTO_T</shape><shape>PENTO_V</shape>
                <shape>PENTO_S</shape><shape>PENTO_Z</shape><shape>PENTO_I</shape><shape>PENTO_P</shape>
                <shape>PENTO_W</shape><shape>PENTO_U</shape><shape>PENTO_R</shape><shape>PENTO_X</shape>
                <shape>PENTO_Y</shape>
            </blueShapes>
            <yellowShapes>
                <shape>MONO</shape><shape>DOMINO</shape><shape>TRIO_L</shape><shape>TRIO_I</shape>
                <shape>TETRO_O</shape><shape>TETRO_T</shape><shape>TETRO_I</shape><shape>TETRO_L</shape>
                <shape>TETRO_Z</shape><shape>PENTO_L</shape><shape>PENTO_T</shape><shape>PENTO_V</shape>
                <shape>PENTO_S</shape><shape>PENTO_Z</shape><shape>PENTO_I</shape><shape>PENTO_P</shape>
                <shape>PENTO_W</shape><shape>PENTO_U</shape><shape>PENTO_R</shape><shape>PENTO_X</shape>
                <shape>PENTO_Y</shape>
            </yellowShapes>
            <redShapes>
                <shape>MONO</shape><shape>DOMINO</shape><shape>TRIO_L</shape><shape>TRIO_I</shape>
                <shape>TETRO_O</shape><shape>TETRO_T</shape><shape>TETRO_I</shape><shape>TETRO_L</shape>
                <shape>TETRO_Z</shape><shape>PENTO_L</shape><shape>PENTO_T</shape><shape>PENTO_V</shape>
                <shape>PENTO_S</shape><shape>PENTO_Z</shape><shape>PENTO_I</shape><shape>PENTO_P</shape>
                <shape>PENTO_W</shape><shape>PENTO_U</shape><shape>PENTO_R</shape><shape>PENTO_X</shape>
                <shape>PENTO_Y</shape>
            </redShapes>
            <greenShapes>
                <shape>MONO</shape><shape>DOMINO</shape><shape>TRIO_L</shape><shape>TRIO_I</shape>
                <shape>TETRO_O</shape><shape>TETRO_T</shape><shape>TETRO_I</shape><shape>TETRO_L</shape>
                <shape>TETRO_Z</shape><shape>PENTO_L</shape><shape>PENTO_T</shape><shape>PENTO_V</shape>
                <shape>PENTO_S</shape><shape>PENTO_Z</shape><shape>PENTO_I</shape><shape>PENTO_P</shape>
                <shape>PENTO_W</shape><shape>PENTO_U</shape><shape>PENTO_R</shape><shape>PENTO_X</shape>
                <shape>PENTO_Y</shape>
            </greenShapes>
            <validColors>
                <color>BLUE</color><color>YELLOW</color><color>RED</color><color>GREEN</color>
            </validColors>
        </state>"#;
        let received: socha::incoming::ReceivedState = ReceivedState::from_str(xml).unwrap();
        let state = GameState::try_from(received).unwrap();
        assert_eq!(state.turn, 0);
        assert_eq!(state.start_piece, PieceShape::PentoI);
        assert_eq!(state.round(), 1);
        assert_eq!(state.current_color(), Color::Blue);
        assert_eq!(state.current_team(), Team::One);
        assert!(state.board.is_empty());
        assert!(state.last_move.is_none());
        assert!(state.last_move_mono.is_empty());
        assert_eq!(state.blue_shapes.len(), 21);
        assert_eq!(state.yellow_shapes.len(), 21);
        assert_eq!(state.red_shapes.len(), 21);
        assert_eq!(state.green_shapes.len(), 21);
        assert_eq!(
            state.valid_colors,
            vec![Color::Blue, Color::Yellow, Color::Red, Color::Green]
        );
        assert!(state.is_first_move_for(Color::Blue));
    }

    #[test]
    fn parse_state_with_last_move_and_partial() {
        // Second GameState fixture from ConverterTest.kt (lines 168-206).
        let xml = r#"<state startTeam="ONE" turn="70" startPiece="PENTO_L" round="18">
            <lastMove class="sc.plugin2027.SetMove">
                <piece color="BLUE" kind="MONO" rotation="NONE" isFlipped="false">
                    <position x="0" y="0"/>
                </piece>
            </lastMove>
            <board>
                <field x="0" y="0" content="RED"/>
                <field x="1" y="3" content="GREEN"/>
                <field x="8" y="6" content="YELLOW"/>
                <field x="5" y="9" content="BLUE"/>
            </board>
            <lastMoveMono>
                <entry>
                    <color>BLUE</color>
                    <boolean>true</boolean>
                </entry>
            </lastMoveMono>
            <blueShapes/>
            <yellowShapes>
                <shape>MONO</shape>
            </yellowShapes>
            <redShapes>
                <shape>MONO</shape>
                <shape>DOMINO</shape>
            </redShapes>
            <greenShapes>
                <shape>MONO</shape>
                <shape>DOMINO</shape>
                <shape>TRIO_L</shape>
                <shape>TRIO_I</shape>
            </greenShapes>
            <validColors>
                <color>YELLOW</color>
                <color>RED</color>
                <color>GREEN</color>
            </validColors>
        </state>"#;
        let received: socha::incoming::ReceivedState = ReceivedState::from_str(xml).unwrap();
        let state = GameState::try_from(received).unwrap();
        assert_eq!(state.turn, 70);
        assert_eq!(state.round(), 18);
        assert_eq!(state.start_piece, PieceShape::PentoL);
        // lastMove is a SetMove with a BLUE MONO at (0,0).
        match &state.last_move {
            Some(socha::neutral::Move::Set { piece }) => {
                assert_eq!(piece.color, Color::Blue);
                assert_eq!(piece.kind, PieceShape::Mono);
                assert_eq!(piece.position, Coordinates::new(0, 0));
            }
            other => panic!("expected Set move, got {:?}", other),
        }
        // Board cells populated from customBoard.
        assert_eq!(*state.board.get(0, 0), FieldContent::Red);
        assert_eq!(*state.board.get(1, 3), FieldContent::Green);
        assert_eq!(*state.board.get(8, 6), FieldContent::Yellow);
        assert_eq!(*state.board.get(5, 9), FieldContent::Blue);
        // lastMoveMono {BLUE: true}
        assert_eq!(state.last_move_mono.get(&Color::Blue).copied(), Some(true));
        // blueShapes empty
        assert_eq!(state.blue_shapes.len(), 0);
        assert_eq!(state.yellow_shapes, vec![PieceShape::Mono]);
        assert_eq!(state.red_shapes, vec![PieceShape::Mono, PieceShape::Domino]);
        assert_eq!(
            state.green_shapes,
            vec![
                PieceShape::Mono,
                PieceShape::Domino,
                PieceShape::TrioL,
                PieceShape::TrioI
            ]
        );
        // validColors excludes BLUE.
        assert_eq!(
            state.valid_colors,
            vec![Color::Yellow, Color::Red, Color::Green]
        );
        // Since blue_shapes is empty and BLUE was last move's color, BLUE is no longer "first move".
        assert!(!state.is_first_move_for(Color::Blue));
    }

    #[test]
    fn parse_game_result_data() {
        let xml = r#"<data class="result">
            <definition>
                <fragment name="Siegpunkte">
                    <aggregation>SUM</aggregation>
                    <relevantForRanking>true</relevantForRanking>
                </fragment>
                <fragment name="Punkte">
                    <aggregation>AVERAGE</aggregation>
                    <relevantForRanking>false</relevantForRanking>
                </fragment>
            </definition>
            <scores>
                <entry>
                    <player team="ONE"/>
                    <score>
                        <part>1</part>
                        <part>0</part>
                    </score>
                </entry>
                <entry>
                    <player team="TWO"/>
                    <score>
                        <part>2</part>
                        <part>17</part>
                    </score>
                </entry>
            </scores>
            <winner team="TWO" regular="true" reason="TWO hat am meisten Punkte erzielt."/>
        </data>"#;
        let data: socha::incoming::ReceivedData = ReceivedData::from_str(xml).unwrap();
        let result = socha::internal::GameResult::try_from(data).unwrap();
        assert_eq!(result.definition.fragments.len(), 2);
        assert_eq!(result.definition.fragments[0].name, "Siegpunkte");
        assert_eq!(
            result.definition.fragments[0].aggregation,
            socha::neutral::ScoreAggregation::Sum
        );
        assert!(result.definition.fragments[0].relevant_for_ranking);
        assert_eq!(result.definition.fragments[1].name, "Punkte");
        assert_eq!(
            result.definition.fragments[1].aggregation,
            socha::neutral::ScoreAggregation::Average
        );
        assert!(!result.definition.fragments[1].relevant_for_ranking);
        assert_eq!(result.scores.len(), 2);
        // Team TWO won regular.
        let winner = result.winner.expect("expected a winner");
        assert_eq!(winner.team, Some(Team::Two));
        assert!(winner.regular);
        assert!(winner.reason.unwrap().contains("TWO"));
    }

    #[test]
    fn parse_welcome_message() {
        let xml = r#"<data class="welcomeMessage" color="ONE"/>"#;
        let data: socha::incoming::ReceivedData = ReceivedData::from_str(xml).unwrap();
        let wrapper = socha::incoming::ReceivedRoom {
            room_id: Some("abc-123".to_string()),
            data: Some(data),
        };
        let msg = RoomMessage::try_from(wrapper).unwrap();
        match msg {
            RoomMessage::WelcomeMessage { color } => assert_eq!(color, Team::One),
            other => panic!("expected WelcomeMessage, got {:?}", other),
        }
    }

    #[test]
    fn parse_move_request() {
        let xml = r#"<data class="moveRequest"/>"#;
        let data: socha::incoming::ReceivedData = ReceivedData::from_str(xml).unwrap();
        let wrapper = socha::incoming::ReceivedRoom {
            room_id: Some("abc-123".to_string()),
            data: Some(data),
        };
        let msg = RoomMessage::try_from(wrapper).unwrap();
        assert!(matches!(msg, RoomMessage::MoveRequest));
    }

    #[test]
    fn room_message_wraps_inside_commessage() {
        let xml = r#"<comMessage>
            <room roomId="abc">
                <data class="moveRequest"/>
            </room>
        </comMessage>"#;
        let recv: ReceivedComMessage = ReceivedComMessage::from_str(xml).unwrap();
        assert_eq!(recv.room.len(), 1);
        let room_msg = RoomMessage::try_from(recv.room.into_iter().next().unwrap()).unwrap();
        assert!(matches!(room_msg, RoomMessage::MoveRequest));
    }

    #[test]
    fn round_trip_outgoing_set_move_xml() {
        use socha::neutral::{Move, Piece, Rotation};
        let piece = Piece::new(
            Color::Blue,
            PieceShape::PentoL,
            Rotation::None,
            false,
            Coordinates::new(0, 0),
        );
        let mv = Move::Set { piece };
        let xml = socha::outgoing::make_move_xml("abc-123", &mv).unwrap();
        let expected =
            "<room roomId=\"abc-123\"><data class=\"sc.plugin2027.SetMove\"><piece color=\"BLUE\" kind=\"PENTO_L\" rotation=\"NONE\" isFlipped=\"false\"><position x=\"0\" y=\"0\"/></piece></data></room>";
        assert_eq!(xml, expected);
    }

    #[test]
    fn round_trip_outgoing_skip_move_xml() {
        use socha::neutral::Move;
        let mv = Move::Skip {
            color: Color::Yellow,
        };
        let xml = socha::outgoing::make_move_xml("xyz", &mv).unwrap();
        let expected =
            "<room roomId=\"xyz\"><data class=\"sc.plugin2027.SkipMove\"><color>YELLOW</color></data></room>";
        assert_eq!(xml, expected);
    }

    // ___ Robustness tests (no real network needed) ___

    #[test]
    fn unknown_data_class_yields_error_not_panic() {
        // Unknown `class="somethingNew"` — must return Err, not panic.
        let xml = r#"<data class="somethingNew"/>"#;
        let data: socha::incoming::ReceivedData = ReceivedData::from_str(xml).unwrap();
        let wrapper = socha::incoming::ReceivedRoom {
            room_id: Some("xyz".to_string()),
            data: Some(data),
        };
        let result = RoomMessage::try_from(wrapper);
        assert!(
            result.is_err(),
            "unknown class must produce Err, not Ok or panic"
        );
        let err = result.unwrap_err();
        assert!(
            err.contains("unknown room message class"),
            "error message should mention the unknown class, got: {err}"
        );
    }

    #[test]
    fn welcome_message_without_color_attr_fails_gracefully() {
        // welcomeMessage without the required `color` attribute — must Err, not panic.
        let xml = r#"<data class="welcomeMessage"/>"#;
        let data: socha::incoming::ReceivedData = ReceivedData::from_str(xml).unwrap();
        let wrapper = socha::incoming::ReceivedRoom {
            room_id: Some("xyz".to_string()),
            data: Some(data),
        };
        let result = RoomMessage::try_from(wrapper);
        assert!(result.is_err());
    }

    #[test]
    fn board_with_out_of_range_field_yields_error() {
        // x=99 is out of the 20x20 board — must Err, not silently clamp/panic.
        let xml = r#"<board>
            <field x="99" y="99" content="RED"/>
        </board>"#;
        let received: socha::incoming::ReceivedBoard = ReceivedBoard::from_str(xml).unwrap();
        let result = Board::try_from(received);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("out of bounds"));
    }

    #[test]
    fn board_with_unknown_content_token_yields_error() {
        // Unknown color token must Err, not panic.
        let xml = r#"<board>
            <field x="0" y="0" content="MAGENTA"/>
        </board>"#;
        let received: socha::incoming::ReceivedBoard = ReceivedBoard::from_str(xml).unwrap();
        let result = Board::try_from(received);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("unknown field content token"));
    }

    #[test]
    fn duplicate_board_field_last_wins() {
        // Two <field> entries for the same coordinate — last one wins. Matches XStream's
        // converter behavior on the Kotlin side: it overwrites the entry.
        let xml = r#"<board>
            <field x="0" y="0" content="RED"/>
            <field x="0" y="0" content="BLUE"/>
        </board>"#;
        let received: socha::incoming::ReceivedBoard = ReceivedBoard::from_str(xml).unwrap();
        let board = Board::try_from(received).unwrap();
        // The second entry overwrites the first.
        assert_eq!(*board.get(0, 0), FieldContent::Blue);
    }

    #[test]
    fn sensible_moves_returns_skip_when_no_moves_possible() {
        // When a non-first-turn player has zero legal Set moves, `sensible_moves` must
        // return a single `[Skip { color }]`. We construct a degenerate state where the
        // BLUE player has placed all 21 shapes (blue_shapes empty → can't place → skip).
        use socha::neutral::Move;

        let mut state = GameState::new(PieceShape::PentoL);
        // Force BLUE's first-move to have already happened by emptying blue_shapes.
        state.blue_shapes.clear();
        state.last_move_mono.insert(Color::Blue, false);
        // Set a couple of board cells so BLUE has SOME presence (needed for non-first-move branch).
        *state.board.get_mut(10, 10) = FieldContent::Blue;
        // Make turn = 4 so current_color is BLUE again (4 % 4 == 0).
        state.turn = 4;
        // Confirm is_first_move_for(BLUE) is false, so we go down the normal branch.
        assert!(!state.is_first_move_for(Color::Blue));
        // No undeployed pieces left → possible_moves should be empty → sensible_moves returns Skip.
        let moves = socha::internal::sensible_moves(&state);
        assert_eq!(moves.len(), 1);
        assert!(matches!(moves[0], Move::Skip { color: Color::Blue }));
    }

    #[test]
    fn outgoing_move_xml_handles_rotated_and_flipped_piece() {
        // A rotated/mirrored piece must serialise its rotation + isFlipped attributes correctly.
        use socha::neutral::{Move, Piece, Rotation};
        let piece = Piece::new(
            Color::Green,
            PieceShape::PentoX,
            Rotation::Mirror,
            true,
            Coordinates::new(7, 3),
        );
        let mv = Move::Set { piece };
        let xml = socha::outgoing::make_move_xml("room-42", &mv).unwrap();
        let expected = "<room roomId=\"room-42\"><data class=\"sc.plugin2027.SetMove\"><piece color=\"GREEN\" kind=\"PENTO_X\" rotation=\"MIRROR\" isFlipped=\"true\"><position x=\"7\" y=\"3\"/></piece></data></room>";
        assert_eq!(xml, expected);
    }
}
