//! Internal game-state and rules for the 2027 Blokus plugin.
//!
//! Faithful Rust port of the relevant Kotlin sources:
//! - `Board.kt`, `GameState.kt`, `GameRuleLogic.kt`, `Constants.kt`.
//!
//! The board is a simple 20×20 array of `FieldContent` (per the user's spec); no bitboards.

use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

use crate::incoming::{
    ReceivedBoard, ReceivedData, ReceivedRoom, ReceivedState, ReceivedValidColors,
};
use crate::neutral::{
    Color, Coordinates, FieldContent, LastMoveMonoMap, Move, Piece, PieceShape, Rotation, Team,
};

pub const BOARD_LENGTH: usize = 20;
pub const ROUND_LIMIT: u32 = 25;
pub const TOTAL_PIECE_SHAPES: usize = 21;
/// Mirrors `Constants.VALIDATE_MOVE` — when true, `make_move` validates before applying.
pub const VALIDATE_MOVE: bool = true;
/// Total squares across all 21 pieces: 1*1 + 1*2 + 2*3 + 5*4 + 12*5 = 89.
pub const SUM_MAX_SQUARES: u32 = 89;

// ___ Connection bookkeeping (unchanged concept from 2026er) ___

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Joined {
    pub room_id: String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
/// opposite of joined
pub struct Left {
    pub room_id: String,
}

// ___ Board: 20x20 array of FieldContent ___

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Row {
    pub fields: [FieldContent; BOARD_LENGTH],
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Board {
    /// rows[y][x] — y=0 is the top row, y increases downward.
    pub rows: [Row; BOARD_LENGTH],
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in &self.rows {
            for (i, field) in row.fields.iter().enumerate() {
                if i > 0 {
                    write!(f, " ")?;
                }
                write!(f, "{}", field.letter())?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Board {
    pub fn contains(x: i32, y: i32) -> bool {
        (0..BOARD_LENGTH as i32).contains(&x) && (0..BOARD_LENGTH as i32).contains(&y)
    }

    pub fn get(&self, x: usize, y: usize) -> &FieldContent {
        &self.rows[y].fields[x]
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> &mut FieldContent {
        &mut self.rows[y].fields[x]
    }

    pub fn is_obstructed(&self, pos: Coordinates) -> bool {
        if !Board::contains(pos.x, pos.y) {
            return false;
        }
        !self.rows[pos.y as usize].fields[pos.x as usize].is_empty()
    }

    pub fn is_empty(&self) -> bool {
        self.rows
            .iter()
            .all(|row| row.fields.iter().all(|f| f.is_empty()))
    }

    /// Whether any cardinal neighbour of `pos` has the given color. Out-of-bounds neighbours are ignored.
    pub fn borders_on_color(&self, pos: Coordinates, color: Color) -> bool {
        let content = color.to_field_content();
        pos.neighbors()
            .iter()
            .filter(|p| Board::contains(p.x, p.y))
            .any(|p| self.rows[p.y as usize].fields[p.x as usize] == content)
    }

    /// Whether any diagonal neighbour of `pos` has the given color. Out-of-bounds neighbours are ignored.
    pub fn corners_on_color(&self, pos: Coordinates, color: Color) -> bool {
        let content = color.to_field_content();
        pos.diagonal_neighbors()
            .iter()
            .filter(|p| Board::contains(p.x, p.y))
            .any(|p| self.rows[p.y as usize].fields[p.x as usize] == content)
    }

    pub fn colored_fields(&self, color: Color) -> Vec<Coordinates> {
        let content = color.to_field_content();
        let mut out = Vec::new();
        for y in 0..BOARD_LENGTH {
            for x in 0..BOARD_LENGTH {
                if self.rows[y].fields[x] == content {
                    out.push(Coordinates::new(x as i32, y as i32));
                }
            }
        }
        out
    }

    pub fn valid_fields(&self, color: Color) -> Vec<Coordinates> {
        let mut out: Vec<Coordinates> = Vec::new();
        for colored in self.colored_fields(color) {
            for corner in colored.diagonal_neighbors() {
                if !Board::contains(corner.x, corner.y) {
                    continue;
                }
                if !self.rows[corner.y as usize].fields[corner.x as usize].is_empty() {
                    continue;
                }
                let touches_same = corner
                    .neighbors()
                    .iter()
                    .filter(|p| Board::contains(p.x, p.y))
                    .any(|p| {
                        self.rows[p.y as usize].fields[p.x as usize] == color.to_field_content()
                    });
                if !touches_same {
                    out.push(corner);
                }
            }
        }
        out
    }
}

impl TryFrom<ReceivedBoard> for Board {
    type Error = String;
    fn try_from(recv: ReceivedBoard) -> Result<Self, Self::Error> {
        let mut board = Board::default();
        for f in recv.fields {
            let x = f.x;
            let y = f.y;
            if !Board::contains(x, y) {
                return Err(format!("board field out of bounds: x={x}, y={y}"));
            }
            let content = FieldContent::from_str(&f.content)?;
            board.rows[y as usize].fields[x as usize] = content;
        }
        Ok(board)
    }
}

// ___ GameState (full state mirror of the server) ___

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameState {
    pub turn: u32,
    pub start_piece: PieceShape,
    pub last_move: Option<Move>,
    pub board: Board,
    /// Set to true for a color when it has placed all its pieces. The boolean value records
    /// whether the Monomino was the *last* piece placed (relevant for the +5 bonus).
    pub last_move_mono: LastMoveMonoMap,
    /// Undeployed (still unplaced) shapes per color — initialised with all 21 shapes.
    pub blue_shapes: Vec<PieceShape>,
    pub yellow_shapes: Vec<PieceShape>,
    pub red_shapes: Vec<PieceShape>,
    pub green_shapes: Vec<PieceShape>,
    /// Colors that are still able to play. When a color cannot move anymore it is removed.
    pub valid_colors: Vec<Color>,
}

impl Default for GameState {
    fn default() -> Self {
        Self::new(PieceShape::PentoL)
    }
}

impl GameState {
    /// Construct a fresh game state with the given start-piece and all 21 shapes per color available.
    pub fn new(start_piece: PieceShape) -> Self {
        let all_shapes: Vec<PieceShape> = PieceShape::ALL.to_vec();
        GameState {
            turn: 0,
            start_piece,
            last_move: None,
            board: Board::default(),
            last_move_mono: HashMap::new(),
            blue_shapes: all_shapes.clone(),
            yellow_shapes: all_shapes.clone(),
            red_shapes: all_shapes.clone(),
            green_shapes: all_shapes,
            valid_colors: Color::ALL.to_vec(),
        }
    }

    pub fn current_color(&self) -> Color {
        Color::ALL[(self.turn as usize) % Color::ALL.len()]
    }

    pub fn current_team(&self) -> Team {
        self.current_color().team()
    }

    /// 1-based round number, matches Kotlin `GameState.roundFromTurn`.
    pub fn round(&self) -> u32 {
        1 + self.turn / Color::ALL.len() as u32
    }

    pub fn is_over(&self) -> bool {
        self.valid_colors.is_empty() || self.turn / 2 >= ROUND_LIMIT
    }

    pub fn undeployed(&self, color: Color) -> &Vec<PieceShape> {
        match color {
            Color::Blue => &self.blue_shapes,
            Color::Yellow => &self.yellow_shapes,
            Color::Red => &self.red_shapes,
            Color::Green => &self.green_shapes,
        }
    }

    pub fn undeployed_mut(&mut self, color: Color) -> &mut Vec<PieceShape> {
        match color {
            Color::Blue => &mut self.blue_shapes,
            Color::Yellow => &mut self.yellow_shapes,
            Color::Red => &mut self.red_shapes,
            Color::Green => &mut self.green_shapes,
        }
    }

    pub fn is_valid_color(&self, color: Color) -> bool {
        self.valid_colors.contains(&color)
    }

    /// The current color has all 21 shapes still available (first move for this color).
    /// Matches Kotlin `isFirstMove`.
    pub fn is_first_move_for(&self, color: Color) -> bool {
        self.undeployed(color).len() == TOTAL_PIECE_SHAPES
    }

    /// Advance the turn counter until we reach a color that is still in `valid_colors`.
    /// Returns `false` if no valid color remains.
    pub fn advance(&mut self) -> bool {
        if self.valid_colors.is_empty() {
            return false;
        }
        self.turn = self.turn.saturating_add(1);
        while !self.valid_colors.contains(&self.current_color()) {
            self.turn = self.turn.saturating_add(1);
            // Safety: extra ensures against wrapping at extreme turn values; if somehow no color matches, abort.
            if self.turn > (ROUND_LIMIT * 4 + 100) {
                return false;
            }
        }
        true
    }

    /// Remove the given color from `valid_colors` and advance. Matches `removeActiveColor`.
    pub fn remove_active_color(&mut self, color: Color) -> bool {
        self.valid_colors.retain(|c| *c != color);
        self.advance()
    }
}

impl TryFrom<ReceivedState> for GameState {
    type Error = String;
    fn try_from(recv: ReceivedState) -> Result<Self, Self::Error> {
        let turn = recv.turn.unwrap_or(0);

        let start_piece = recv
            .start_piece
            .as_deref()
            .map(PieceShape::from_str)
            .transpose()?
            .unwrap_or(PieceShape::PentoL);

        let board = recv
            .board
            .map(Board::try_from)
            .transpose()?
            .unwrap_or_default();

        let last_move = match recv.last_move.as_ref() {
            Some(lm) => parse_last_move(lm)?,
            None => None,
        };

        let mut last_move_mono = HashMap::new();
        if let Some(mono) = recv.last_move_mono {
            for entry in mono.entries {
                let color = Color::from_str(&entry.color.value)?;
                let value = entry.boolean.value.trim() == "true";
                last_move_mono.insert(color, value);
            }
        }

        let blue_shapes = parse_shapes(recv.blue_shapes.map(|s| s.shapes))?;
        let yellow_shapes = parse_shapes(recv.yellow_shapes.map(|s| s.shapes))?;
        let red_shapes = parse_shapes(recv.red_shapes.map(|s| s.shapes))?;
        let green_shapes = parse_shapes(recv.green_shapes.map(|s| s.shapes))?;

        let valid_colors = recv
            .valid_colors
            .map(|v: ReceivedValidColors| {
                v.colors
                    .iter()
                    .map(|c| Color::from_str(&c.value))
                    .collect::<Result<Vec<_>, _>>()
            })
            .transpose()?
            .unwrap_or_else(|| Color::ALL.to_vec());

        Ok(GameState {
            turn,
            start_piece,
            last_move,
            board,
            last_move_mono,
            blue_shapes,
            yellow_shapes,
            red_shapes,
            green_shapes,
            valid_colors,
        })
    }
}

fn parse_shapes(
    recv: Option<Vec<crate::incoming::ReceivedShape>>,
) -> Result<Vec<PieceShape>, String> {
    match recv {
        None => Ok(PieceShape::ALL.to_vec()),
        Some(shapes) => shapes
            .into_iter()
            .map(|s| PieceShape::from_str(&s.value))
            .collect(),
    }
}

fn build_piece_from_received(p: &crate::incoming::ReceivedPiece) -> Result<Piece, String> {
    let color = Color::from_str(&p.color)?;
    let kind = PieceShape::from_str(&p.kind)?;
    let rotation = Rotation::from_str(&p.rotation)?;
    let is_flipped = p.is_flipped.trim() == "true";
    let position = p
        .position
        .as_ref()
        .map(|pos| Coordinates::new(pos.x, pos.y))
        .unwrap_or(Coordinates::new(0, 0));
    Ok(Piece::new(color, kind, rotation, is_flipped, position))
}

/// Parse a `<lastMove class="...">...</lastMove>` element into a `Move`, if any.
fn parse_last_move(lm: &crate::incoming::ReceivedLastMove) -> Result<Option<Move>, String> {
    let class = lm.class.as_deref().unwrap_or("");
    if class.ends_with("SetMove") || class == "setmove" {
        if let Some(p) = lm.piece.as_ref() {
            let piece = build_piece_from_received(p)?;
            return Ok(Some(Move::Set { piece }));
        }
        Ok(None)
    } else if class.ends_with("SkipMove") || class == "skipmove" {
        if let Some(c) = lm.color.as_ref() {
            let color = Color::from_str(&c.value)?;
            return Ok(Some(Move::Skip { color }));
        }
        Ok(None)
    } else {
        Ok(None)
    }
}

// ___ Rule helpers (free-standing) ___

pub fn is_on_border(pos: Coordinates) -> bool {
    let l = BOARD_LENGTH as i32;
    pos.x == 0 || pos.y == 0 || pos.x == l - 1 || pos.y == l - 1
}

// ___ Validate (strict — returns the specific BlokusMoveMistake) ___

use crate::neutral::BlokusMoveMistake as BMM;

pub fn validate_set_move(state: &GameState, mv: &Piece) -> Result<(), BMM> {
    let color = mv.color;

    if color != state.current_color() {
        return Err(BMM::WrongColor);
    }

    if state.is_first_move_for(color) {
        if mv.kind != state.start_piece {
            return Err(BMM::WrongShape);
        }
    } else if !state.undeployed(color).contains(&mv.kind) {
        return Err(BMM::DuplicateShape);
    }

    let coords = mv.coordinates();
    for c in &coords {
        if !Board::contains(c.x, c.y) {
            return Err(BMM::OutOfBounds);
        }
        if state.board.is_obstructed(*c) {
            return Err(BMM::Obstructed);
        }
        if state.board.borders_on_color(*c, color) {
            return Err(BMM::TouchesSameColor);
        }
    }

    if state.is_first_move_for(color) {
        if !coords.iter().any(|c| is_on_border(*c)) {
            return Err(BMM::NotOnBorder);
        }
    } else if !coords
        .iter()
        .any(|c| state.board.corners_on_color(*c, color))
    {
        return Err(BMM::NoSharedCorner);
    }

    Ok(())
}

pub fn validate_skip_move(state: &GameState, color: Color) -> Result<(), BMM> {
    if color != state.current_color() {
        return Err(BMM::WrongColor);
    }
    if state.is_first_move_for(color) {
        return Err(BMM::SkipFirstTurn);
    }
    Ok(())
}

pub fn validate_move(state: &GameState, mv: &Move) -> Result<(), BMM> {
    match mv {
        Move::Set { piece } => validate_set_move(state, piece),
        Move::Skip { color } => validate_skip_move(state, *color),
    }
}

// ___ Move generation ___

/// All Set moves the current color can play this turn, or an empty list if none.
/// Matches Kotlin `getAllPossibleMoves`.
pub fn possible_moves(state: &GameState) -> Vec<Move> {
    if state.is_first_move_for(state.current_color()) {
        possible_start_moves(state)
    } else {
        possible_moves_normal(state)
    }
}

/// All valid Set moves for first-turn placement of `start_piece` along the border.
/// Mirrors Kotlin `getPossibleStartMoves`.
pub fn possible_start_moves(state: &GameState) -> Vec<Move> {
    let color = state.current_color();
    let kind = state.start_piece;
    let l = BOARD_LENGTH as i32;
    let mut out = Vec::new();

    for variant in kind.variants() {
        let rotation = variant.rotation;
        let is_flipped = variant.is_flipped;
        let shape = variant.shape;
        let (max_dx, max_dy) = bbox(&shape);
        // Top edge: y=0, x ranges so the shape stays in-bounds.
        for x in 0..=(l - 1 - max_dx) {
            let pos = Coordinates::new(x, 0);
            if try_add_start(state, kind, color, rotation, is_flipped, pos, &mut out) {
                continue;
            }
        }
        // Right edge: x = l-1-max_dx, y 0..=l-1-max_dy.
        for y in 0..=(l - 1 - max_dy) {
            let pos = Coordinates::new(l - 1 - max_dx, y);
            let _ = try_add_start(state, kind, color, rotation, is_flipped, pos, &mut out);
        }
        // Bottom edge: y = l-1-max_dy, x 0..=l-1-max_dx.
        for x in 0..=(l - 1 - max_dx) {
            let pos = Coordinates::new(x, l - 1 - max_dy);
            let _ = try_add_start(state, kind, color, rotation, is_flipped, pos, &mut out);
        }
        // Left edge: x=0, y 0..=l-1-max_dy.
        for y in 0..=(l - 1 - max_dy) {
            let pos = Coordinates::new(0, y);
            let _ = try_add_start(state, kind, color, rotation, is_flipped, pos, &mut out);
        }
    }
    out
}

fn try_add_start(
    state: &GameState,
    kind: PieceShape,
    color: Color,
    rotation: Rotation,
    is_flipped: bool,
    pos: Coordinates,
    out: &mut Vec<Move>,
) -> bool {
    let piece = Piece::new(color, kind, rotation, is_flipped, pos);
    if validate_set_move(state, &piece).is_ok() {
        out.push(Move::Set { piece });
        true
    } else {
        false
    }
}

/// All valid Set moves for a non-first turn. Mirrors Kotlin `getPossibleMoves` + `getPossibleMovesForShape`.
pub fn possible_moves_normal(state: &GameState) -> Vec<Move> {
    let color = state.current_color();
    let valid_fields = state.board.valid_fields(color);
    let mut out = Vec::new();

    for &kind in state.undeployed(color) {
        for variant in kind.variants() {
            let rotation = variant.rotation;
            let is_flipped = variant.is_flipped;
            let shape = variant.shape;
            let (max_dx, max_dy) = bbox(&shape);
            for field in &valid_fields {
                // Slide the piece so the valid field is somewhere inside its bounding box.
                let x_start = field.x - max_dx;
                let y_start = field.y - max_dy;
                for dx in x_start..=field.x {
                    for dy in y_start..=field.y {
                        let pos = Coordinates::new(dx, dy);
                        let piece = Piece::new(color, kind, rotation, is_flipped, pos);
                        if validate_set_move(state, &piece).is_ok() {
                            out.push(Move::Set { piece });
                        }
                    }
                }
            }
        }
    }
    out
}

/// Returns possible moves; if empty, returns a single Skip move of the current color.
/// Matches Kotlin `getSensibleMoves`.
pub fn sensible_moves(state: &GameState) -> Vec<Move> {
    let moves = possible_moves(state);
    if moves.is_empty() {
        vec![Move::Skip {
            color: state.current_color(),
        }]
    } else {
        moves
    }
}

fn bbox(shape: &[(i32, i32)]) -> (i32, i32) {
    let mut dx = 0;
    let mut dy = 0;
    for &(x, y) in shape {
        if x > dx {
            dx = x;
        }
        if y > dy {
            dy = y;
        }
    }
    (dx, dy)
}

// ___ make / unmake ___

/// Records enough to reverse a single Set/Skip move.
#[derive(Debug, Clone)]
pub struct MoveChange {
    pub color: Color,
    /// For Set: occupied cells (previously empty) we colored.
    pub set_cells: Vec<Coordinates>,
    /// The kind that was removed from this color's undeployed list, if any.
    pub kind_removed: Option<PieceShape>,
    /// Previous turn counter.
    pub prev_turn: u32,
    /// Previous `last_move` value.
    pub prev_last_move: Option<Move>,
    /// Number of turns that were advanced (>=1).
    pub turns_advanced: u32,
    /// Whether `last_move_mono` was modified and what the previous value was (if any).
    pub prev_mono_entry: Option<(Color, Option<bool>)>,
}

impl Move {
    /// Apply this move to `state`, returning the change record needed for `unmake_move`.
    /// When `VALIDATE_MOVE` is true, this first validates the move.
    pub fn make_move(&self, state: &mut GameState) -> Result<MoveChange, BMM> {
        let color = self.color();
        if VALIDATE_MOVE {
            validate_move(state, self)?;
        }

        let prev_turn = state.turn;
        let prev_last_move = state.last_move;

        let mut set_cells: Vec<Coordinates> = Vec::new();
        let mut kind_removed: Option<PieceShape> = None;
        let mut prev_mono_entry: Option<(Color, Option<bool>)> = None;

        match self {
            Move::Set { piece } => {
                let coords = piece.coordinates();
                for c in &coords {
                    if Board::contains(c.x, c.y) {
                        let slot = state.board.get_mut(c.x as usize, c.y as usize);
                        *slot = color.to_field_content();
                        set_cells.push(*c);
                    }
                }
                // Remove kind from undeployed list.
                let list = state.undeployed_mut(color);
                if let Some(idx) = list.iter().position(|s| *s == piece.kind) {
                    kind_removed = Some(list.remove(idx));
                }
                // If the color has no pieces left, record monoLast.
                if state.undeployed(color).is_empty() {
                    prev_mono_entry = Some((color, state.last_move_mono.get(&color).copied()));
                    state
                        .last_move_mono
                        .insert(color, piece.kind == PieceShape::Mono);
                }
            }
            Move::Skip { .. } => {
                // No board change.
            }
        }

        let _ = state.advance();
        state.last_move = Some(*self);

        Ok(MoveChange {
            color,
            set_cells,
            kind_removed,
            prev_turn,
            prev_last_move,
            turns_advanced: state.turn.saturating_sub(prev_turn),
            prev_mono_entry,
        })
    }

    /// Reverse a previously applied move given the [`MoveChange`] record.
    pub fn unmake_move(state: &mut GameState, change: MoveChange) {
        // Restore board cells.
        for c in &change.set_cells {
            if Board::contains(c.x, c.y) {
                *state.board.get_mut(c.x as usize, c.y as usize) = FieldContent::Empty;
            }
        }
        // Restore the undeployed list: re-insert the kind in canonical position.
        if let Some(kind) = change.kind_removed {
            let list = state.undeployed_mut(change.color);
            if !list.contains(&kind) {
                list.push(kind);
                // Re-sort to canonical PieceShape::ALL order so make/unmake is stable.
                list.sort_by_key(|s| s.to_index());
            }
        }
        // Restore last_move_mono entry.
        if let Some((color, prev)) = change.prev_mono_entry {
            match prev {
                Some(value) => {
                    state.last_move_mono.insert(color, value);
                }
                None => {
                    state.last_move_mono.remove(&color);
                }
            }
        }
        // Restore turn.
        state.turn = change.prev_turn;
        state.last_move = change.prev_last_move;
    }
}

// ___ Scoring ___

/// Matches Kotlin `getPointsFromUndeployed`.
pub fn points_from_undeployed(undeployed: &[PieceShape], mono_last: bool) -> u32 {
    if undeployed.is_empty() {
        return SUM_MAX_SQUARES + 15 + if mono_last { 5 } else { 0 };
    }
    SUM_MAX_SQUARES - undeployed.iter().map(|s| s.size() as u32).sum::<u32>()
}

impl GameState {
    pub fn points_for_color(&self, color: Color) -> u32 {
        let mono_last = self.last_move_mono.get(&color).copied().unwrap_or(false);
        points_from_undeployed(self.undeployed(color), mono_last)
    }

    pub fn points_for_team(&self, team: Team) -> u32 {
        Color::ALL
            .iter()
            .filter(|c| c.team() == team)
            .map(|c| self.points_for_color(*c))
            .sum()
    }

    /// Winner team + reason. `Some((None, ...))` for a tie. `None` if the game isn't over.
    pub fn game_result(&self) -> Option<(Option<Team>, crate::neutral::Winner)> {
        if !self.is_over() {
            return None;
        }
        let one = self.points_for_team(Team::One);
        let two = self.points_for_team(Team::Two);
        let (winner_team, reason_text) = if one > two {
            (
                Some(Team::One),
                format!("{one} hat am meisten Punkte erzielt."),
            )
        } else if two > one {
            (
                Some(Team::Two),
                format!("{two} hat am meisten Punkte erzielt."),
            )
        } else {
            (None, "Unentschieden".to_string())
        };
        Some((
            winner_team,
            crate::neutral::Winner {
                team: winner_team,
                regular: true,
                reason: Some(reason_text),
            },
        ))
    }
}

// ___ helper trait extension on PieceShape for canonical ordering ___

impl PieceShape {
    pub fn to_index(self) -> usize {
        self as usize
    }
}

// ___ Score / GameResult transport structs (parsed from <data class="result">) ___

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoreDefinition {
    pub fragments: Vec<crate::neutral::ScoreFragment>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GameResult {
    pub definition: ScoreDefinition,
    /// One (Team, PlayerScore) per player — order is server-determined.
    pub scores: Vec<(Team, crate::neutral::PlayerScore)>,
    pub winner: Option<crate::neutral::Winner>,
}

impl TryFrom<ReceivedData> for GameResult {
    type Error = String;
    fn try_from(data: ReceivedData) -> Result<Self, Self::Error> {
        let definition = {
            let fragments = data
                .definition
                .map(|d| {
                    d.fragments
                        .into_iter()
                        .map(|f| {
                            let aggregation =
                                crate::neutral::ScoreAggregation::from_str(&f.aggregation.value)?;
                            let relevant = f.relevant_for_ranking.value.trim() == "true";
                            Ok(crate::neutral::ScoreFragment {
                                name: f.name,
                                aggregation,
                                relevant_for_ranking: relevant,
                            })
                        })
                        .collect::<Result<Vec<_>, String>>()
                })
                .transpose()?
                .unwrap_or_default();
            ScoreDefinition { fragments }
        };

        let mut scores: Vec<(Team, crate::neutral::PlayerScore)> = Vec::new();
        if let Some(s) = data.scores {
            for entry in s.entries {
                let team = Team::from_str(&entry.player.team)?;
                if let Some(score) = entry.score {
                    let mut parts = Vec::with_capacity(score.parts.len());
                    for p in score.parts {
                        let v = p.value.trim().parse::<f64>().map_err(|e| e.to_string())?;
                        parts.push(v);
                    }
                    scores.push((team, crate::neutral::PlayerScore { parts }));
                }
            }
        }

        let winner = data.winner.map(|w| crate::neutral::Winner {
            team: w.team.as_deref().and_then(|t| Team::from_str(t).ok()),
            regular: w
                .regular
                .as_deref()
                .map(|s| s.trim() == "true")
                .unwrap_or(true),
            reason: w.reason,
        });

        Ok(GameResult {
            definition,
            scores,
            winner,
        })
    }
}

// ___ ComMessage: same architecture as 2026er ___

#[derive(Debug, Clone, PartialEq)]
pub enum RoomMessage {
    Memento(Box<GameState>),
    Result(Box<GameResult>),
    WelcomeMessage { color: Team },
    MoveRequest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedRoom {
    pub reservations: (String, String),
    pub room_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdminMessage {
    /// (reservation, reservation)
    Prepared(PreparedRoom),
}

impl TryFrom<ReceivedRoom> for RoomMessage {
    type Error = String;
    fn try_from(recv_room: ReceivedRoom) -> Result<Self, Self::Error> {
        let data = recv_room
            .data
            .ok_or_else(|| "missing room message <data>".to_string())?;
        let class = data
            .class
            .as_deref()
            .ok_or_else(|| "missing room message class".to_string())?;
        match class {
            "memento" => {
                let state = data
                    .state
                    .ok_or_else(|| "memento should contain <state>".to_string())?;
                let state = GameState::try_from(state)?;
                Ok(RoomMessage::Memento(Box::new(state)))
            }
            "result" => {
                let result = GameResult::try_from(data)?;
                Ok(RoomMessage::Result(Box::new(result)))
            }
            "welcomeMessage" => {
                let color = data
                    .color_attr
                    .as_deref()
                    .map(Team::from_str)
                    .transpose()?
                    .ok_or_else(|| "welcomeMessage should have a color attribute".to_string())?;
                Ok(RoomMessage::WelcomeMessage { color })
            }
            "moveRequest" => Ok(RoomMessage::MoveRequest),
            other => Err(format!("unknown room message class '{other}'")),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ComMessage {
    Joined(Joined),
    Left(Left),
    Room(Box<RoomMessage>),
    Admin(AdminMessage),
}

// re-export of neutral::align kept inside neutral module; not used here directly.
