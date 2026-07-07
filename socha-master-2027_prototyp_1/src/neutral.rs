//! Domain enums and structs for the 2027 Blokus plugin (sc.plugin2027).
//!
//! These mirror the official Kotlin source file-for-file:
//! - `Color`              <- `Color.kt`
//! - `Team`               <- `sc.api.plugins.Team`
//! - `FieldContent`       <- `FieldContent.kt`
//! - `Rotation`           <- `Rototation.kt`
//! - `PieceShape`         <- `PieceShape.kt` (21 shapes, exact coordinates)
//! - `Piece`              <- `Piece.kt`
//! - `Move`               <- `Move.kt` (SetMove / SkipMove)
//! - `BlokusMoveMistake`  <- `BlokusMoveMistake.kt` (9 reasons)
//! - `ScoreAggregation` / `ScoreFragment` <- `sc.shared.*`
//! - `Coordinates`        <- `sc.api.plugins.Coordinates`

use std::collections::HashMap;
use std::fmt;

use std::str::FromStr;

/// Cartesian 2D coordinate (x, y). y=0 is the top row, y increases downward.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Coordinates {
    pub x: i32,
    pub y: i32,
}

impl Coordinates {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
    pub const fn offset(self, dx: i32, dy: i32) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
        }
    }

    /// The 4 cardinal neighbours (up/down/left/right).
    pub fn neighbors(self) -> [Coordinates; 4] {
        [
            self.offset(0, -1),
            self.offset(0, 1),
            self.offset(-1, 0),
            self.offset(1, 0),
        ]
    }

    /// The 4 diagonal neighbours.
    pub fn diagonal_neighbors(self) -> [Coordinates; 4] {
        [
            self.offset(-1, -1),
            self.offset(1, -1),
            self.offset(-1, 1),
            self.offset(1, 1),
        ]
    }
}

impl fmt::Display for Coordinates {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}|{}]", self.x, self.y)
    }
}

/// The four colours in Blokus. Turn order: Blue -> Yellow -> Red -> Green.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    Blue,
    Yellow,
    Red,
    Green,
}

impl Color {
    pub const ALL: [Color; 4] = [Color::Blue, Color::Yellow, Color::Red, Color::Green];

    pub fn next(self) -> Color {
        match self {
            Color::Blue => Color::Yellow,
            Color::Yellow => Color::Red,
            Color::Red => Color::Green,
            Color::Green => Color::Blue,
        }
    }

    pub fn team(self) -> Team {
        match self {
            Color::Blue | Color::Red => Team::One,
            Color::Yellow | Color::Green => Team::Two,
        }
    }

    pub fn to_field_content(self) -> FieldContent {
        match self {
            Color::Blue => FieldContent::Blue,
            Color::Yellow => FieldContent::Yellow,
            Color::Red => FieldContent::Red,
            Color::Green => FieldContent::Green,
        }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Color::Blue => "BLUE",
            Color::Yellow => "YELLOW",
            Color::Red => "RED",
            Color::Green => "GREEN",
        };
        f.write_str(s)
    }
}

impl FromStr for Color {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "BLUE" => Ok(Color::Blue),
            "YELLOW" => Ok(Color::Yellow),
            "RED" => Ok(Color::Red),
            "GREEN" => Ok(Color::Green),
            other => Err(format!("unknown color token '{other}'")),
        }
    }
}

/// The two teams: One = Blue+Red, Two = Yellow+Green.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Team {
    One,
    Two,
}

impl Team {
    pub fn opponent(self) -> Team {
        match self {
            Team::One => Team::Two,
            Team::Two => Team::One,
        }
    }
}

impl fmt::Display for Team {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Team::One => "ONE",
            Team::Two => "TWO",
        })
    }
}

impl FromStr for Team {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "ONE" => Ok(Team::One),
            "TWO" => Ok(Team::Two),
            other => Err(format!("unknown team token '{other}'")),
        }
    }
}

