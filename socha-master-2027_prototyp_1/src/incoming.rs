#![allow(clippy::needless_late_init, unused_must_use)]
// allowing needless_late_init due to warnings coming from macros inside the StrongXml crate
use strong_xml::{XmlRead, XmlWrite};

// ___ Board (custom converter: only non-empty fields are emitted) ___

#[derive(Debug, XmlRead, XmlWrite, Clone, PartialEq, Eq)]
#[xml(tag = "field")]
pub struct ReceivedField {
    #[xml(attr = "x")]
    pub x: i32,
    #[xml(attr = "y")]
    pub y: i32,
    #[xml(attr = "content")]
    pub content: String,
}

#[derive(Debug, XmlRead, XmlWrite, Clone, PartialEq, Eq)]
#[xml(tag = "board")]
pub struct ReceivedBoard {
    #[xml(child = "field")]
    pub fields: Vec<ReceivedField>,
}

// ___ Piece + position ___

#[derive(Debug, XmlRead, XmlWrite, Clone, PartialEq, Eq)]
#[xml(tag = "position")]
pub struct ReceivedPosition {
    #[xml(attr = "x")]
    pub x: i32,
    #[xml(attr = "y")]
    pub y: i32,
}

#[derive(Debug, XmlRead, XmlWrite, Clone, PartialEq, Eq)]
#[xml(tag = "piece")]
pub struct ReceivedPiece {
    #[xml(attr = "color")]
    pub color: String,
    #[xml(attr = "kind")]
    pub kind: String,
    #[xml(attr = "rotation")]
    pub rotation: String,
    #[xml(attr = "isFlipped")]
    pub is_flipped: String,
    #[xml(child = "position")]
    pub position: Option<ReceivedPosition>,
}

// ___ lastMove + color (SkipMove encloses <color>BLUE</color>) ___

#[derive(Debug, XmlRead, XmlWrite, Clone, PartialEq, Eq)]
#[xml(tag = "color")]
pub struct ReceivedColorText {
    #[xml(text)]
    pub value: String,
}

#[derive(Debug, XmlRead, XmlWrite, Clone, PartialEq, Eq)]
#[xml(tag = "lastMove")]
pub struct ReceivedLastMove {
    #[xml(attr = "class")]
    pub class: Option<String>,
    #[xml(child = "piece")]
    pub piece: Option<ReceivedPiece>,
    #[xml(child = "color")]
    pub color: Option<ReceivedColorText>,
}

// ___ lastMoveMono (Map serialised as <entry><color>X</color><boolean>true</boolean></entry>) ___

#[derive(Debug, XmlRead, XmlWrite, Clone, PartialEq, Eq)]
#[xml(tag = "boolean")]
pub struct ReceivedBoolean {
    #[xml(text)]
    pub value: String,
}

#[derive(Debug, XmlRead, XmlWrite, Clone, PartialEq, Eq)]
#[xml(tag = "entry")]
pub struct ReceivedLastMoveMonoEntry {
    #[xml(child = "color")]
    pub color: ReceivedColorText,
    #[xml(child = "boolean")]
    pub boolean: ReceivedBoolean,
}

#[derive(Debug, XmlRead, XmlWrite, Clone, PartialEq, Eq, Default)]
#[xml(tag = "lastMoveMono")]
pub struct ReceivedLastMoveMono {
    #[xml(child = "entry")]
    pub entries: Vec<ReceivedLastMoveMonoEntry>,
}

// ___ Undeployed-shape lists: <blueShapes><shape>MONO</shape>...</blueShapes> ___

#[derive(Debug, XmlRead, XmlWrite, Clone, PartialEq, Eq)]
#[xml(tag = "shape")]
pub struct ReceivedShape {
    #[xml(text)]
    pub value: String,
}

#[derive(Debug, XmlRead, XmlWrite, Clone, PartialEq, Eq, Default)]
#[xml(tag = "blueShapes")]
pub struct ReceivedBlueShapes {
    #[xml(child = "shape")]
    pub shapes: Vec<ReceivedShape>,
}

