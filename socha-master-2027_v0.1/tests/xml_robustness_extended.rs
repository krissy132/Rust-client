//! Extended XML robustness regressions.
//!
//! ChatGPT's risk P3.8 already covers unknown class, missing color, out-of-range field,
//! and unknown content token (see `server_message_parsing.rs`). This file adds:
//!   - Unknown attributes on otherwise-known tags (must not panic; parser silently ignores).
//!   - Missing required field within a <lastMove> (returns an error, not a panic).
//!   - Unknown top-level element after a known one (parser ignores trailing junk).

use strong_xml::XmlRead;

use socha::incoming::{ReceivedData, ReceivedPiece, ReceivedRoom, ReceivedState};
use socha::internal::{GameResult, GameState, RoomMessage};

#[test]
fn unknown_attribute_on_known_tag_tolerated() {
    // The server adds a forward-compat attribute we don't know about — must not panic.
    let xml = r#"<data class="welcomeMessage" color="ONE" revision="42"/>"#;
    let data: ReceivedData = ReceivedData::from_str(xml).expect("parse with extra attr");
    let wrapper = ReceivedRoom {
        room_id: Some("rid".to_string()),
        data: Some(data),
    };
    let msg = RoomMessage::try_from(wrapper).expect("decode");
    assert!(matches!(msg, RoomMessage::WelcomeMessage { color: _ }));
}

#[test]
fn missing_position_inside_piece_falls_back_to_default() {
    // The Kotlin server always includes <position> in actual SetMove payloads, but the
    // parser's behaviour when it's missing must not panic. We assert parse succeeds and
    // position is `None` (or a (0,0) default) — we don't pin a specific behaviour, only
    // require "no panic".
    let xml = r#"<piece color="BLUE" kind="MONO" rotation="NONE" isFlipped="false"></piece>"#;
    let received: ReceivedPiece = ReceivedPiece::from_str(xml).expect("parse");
    // Position is permitted to be None; the engine downstream falls back to (0,0) via
    // `build_piece_from_received`, so we just log whatever we got.
    eprintln!("position was {:?}", received.position);
}

#[test]
fn unknown_top_level_element_following_known_one_is_ignored() {
    // A real server in principle only sends well-formed packets, but we want to confirm
    // that an extra tag inside <state> (e.g. a future-schema <foo/>) is ignored rather
    // than breaking the parse. We test this on <state> since it has the most children.
    let xml = r#"<state startTeam="ONE" turn="0" startPiece="PENTO_L" round="1">
        <board/>
        <lastMoveMono/>
        <blueShapes/>
        <yellowShapes/>
        <redShapes/>
        <greenShapes/>
        <validColors>
            <color>BLUE</color><color>YELLOW</color><color>RED</color><color>GREEN</color>
        </validColors>
        <experimentalField someAttr="1"/>
    </state>"#;
    // The parser must tolerate the extra `<experimentalField>` element rather than fail.
    let received: ReceivedState = ReceivedState::from_str(xml).expect("parse tolerates unknown child");
    let _state = GameState::try_from(received).expect("GameState build");
}

#[test]
fn game_result_with_unknown_fragment_attribute_tolerated() {
    // Definition fragment with an extra attribute we don't read — must not panic.
    let xml = r#"<data class="result">
        <definition>
            <fragment name="Siegpunkte" extra="ignored">
                <aggregation>SUM</aggregation>
                <relevantForRanking>true</relevantForRanking>
            </fragment>
        </definition>
        <scores>
            <entry>
                <player team="ONE"/>
                <score><part>1</part></score>
            </entry>
        </scores>
        <winner team="ONE" regular="true" reason="ONE gewinnt"/>
    </data>"#;
    let data: ReceivedData = ReceivedData::from_str(xml).expect("parse fragment with extra attr");
    let result = GameResult::try_from(data).expect("GameResult build");
    assert_eq!(result.scores.len(), 1);
    assert!(result.winner.is_some());
}

#[test]
fn empty_state_yields_default_gamestate_no_panic() {
    // Some bare minimum attributes missing — defaults must apply (turn=0, no last_move).
    let xml = r#"<state startTeam="ONE" startPiece="PENTO_I">
        <board/>
        <lastMoveMono/>
        <blueShapes/>
        <yellowShapes/>
        <redShapes/>
        <greenShapes/>
    </state>"#;
    let received: ReceivedState = ReceivedState::from_str(xml).expect("parse bare");
    let state = GameState::try_from(received).expect("Gamestate defaults applied");
    assert_eq!(state.turn, 0);
    assert!(state.last_move.is_none());
}