/// Empty or one of the four colours, as stored in a board cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FieldContent {
    #[default]
    Empty,
    Blue,
    Yellow,
    Red,
    Green,
}

impl FieldContent {
    pub fn letter(self) -> char {
        match self {
            FieldContent::Blue => 'B',
            FieldContent::Yellow => 'Y',
            FieldContent::Red => 'R',
            FieldContent::Green => 'G',
            FieldContent::Empty => '-',
        }
    }

    pub fn to_color(self) -> Option<Color> {
        match self {
            FieldContent::Blue => Some(Color::Blue),
            FieldContent::Yellow => Some(Color::Yellow),
            FieldContent::Red => Some(Color::Red),
            FieldContent::Green => Some(Color::Green),
            FieldContent::Empty => None,
        }
    }

    pub fn is_empty(self) -> bool {
        matches!(self, FieldContent::Empty)
    }
}

impl fmt::Display for FieldContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            FieldContent::Blue => "BLUE",
            FieldContent::Yellow => "YELLOW",
            FieldContent::Red => "RED",
            FieldContent::Green => "GREEN",
            FieldContent::Empty => "EMPTY",
        })
    }
}

impl FromStr for FieldContent {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "BLUE" => Ok(FieldContent::Blue),
            "YELLOW" => Ok(FieldContent::Yellow),
            "RED" => Ok(FieldContent::Red),
            "GREEN" => Ok(FieldContent::Green),
            "EMPTY" => Ok(FieldContent::Empty),
            other => Err(format!("unknown field content token '{other}'")),
        }
    }
}

/// Rotation amount for a PieceShape (Kotlin `Rotation`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Rotation {
    #[default]
    None,
    Right,
    /// 180 degrees.
    Mirror,
    Left,
}

impl Rotation {
    pub fn value(self) -> u8 {
        match self {
            Rotation::None => 0,
            Rotation::Right => 1,
            Rotation::Mirror => 2,
            Rotation::Left => 3,
        }
    }

    pub fn combine(self, other: Rotation) -> Rotation {
        const ALL: [Rotation; 4] = [
            Rotation::None,
            Rotation::Right,
            Rotation::Mirror,
            Rotation::Left,
        ];
        ALL[(self.value() as usize + other.value() as usize) % 4]
    }
}

impl fmt::Display for Rotation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Rotation::None => "NONE",
            Rotation::Right => "RIGHT",
            Rotation::Mirror => "MIRROR",
            Rotation::Left => "LEFT",
        })
    }
}

impl FromStr for Rotation {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "NONE" => Ok(Rotation::None),
            "RIGHT" => Ok(Rotation::Right),
            "MIRROR" => Ok(Rotation::Mirror),
            "LEFT" => Ok(Rotation::Left),
            other => Err(format!("unknown rotation token '{other}'")),
        }
    }
}

/// All 21 piece shapes of Blokus. Variants are in the same ordinal order as the
/// Kotlin `PieceShape` enum (indices 0..21). The `coordinates()` of each shape
/// are the aligned (top-left normalized) coordinate set transcribed verbatim from
/// `PieceShape.kt` lines 14-34 (which itself calls `.align()`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PieceShape {
    Mono = 0,
    Domino = 1,
    TrioL = 2,
    TrioI = 3,
    TetroO = 4,
    TetroT = 5,
    TetroI = 6,
    TetroL = 7,
    TetroZ = 8,
    PentoL = 9,
    PentoT = 10,
    PentoV = 11,
    PentoS = 12,
    PentoZ = 13,
    PentoI = 14,
    PentoP = 15,
    PentoW = 16,
    PentoU = 17,
    PentoR = 18,
    PentoX = 19,
    PentoY = 20,
}

/// A single de-duplicated variant of a [`PieceShape`]: which (rotation, is_flipped)
/// produced the contained aligned shape coordinate set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShapeVariant {
    pub rotation: Rotation,
    pub is_flipped: bool,
    pub shape: Vec<(i32, i32)>,
}