#[derive(Debug, XmlRead, XmlWrite, Clone, PartialEq, Eq, Default)]
#[xml(tag = "yellowShapes")]
pub struct ReceivedYellowShapes {
    #[xml(child = "shape")]
    pub shapes: Vec<ReceivedShape>,
}

#[derive(Debug, XmlRead, XmlWrite, Clone, PartialEq, Eq, Default)]
#[xml(tag = "redShapes")]
pub struct ReceivedRedShapes {
    #[xml(child = "shape")]
    pub shapes: Vec<ReceivedShape>,
}

#[derive(Debug, XmlRead, XmlWrite, Clone, PartialEq, Eq, Default)]
#[xml(tag = "greenShapes")]
pub struct ReceivedGreenShapes {
    #[xml(child = "shape")]
    pub shapes: Vec<ReceivedShape>,
}

// ___ validColors: <validColors><color>BLUE</color>...</validColors> ___

#[derive(Debug, XmlRead, XmlWrite, Clone, PartialEq, Eq, Default)]
#[xml(tag = "validColors")]
pub struct ReceivedValidColors {
    #[xml(child = "color")]
    pub colors: Vec<ReceivedColorText>,
}

// ___ <state> rich element ___

#[derive(Debug, XmlRead, XmlWrite, Clone, PartialEq, Eq, Default)]
#[xml(tag = "state")]
pub struct ReceivedState {
    #[xml(attr = "turn")]
    pub turn: Option<u32>,
    #[xml(attr = "startTeam")]
    pub start_team: Option<String>,
    #[xml(attr = "startPiece")]
    pub start_piece: Option<String>,
    #[xml(attr = "round")]
    pub round: Option<u32>,
    #[xml(child = "lastMove")]
    pub last_move: Option<ReceivedLastMove>,
    #[xml(child = "board")]
    pub board: Option<ReceivedBoard>,
    #[xml(child = "lastMoveMono")]
    pub last_move_mono: Option<ReceivedLastMoveMono>,
    #[xml(child = "blueShapes")]
    pub blue_shapes: Option<ReceivedBlueShapes>,
    #[xml(child = "yellowShapes")]
    pub yellow_shapes: Option<ReceivedYellowShapes>,
    #[xml(child = "redShapes")]
    pub red_shapes: Option<ReceivedRedShapes>,
    #[xml(child = "greenShapes")]
    pub green_shapes: Option<ReceivedGreenShapes>,
    #[xml(child = "validColors")]
    pub valid_colors: Option<ReceivedValidColors>,
}

// ___ GameResult XML (matches GameResultConverter + GameResultTest.kt) ___

#[derive(Debug, XmlRead, XmlWrite, Clone, PartialEq, Eq)]
#[xml(tag = "aggregation")]
pub struct ReceivedAggregation {
    #[xml(text)]
    pub value: String,
}

#[derive(Debug, XmlRead, XmlWrite, Clone, PartialEq, Eq)]
#[xml(tag = "relevantForRanking")]
pub struct ReceivedBool {
    #[xml(text)]
    pub value: String,
}

#[derive(Debug, XmlRead, XmlWrite, Clone, PartialEq, Eq)]
#[xml(tag = "fragment")]
pub struct ReceivedFragment {
    #[xml(attr = "name")]
    pub name: String,
    #[xml(child = "aggregation")]
    pub aggregation: ReceivedAggregation,
    #[xml(child = "relevantForRanking")]
    pub relevant_for_ranking: ReceivedBool,
}

#[derive(Debug, XmlRead, XmlWrite, Clone, PartialEq, Eq)]
#[xml(tag = "definition")]
pub struct ReceivedDefinition {
    #[xml(child = "fragment")]
    pub fragments: Vec<ReceivedFragment>,
}

#[derive(Debug, XmlRead, XmlWrite, Clone, PartialEq, Eq)]
#[xml(tag = "player")]
pub struct ReceivedPlayerAttr {
    #[xml(attr = "team")]
    pub team: String,
}

#[derive(Debug, XmlRead, XmlWrite, Clone, PartialEq, Eq)]
#[xml(tag = "part")]
pub struct ReceivedPart {
    #[xml(text)]
    pub value: String,
}

