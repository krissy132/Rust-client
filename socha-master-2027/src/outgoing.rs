#![allow(clippy::needless_late_init, unused_must_use)]
// allowing needless_late_init due to warnings coming from macros inside the StrongXml crate
use std::error::Error;

use strong_xml::{XmlRead, XmlWrite};

use crate::neutral::{Color, Move, Piece};
use crate::socha_com::PrepareSlot;

// ___ join ___

#[derive(Debug, XmlWrite, XmlRead)]
#[xml(tag = "join")]
pub struct Join {
    #[xml(attr = "gameType")]
    pub game_type: String,
    #[xml(attr = "participantId")]
    pub participant_id: Option<String>,
}

#[derive(Debug, XmlWrite, XmlRead)]
#[xml(tag = "joinPrepared")]
pub struct JoinPrepared {
    #[xml(attr = "reservationCode")]
    pub reservation_code: String,
}

pub fn make_join_xml(
    game_type: &str,
    participant_id: Option<&str>,
) -> Result<String, Box<dyn Error>> {
    let join = Join {
        game_type: game_type.to_string(),
        participant_id: participant_id.map(str::to_string),
    };
    Ok(join.to_string()?)
}

pub fn make_join_prepared_xml(reservation_code: &str) -> Result<String, Box<dyn Error>> {
    let join_prepared = JoinPrepared {
        reservation_code: reservation_code.to_string(),
    };
    Ok(join_prepared.to_string()?)
}

pub fn make_close_xml() -> String {
    "<close></close>".to_string()
}

// ___ move (manual string building — use FULL class names, NOT short aliases, as server requires) ___

/// Build the `<room roomId="..."><data class="...">...</data></room>` XML for a Move.
pub fn make_move_xml(room_id: &str, mv: &Move) -> Result<String, Box<dyn Error>> {
    let body = match mv {
        Move::Set { piece } => set_move_body(piece),
        Move::Skip { color } => skip_move_body(*color),
    };
    Ok(format!("<room roomId=\"{room_id}\">{body}</room>"))
}

fn set_move_body(piece: &Piece) -> String {
    let color = piece.color;
    let kind = piece.kind;
    let rotation = piece.rotation;
    let flipped = piece.is_flipped;
    let px = piece.position.x;
    let py = piece.position.y;
    // Use FULL class name "sc.plugin2027.SetMove" — the server's XStream cannot resolve short aliases
    format!(
        "<data class=\"sc.plugin2027.SetMove\"><piece color=\"{color}\" kind=\"{kind}\" rotation=\"{rotation}\" isFlipped=\"{flipped}\"><position x=\"{px}\" y=\"{py}\"/></piece></data>"
    )
}

fn skip_move_body(color: Color) -> String {
    // Use FULL class name "sc.plugin2027.SkipMove"
    format!("<data class=\"sc.plugin2027.SkipMove\"><color>{color}</color></data>")
}

// ___ auth ___

#[derive(Debug, XmlWrite, XmlRead)]
#[xml(tag = "authenticate")]
pub struct Authenticate {
    #[xml(attr = "password")]
    pub password: String,
}

#[derive(Debug, XmlWrite, XmlRead)]
#[xml(tag = "observe")]
pub struct Observe {
    #[xml(attr = "roomId")]
    pub room_id: String,
}

#[derive(Debug, XmlWrite, XmlRead)]
#[xml(tag = "pause")]
pub struct Pause {
    #[xml(attr = "roomId")]
    pub room_id: String,
    // use string "true"/"false" to exactly match docs
    #[xml(attr = "pause")]
    pub pause: String,
}

#[derive(Debug, XmlWrite, XmlRead)]
#[xml(tag = "step")]
pub struct Step {
    #[xml(attr = "roomId")]
    pub room_id: String,
}

#[derive(Debug, XmlWrite, XmlRead)]
#[xml(tag = "cancel")]
pub struct Cancel {
    #[xml(attr = "roomId")]
    pub room_id: String,
}

#[derive(Debug, XmlWrite, XmlRead)]
#[xml(tag = "slot")]
pub struct Slot {
    #[xml(attr = "displayName")]
    pub display_name: String,
    // "true"/"false" strings to match doc examples
    #[xml(attr = "canTimeout")]
    pub can_timeout: String,
    #[xml(attr = "reserved")]
    pub reserved: String,
}

#[derive(Debug, XmlWrite, XmlRead)]
#[xml(tag = "prepare")]
pub struct Prepare {
    #[xml(attr = "gameType")]
    pub game_type: String,
    #[xml(attr = "pause")]
    pub pause: String,
    #[xml(child = "slot")]
    pub slots: Vec<Slot>,
}

pub fn make_authenticate_xml(password: &str) -> Result<String, Box<dyn Error>> {
    let auth = Authenticate {
        password: password.to_string(),
    };
    Ok(auth.to_string()?)
}

pub fn make_observe_xml(room_id: &str) -> Result<String, Box<dyn Error>> {
    let o = Observe {
        room_id: room_id.to_string(),
    };
    Ok(o.to_string()?)
}

pub fn make_pause_xml(room_id: &str, pause: bool) -> Result<String, Box<dyn Error>> {
    let p = Pause {
        room_id: room_id.to_string(),
        pause: if pause { "true".into() } else { "false".into() },
    };
    Ok(p.to_string()?)
}

pub fn make_step_xml(room_id: &str) -> Result<String, Box<dyn Error>> {
    let s = Step {
        room_id: room_id.to_string(),
    };
    Ok(s.to_string()?)
}

pub fn make_cancel_xml(room_id: &str) -> Result<String, Box<dyn Error>> {
    let c = Cancel {
        room_id: room_id.to_string(),
    };
    Ok(c.to_string()?)
}

/// `slots` is a slice of tuples: (display_name, can_timeout, reserved)
pub fn make_prepare_xml(
    game_type: &str,
    pause: bool,
    slots: &[PrepareSlot],
) -> Result<String, Box<dyn Error>> {
    let slots_vec = slots
        .iter()
        .map(|prep_slot| Slot {
            display_name: prep_slot.displayname.to_string(),
            can_timeout: if prep_slot.can_timeout {
                "true".into()
            } else {
                "false".into()
            },
            reserved: if prep_slot.reserved {
                "true".into()
            } else {
                "false".into()
            },
        })
        .collect();
    let p = Prepare {
        game_type: game_type.to_string(),
        pause: if pause { "true".into() } else { "false".into() },
        slots: slots_vec,
    };
    Ok(p.to_string()?)
}

// (kind/rotation kept for `set_move_body` via `Display` impls through format-string positionals).