impl PieceShape {
    /// All 21 shapes in their canonical ordinal order.
    pub const ALL: [PieceShape; 21] = [
        PieceShape::Mono,
        PieceShape::Domino,
        PieceShape::TrioL,
        PieceShape::TrioI,
        PieceShape::TetroO,
        PieceShape::TetroT,
        PieceShape::TetroI,
        PieceShape::TetroL,
        PieceShape::TetroZ,
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

    pub const TOTAL: usize = 21;

    /// Canonical aligned coordinate set of this shape (without rotation/flip).
    /// Transcribed verbatim from `PieceShape.kt` — every set is already `.align()`-ed.
    pub fn coordinates(self) -> &'static [(i32, i32)] {
        match self {
            // 0: MONO {(0,0)}
            PieceShape::Mono => &[(0, 0)],
            // 1: DOMINO {(0,0),(1,0)}
            PieceShape::Domino => &[(0, 0), (1, 0)],
            // 2: TRIO_L {(0,0),(0,1),(1,1)}
            PieceShape::TrioL => &[(0, 0), (0, 1), (1, 1)],
            // 3: TRIO_I {(0,0),(0,1),(0,2)}
            PieceShape::TrioI => &[(0, 0), (0, 1), (0, 2)],
            // 4: TETRO_O {(0,0),(1,0),(0,1),(1,1)}
            PieceShape::TetroO => &[(0, 0), (1, 0), (0, 1), (1, 1)],
            // 5: TETRO_T {(0,0),(1,0),(2,0),(1,1)}
            PieceShape::TetroT => &[(0, 0), (1, 0), (2, 0), (1, 1)],
            // 6: TETRO_I {(0,0),(0,1),(0,2),(0,3)}
            PieceShape::TetroI => &[(0, 0), (0, 1), (0, 2), (0, 3)],
            // 7: TETRO_L {(0,0),(0,1),(0,2),(1,2)}
            PieceShape::TetroL => &[(0, 0), (0, 1), (0, 2), (1, 2)],
            // 8: TETRO_Z {(0,0),(1,0),(1,1),(2,1)}
            PieceShape::TetroZ => &[(0, 0), (1, 0), (1, 1), (2, 1)],
            // 9: PENTO_L {(0,0),(0,1),(0,2),(0,3),(1,3)}
            PieceShape::PentoL => &[(0, 0), (0, 1), (0, 2), (0, 3), (1, 3)],
            // 10: PENTO_T {(0,0),(1,0),(2,0),(1,1),(1,2)}
            PieceShape::PentoT => &[(0, 0), (1, 0), (2, 0), (1, 1), (1, 2)],
            // 11: PENTO_V {(0,0),(0,1),(0,2),(1,2),(2,2)}
            PieceShape::PentoV => &[(0, 0), (0, 1), (0, 2), (1, 2), (2, 2)],
            // 12: PENTO_S {(1,0),(2,0),(3,0),(0,1),(1,1)}
            PieceShape::PentoS => &[(1, 0), (2, 0), (3, 0), (0, 1), (1, 1)],
            // 13: PENTO_Z {(0,0),(1,0),(1,1),(1,2),(2,2)}
            PieceShape::PentoZ => &[(0, 0), (1, 0), (1, 1), (1, 2), (2, 2)],
            // 14: PENTO_I {(0,0),(0,1),(0,2),(0,3),(0,4)}
            PieceShape::PentoI => &[(0, 0), (0, 1), (0, 2), (0, 3), (0, 4)],
            // 15: PENTO_P {(0,0),(1,0),(0,1),(1,1),(0,2)}
            PieceShape::PentoP => &[(0, 0), (1, 0), (0, 1), (1, 1), (0, 2)],
            // 16: PENTO_W {(0,0),(0,1),(1,1),(1,2),(2,2)}
            PieceShape::PentoW => &[(0, 0), (0, 1), (1, 1), (1, 2), (2, 2)],
            // 17: PENTO_U {(0,0),(0,1),(1,1),(2,1),(2,0)}
            PieceShape::PentoU => &[(0, 0), (0, 1), (1, 1), (2, 1), (2, 0)],
            // 18: PENTO_R {(0,1),(1,1),(1,2),(2,1),(2,0)}
            PieceShape::PentoR => &[(0, 1), (1, 1), (1, 2), (2, 1), (2, 0)],
            // 19: PENTO_X {(1,0),(0,1),(1,1),(2,1),(1,2)}
            PieceShape::PentoX => &[(1, 0), (0, 1), (1, 1), (2, 1), (1, 2)],
            // 20: PENTO_Y {(0,1),(1,0),(1,1),(1,2),(1,3)}
            PieceShape::PentoY => &[(0, 1), (1, 0), (1, 1), (1, 2), (1, 3)],
        }
    }