#[derive(Debug, XmlRead, XmlWrite, Clone, PartialEq, Eq)]
#[xml(tag = "score")]
pub struct ReceivedScore {
    #[xml(child = "part")]
    pub parts: Vec<ReceivedPart>,
}

#[derive(Debug, XmlRead, XmlWrite, Clone, PartialEq, Eq)]
#[xml(tag = "entry")]
pub struct ReceivedScoreEntry {
    #[xml(child = "player")]
    pub player: ReceivedPlayerAttr,
    #[xml(child = "score")]
    pub score: Option<ReceivedScore>,
}

#[derive(Debug, XmlRead, XmlWrite, Clone, PartialEq, Eq)]
#[xml(tag = "scores")]
pub struct ReceivedScores {
    #[xml(child = "entry")]
    pub entries: Vec<ReceivedScoreEntry>,
}

#[derive(Debug, XmlRead, XmlWrite, Clone, PartialEq, Eq)]
#[xml(tag = "winner")]
pub struct ReceivedWinner {
    #[xml(attr = "team")]
    pub team: Option<String>,
    #[xml(attr = "regular")]
    pub regular: Option<String>,
    #[xml(attr = "reason")]
    pub reason: Option<String>,
}

// ___ <data class="..."> envelope (memento / result / welcomeMessage / moveRequest) ___

#[derive(Debug, XmlRead, XmlWrite, Clone, PartialEq, Eq)]
#[xml(tag = "data")]
pub struct ReceivedData {
    #[xml(attr = "class")]
    pub class: Option<String>,
    /// welcomeMessage uses a `color` attribute (Team name string).
    #[xml(attr = "color")]
    pub color_attr: Option<String>,
    /// `<state>` from memento.
    #[xml(child = "state")]
    pub state: Option<ReceivedState>,
    /// GameResult children:
    #[xml(child = "definition")]
    pub definition: Option<ReceivedDefinition>,
    #[xml(child = "scores")]
    pub scores: Option<ReceivedScores>,
    #[xml(child = "winner")]
    pub winner: Option<ReceivedWinner>,
}

#[derive(Debug, XmlRead, XmlWrite, Clone, PartialEq, Eq)]
#[xml(tag = "room")]
pub struct ReceivedRoom {
    #[xml(attr = "roomId")]
    pub room_id: Option<String>,
    #[xml(child = "data")]
    pub data: Option<ReceivedData>,
}

#[derive(Debug, XmlRead, XmlWrite, Clone, PartialEq, Eq)]
#[xml(tag = "joined")]
pub struct ReceivedJoined {
    #[xml(attr = "roomId")]
    pub room_id: Option<String>,
}

#[derive(Debug, XmlRead, XmlWrite, Clone, PartialEq, Eq)]
#[xml(tag = "left")]
pub struct ReceivedLeft {
    #[xml(attr = "roomId")]
    pub room_id: Option<String>,
}

#[derive(Debug, XmlRead, XmlWrite, Clone, PartialEq, Eq)]
#[xml(tag = "comMessage")]
pub struct ReceivedComMessage {
    #[xml(child = "left")]
    pub left: Option<ReceivedLeft>,
    #[xml(child = "joined")]
    pub joined: Option<ReceivedJoined>,
    #[xml(child = "room")]
    pub room: Vec<ReceivedRoom>,
    #[xml(child = "prepared")]
    pub admin_prepared: Option<ReceivedAdminPrepared>,
}

// ___ admin prepared ___

#[derive(Debug, XmlRead, XmlWrite, Clone, PartialEq, Eq)]
#[xml(tag = "prepared")]
pub struct ReceivedAdminPrepared {
    #[xml(child = "reservation")]
    pub admin_reservation: Vec<ReceivedAdminReservation>,
    #[xml(attr = "roomId")]
    pub room_id: String,
}

#[derive(Debug, XmlRead, XmlWrite, Clone, PartialEq, Eq)]
#[xml(tag = "reservation")]
pub struct ReceivedAdminReservation {
    #[xml(text)]
    pub reservation_id: String,
}