    pub fn size(self) -> usize {
        self.coordinates().len()
    }

    /// Returns true if this shape is a pentomino (size 5). Useful for the start-piece constraint.
    pub fn is_pentomino(self) -> bool {
        self.size() == 5
    }

    /// Width and height of this shape's bounding box (relative to the (0,0) anchor).
    pub fn bounding_box(self) -> (i32, i32) {
        let mut dx = 0;
        let mut dy = 0;
        for &(x, y) in self.coordinates() {
            if x > dx {
                dx = x;
            }
            if y > dy {
                dy = y;
            }
        }
        (dx, dy)
    }

    /// Return the shape transformed by `rotation` then (optionally) y-axis flip,
    /// then re-aligned so the top-left corner sits at (0,0). Matches the Kotlin
    /// `Set<Coordinates>.rotate(rotation).flip(shouldFlip)` plus implicit `.align()`.
    pub fn transform(self, rotation: Rotation, is_flipped: bool) -> Vec<(i32, i32)> {
        let base = self.coordinates();
        let rotated: Vec<(i32, i32)> = match rotation {
            Rotation::None => base.to_vec(),
            Rotation::Right => base.iter().map(|&(x, y)| (-y, x)).collect(),
            Rotation::Mirror => base.iter().map(|&(x, y)| (-x, -y)).collect(),
            Rotation::Left => base.iter().map(|&(x, y)| (y, -x)).collect(),
        };
        let flipped: Vec<(i32, i32)> = if is_flipped {
            rotated.iter().map(|&(x, y)| (-x, y)).collect()
        } else {
            rotated
        };
        align(&flipped)
    }

    /// Precomputed de-duplicated variants: each unique aligned shape, paired with
    /// the canonical (rotation, is_flipped) that produced it. Matches Kotlin's
    /// `PieceShape.variants` map semantics, in deterministic iteration order.
    pub fn variants(self) -> Vec<ShapeVariant> {
        let mut out: Vec<ShapeVariant> = Vec::new();
        let rotations = [
            Rotation::None,
            Rotation::Right,
            Rotation::Mirror,
            Rotation::Left,
        ];
        for r in rotations {
            for &flip in &[false, true] {
                let shape = self.transform(r, flip);
                if !out.iter().any(|v| v.shape == shape) {
                    out.push(ShapeVariant {
                        rotation: r,
                        is_flipped: flip,
                        shape,
                    });
                }
            }
        }
        out
    }
}

impl fmt::Display for PieceShape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            PieceShape::Mono => "MONO",
            PieceShape::Domino => "DOMINO",
            PieceShape::TrioL => "TRIO_L",
            PieceShape::TrioI => "TRIO_I",
            PieceShape::TetroO => "TETRO_O",
            PieceShape::TetroT => "TETRO_T",
            PieceShape::TetroI => "TETRO_I",
            PieceShape::TetroL => "TETRO_L",
            PieceShape::TetroZ => "TETRO_Z",
            PieceShape::PentoL => "PENTO_L",
            PieceShape::PentoT => "PENTO_T",
            PieceShape::PentoV => "PENTO_V",
            PieceShape::PentoS => "PENTO_S",
            PieceShape::PentoZ => "PENTO_Z",
            PieceShape::PentoI => "PENTO_I",
            PieceShape::PentoP => "PENTO_P",
            PieceShape::PentoW => "PENTO_W",
            PieceShape::PentoU => "PENTO_U",
            PieceShape::PentoR => "PENTO_R",
            PieceShape::PentoX => "PENTO_X",
            PieceShape::PentoY => "PENTO_Y",
        })
    }
}

impl FromStr for PieceShape {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "MONO" => Ok(PieceShape::Mono),
            "DOMINO" => Ok(PieceShape::Domino),
            "TRIO_L" => Ok(PieceShape::TrioL),
            "TRIO_I" => Ok(PieceShape::TrioI),
            "TETRO_O" => Ok(PieceShape::TetroO),
            "TETRO_T" => Ok(PieceShape::TetroT),
            "TETRO_I" => Ok(PieceShape::TetroI),
            "TETRO_L" => Ok(PieceShape::TetroL),
            "TETRO_Z" => Ok(PieceShape::TetroZ),
            "PENTO_L" => Ok(PieceShape::PentoL),
            "PENTO_T" => Ok(PieceShape::PentoT),
            "PENTO_V" => Ok(PieceShape::PentoV),
            "PENTO_S" => Ok(PieceShape::PentoS),
            "PENTO_Z" => Ok(PieceShape::PentoZ),
            "PENTO_I" => Ok(PieceShape::PentoI),
            "PENTO_P" => Ok(PieceShape::PentoP),
            "PENTO_W" => Ok(PieceShape::PentoW),
            "PENTO_U" => Ok(PieceShape::PentoU),
            "PENTO_R" => Ok(PieceShape::PentoR),
            "PENTO_X" => Ok(PieceShape::PentoX),
            "PENTO_Y" => Ok(PieceShape::PentoY),
            other => Err(format!("unknown piece shape token '{other}'")),
        }
    }
}

/// A single piece: a kind of shape, rotated and possibly flipped, anchored at `position`.
///
/// `position` is the top-left of the bounding box (matches Kotlin `Piece.position`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Piece {
    pub color: Color,
    pub kind: PieceShape,
    pub rotation: Rotation,
    pub is_flipped: bool,
    pub position: Coordinates,
}

impl Piece {
    pub fn new(
        color: Color,
        kind: PieceShape,
        rotation: Rotation,
        is_flipped: bool,
        position: Coordinates,
    ) -> Self {
        Self {
            color,
            kind,
            rotation,
            is_flipped,
            position,
        }
    }

    /// The transformed shape coordinates relative to (0,0).
    pub fn shape(&self) -> Vec<(i32, i32)> {
        self.kind.transform(self.rotation, self.is_flipped)
    }

    /// The actual board coordinates this piece will occupy.
    pub fn coordinates(&self) -> Vec<Coordinates> {
        let (px, py) = (self.position.x, self.position.y);
        self.shape()
            .iter()
            .map(|&(dx, dy)| Coordinates::new(px + dx, py + dy))
            .collect()
    }
}

/// A Blokus move: either place a piece (`Set`) or skip the turn (`Skip`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Move {
    Set { piece: Piece },
    Skip { color: Color },
}

impl Move {
    pub fn color(&self) -> Color {
        match self {
            Move::Set { piece } => piece.color,
            Move::Skip { color } => *color,
        }
    }
}

/// The 9 invalid-move reasons from `BlokusMoveMistake.kt`. Order matches the Kotlin enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlokusMoveMistake {
    WrongColor,
    NotOnBorder,
    NoSharedCorner,
    WrongShape,
    SkipFirstTurn,
    DuplicateShape,
    OutOfBounds,
    Obstructed,
    TouchesSameColor,
}

impl BlokusMoveMistake {
    pub fn message(self) -> &'static str {
        match self {
            BlokusMoveMistake::WrongColor => "Die Farbe des Zuges ist nicht an der Reihe",
            BlokusMoveMistake::NotOnBorder => "Der erste Zug muss an den Rand gesetzt werden",
            BlokusMoveMistake::NoSharedCorner => {
                "Alle Teile müssen ein vorheriges Teil gleicher Farbe über mindestens eine Ecke berühren"
            }
            BlokusMoveMistake::WrongShape => "Der erste Zug muss den festgelegten Spielstein setzen",
            BlokusMoveMistake::SkipFirstTurn => "Der erste Zug muss einen Stein setzen",
            BlokusMoveMistake::DuplicateShape => "Der gewählte Stein wurde bereits gesetzt",
            BlokusMoveMistake::OutOfBounds => "Der Spielstein passt nicht vollständig auf das Spielfeld",
            BlokusMoveMistake::Obstructed => "Der Spielstein würde eine andere Farbe überlagern",
            BlokusMoveMistake::TouchesSameColor => "Der Spielstein berührt ein Feld gleicher Farbe",
        }
    }
}

impl fmt::Display for BlokusMoveMistake {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message())
    }
}

impl FromStr for BlokusMoveMistake {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "WRONG_COLOR" => Ok(BlokusMoveMistake::WrongColor),
            "NOT_ON_BORDER" => Ok(BlokusMoveMistake::NotOnBorder),
            "NO_SHARED_CORNER" => Ok(BlokusMoveMistake::NoSharedCorner),
            "WRONG_SHAPE" => Ok(BlokusMoveMistake::WrongShape),
            "SKIP_FIRST_TURN" => Ok(BlokusMoveMistake::SkipFirstTurn),
            "DUPLICATE_SHAPE" => Ok(BlokusMoveMistake::DuplicateShape),
            "OUT_OF_BOUNDS" => Ok(BlokusMoveMistake::OutOfBounds),
            "OBSTRUCTED" => Ok(BlokusMoveMistake::Obstructed),
            "TOUCHES_SAME_COLOR" => Ok(BlokusMoveMistake::TouchesSameColor),
            other => Err(format!("unknown BlokusMoveMistake token '{other}'")),
        }
    }
}

/// How GameResult fragment values get aggregated across a match. Matches `ScoreAggregation.kt`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScoreAggregation {
    Sum,
    Average,
}

impl fmt::Display for ScoreAggregation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            ScoreAggregation::Sum => "SUM",
            ScoreAggregation::Average => "AVERAGE",
        })
    }
}

impl FromStr for ScoreAggregation {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "SUM" => Ok(ScoreAggregation::Sum),
            "AVERAGE" => Ok(ScoreAggregation::Average),
            other => Err(format!("unknown score aggregation token '{other}'")),
        }
    }
}

/// One row of the `ScoreDefinition` table for the game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoreFragment {
    pub name: String,
    pub aggregation: ScoreAggregation,
    pub relevant_for_ranking: bool,
}

/// A single player's numeric values, one per `ScoreFragment` of the definition.
#[derive(Debug, Clone, PartialEq)]
pub struct PlayerScore {
    pub parts: Vec<f64>,
}

/// Winner info for a finished game.
#[derive(Debug, Clone, PartialEq)]
pub struct Winner {
    pub team: Option<Team>,
    pub regular: bool,
    pub reason: Option<String>,
}

/// Re-align a set of coordinates so that the minimum x and minimum y are both 0.
fn align(coords: &[(i32, i32)]) -> Vec<(i32, i32)> {
    if coords.is_empty() {
        return Vec::new();
    }
    let mut min_x = i32::MAX;
    let mut min_y = i32::MAX;
    for &(x, y) in coords {
        if x < min_x {
            min_x = x;
        }
        if y < min_y {
            min_y = y;
        }
    }
    coords
        .iter()
        .map(|&(x, y)| (x - min_x, y - min_y))
        .collect()
}

/// Helper to look up a Color by its index in [`Color::ALL`].
pub fn color_at(index: usize) -> Color {
    Color::ALL[index % 4]
}

/// Convenience map alias used by `GameState::last_move_mono`.
pub type LastMoveMonoMap = HashMap<Color, bool>;
