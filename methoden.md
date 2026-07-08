# API-Referenz — Rust-Client für Blokus 2027

Hier findest du alle öffentlichen Methoden, Strukturen und Aufzählungen des Rust-Clients,
geordnet nach den Klassen/Kategorien des offiziellen Java-SDKs.

---

## Inhaltsverzeichnis

- [GameState (Spielstand)](#gamestate-spielstand)
- [Board (Spielfeld)](#board-spielfeld)
- [Move (Zug)](#move-zug)
- [Piece (Spielstein)](#piece-spielstein)
- [PieceShape (Steinform)](#pieceshape-steinform)
- [Color (Farbe)](#color-farbe)
- [Team (Team)](#team-team)
- [Rotation (Drehung)](#rotation-drehung)
- [Coordinates (Koordinaten)](#coordinates-koordinaten)
- [FieldContent (Feldinhalt)](#fieldcontent-feldinhalt)
- [Move-Generierung (GameRuleLogic)](#move-generierung-gamerulelogic)
- [IClientHandler (Client-Schnittstelle)](#iclienthandler-client-schnittstelle)
- [Fehlerbehandlung](#fehlerbehandlung)

---

## GameState (Spielstand)

```rust
pub struct GameState {
    pub turn: u32,
    pub start_piece: PieceShape,
    pub last_move: Option<Move>,
    pub board: Board,
    pub last_move_mono: HashMap<Color, bool>,
    pub blue_shapes: Vec<PieceShape>,
    pub yellow_shapes: Vec<PieceShape>,
    pub red_shapes: Vec<PieceShape>,
    pub green_shapes: Vec<PieceShape>,
    pub valid_colors: Vec<Color>,
}
```

### `.current_color()`

Gibt die Farbe zurück, die gerade am Zug ist.

- **Rückgabetyp:** `Color`

```rust
let color: Color = game_state.current_color();
```

### `.current_team()`

Gibt das Team zurück, das gerade am Zug ist.

- **Rückgabetyp:** `Team`

```rust
let team: Team = game_state.current_team();
```

### `.round()`

Gibt die aktuelle Runde zurück (1-basiert).

- **Rückgabetyp:** `u32`

```rust
let round: u32 = game_state.round();
```

### `.turn`

Der aktuelle Spielzug-Index (0-basiert).

- **Typ:** `u32`

```rust
let turn: u32 = game_state.turn;
```

### `.last_move`

Der letzte ausgeführte Zug, falls vorhanden.

- **Typ:** `Option<Move>`

```rust
if let Some(last) = &game_state.last_move {
    // ...
}
```

### `.is_over()`

Prüft, ob das Spiel beendet ist (keine gültigen Farben mehr, Rundenlimit erreicht oder alle Steine platziert).

- **Rückgabetyp:** `bool`

```rust
if game_state.is_over() {
    // Spiel beendet
}
```

### `.game_result()`

Gibt das Spielergebnis zurück, falls das Spiel beendet ist.

- **Rückgabetyp:** `Option<(Option<Team>, Winner)>`

```rust
if let Some((winner_team, winner_info)) = game_state.game_result() {
    // winner_team: None = Unentschieden, Some(team) = Sieger
}
```

### `.points_for_color(Color)`

Berechnet die Punkte für eine einzelne Farbe anhand der noch nicht gesetzten Steine.

- **Rückgabetyp:** `u32`

```rust
let points: u32 = game_state.points_for_color(Color::Blue);
```

### `.points_for_team(Team)`

Berechnet die Gesamtpunktzahl für ein Team (Summe beider Farben).

- **Rückgabetyp:** `u32`

```rust
let team_points: u32 = game_state.points_for_team(Team::Two);
```

### `.undeployed(Color)`

Gibt die Liste der noch nicht gesetzten Steine einer Farbe zurück.

- **Rückgabetyp:** `&Vec<PieceShape>`

```rust
let remaining: &Vec<PieceShape> = game_state.undeployed(Color::Yellow);
```

### `.is_first_move_for(Color)`

Prüft, ob die angegebene Farbe noch ihren ersten Zug machen muss (alle 21 Steine verfügbar).

- **Rückgabetyp:** `bool`

```rust
if game_state.is_first_move_for(Color::Red) {
    // Erster Zug dieser Farbe
}
```

### `.is_valid_color(Color)`

Prüft, ob eine Farbe noch im Spiel ist (nicht eliminiert wurde).

- **Rückgabetyp:** `bool`

```rust
if game_state.is_valid_color(Color::Green) {
    // Grün kann noch spielen
}
```

### `.valid_colors`

Liste der Farben, die noch spielberechtigt sind.

- **Typ:** `Vec<Color>`

```rust
for color in &game_state.valid_colors {
    println!("{:?} ist noch im Spiel", color);
}
```

### `.board`

Das aktuelle Spielfeld. Siehe [Board](#board-spielfeld).

---

## Board (Spielfeld)

```rust
pub struct Board {
    pub rows: [Row; BOARD_LENGTH],  // BOARD_LENGTH = 20
}
```

### `.contains(x, y)`

Prüft, ob die Koordinate innerhalb des 20×20-Spielfelds liegt.

- **Rückgabetyp:** `bool`

```rust
if Board::contains(5, 7) {
    // Gültige Position
}
```

### `.get(x, y)`

Liest den Feldinhalt an Position `(x, y)`.

- **Rückgabetyp:** `&FieldContent`

```rust
let content: &FieldContent = board.get(3, 4);
```

### `.colored_fields(Color)`

Gibt alle Koordinaten zurück, die mit der angegebenen Farbe belegt sind.

- **Rückgabetyp:** `Vec<Coordinates>`

```rust
let blue_fields: Vec<Coordinates> = board.colored_fields(Color::Blue);
```

### `.valid_fields(Color)`

Gibt alle leeren Felder zurück, die eine diagonale Berührung (Eckberührung) mit der angegebenen Farbe haben — mögliche Positionen für den nächsten Stein.

- **Rückgabetyp:** `Vec<Coordinates>`

```rust
let valid: Vec<Coordinates> = board.valid_fields(Color::Yellow);
```

### `.is_obstructed(Coordinates)`

Prüft, ob ein Feld bereits belegt ist.

- **Rückgabetyp:** `bool`

```rust
if board.is_obstructed(pos) {
    // Feld ist besetzt
}
```

### `.borders_on_color(Coordinates, Color)`

Prüft, ob ein Feld orthogonal an eine Farbe grenzt.

- **Rückgabetyp:** `bool`

```rust
if board.borders_on_color(pos, Color::Blue) {
    // Berührt Blau orthogonal (nicht erlaubt)
}
```

### `.corners_on_color(Coordinates, Color)`

Prüft, ob ein Feld diagonal an eine Farbe grenzt (Eckberührung).

- **Rückgabetyp:** `bool`

```rust
if board.corners_on_color(pos, Color::Yellow) {
    // Berührt Gelb diagonal (erlaubt, notwendig)
}
```

### `.is_empty()`

Prüft, ob das gesamte Spielfeld leer ist.

- **Rückgabetyp:** `bool`

```rust
if board.is_empty() {
    // Spielfeld ist leer — Eröffnung
}
```

---

## Move (Zug)

```rust
pub enum Move {
    Set { piece: Piece },
    Skip { color: Color },
}
```

### `.color()`

Gibt die Farbe zurück, die den Zug ausführt.

- **Rückgabetyp:** `Color`

```rust
let color: Color = mv.color();
```

### `Move::Set { piece }`

Setzt einen Stein auf das Spielfeld.

```rust
let mv = Move::Set { piece: my_piece };
```

### `Move::Skip { color }`

Überspringt den Zug (nicht im ersten Zug erlaubt).

```rust
let mv = Move::Skip { color: Color::Yellow };
```

### `.make_move(state)`

Wendet den Zug auf einen Spielstand an (validiert vorher wenn `VALIDATE_MOVE=true`).

- **Rückgabetyp:** `Result<MoveChange, BlokusMoveMistake>`

```rust
match mv.make_move(&mut game_state) {
    Ok(change) => { /* Zug rückgängig machbar via Move::unmake_move */ }
    Err(mistake) => { /* Zug ungültig */ }
}
```

### `.unmake_move(state, change)`

Macht einen zuvor ausgeführten Zug rückgängig.

```rust
Move::unmake_move(&mut game_state, change);
```

---

## Piece (Spielstein)

```rust
pub struct Piece {
    pub color: Color,
    pub kind: PieceShape,
    pub rotation: Rotation,
    pub is_flipped: bool,
    pub position: Coordinates,
}
```

### `Piece::new(color, kind, rotation, is_flipped, position)`

Erzeugt einen neuen Spielstein.

```rust
use socha::neutral::{Piece, PieceShape, Rotation, Color, Coordinates};
let piece = Piece::new(
    Color::Red,
    PieceShape::TetroL,
    Rotation::None,
    false,
    Coordinates::new(5, 5),
);
```

### `.coordinates()`

Berechnet die absoluten Spielfeld-Koordinaten, die dieser Stein belegt (abhängig von Position, Drehung und Spiegelung).

- **Rückgabetyp:** `Vec<Coordinates>`

```rust
let cells: Vec<Coordinates> = piece.coordinates();
```

### `.shape()`

Gibt die relative Form des Steins zurück (nach Drehung/Spiegelung, aber ohne Position).

- **Rückgabetyp:** `&[(i32, i32)]`

---

## PieceShape (Steinform)

```rust
pub enum PieceShape {
    // 21 Formen: 1 Monomino, 1 Domino, 2 Triominos, 5 Tetrominos, 12 Pentominos
    Mono,
    Domino,
    TrioL,
    TrioI,
    TetroO,
    TetroT,
    TetroI,
    TetroL,
    TetroZ,
    PentoL,
    PentoT,
    PentoV,
    PentoS,
    PentoZ,
    PentoI,
    PentoP,
    PentoW,
    PentoU,
    PentoR,
    PentoX,
    PentoY,
}
```

### `.variants()`

Gibt alle möglichen Drehungen/Spiegelungen dieser Form zurück (statisch gecacht).

- **Rückgabetyp:** `&'static [ShapeVariant]`

```rust
let variants: &[ShapeVariant] = PieceShape::TetroT.variants();
```

### `.coordinates()`

Gibt die kanonische (ausgerichtete) Koordinatenliste der Form zurück.

- **Rückgabetyp:** `&'static [(i32, i32)]`

```rust
let coords: &[(i32, i32)] = PieceShape::PentoL.coordinates();
```

### `.size()`

Gibt die Anzahl der Felder dieser Form zurück (1–5).

- **Rückgabetyp:** `usize`

```rust
let size: usize = kind.size();
```

### `PieceShape::ALL`

Alle 21 Formen als konstantes Array.

```rust
for shape in PieceShape::ALL {
    println!("{:?} hat {} Felder", shape, shape.size());
}
```

### `PieceShape::TOTAL`

Die Anzahl der Formen (21).

```rust
const COUNT: usize = PieceShape::TOTAL;
```

### `.transform(rotation, is_flipped)`

Wendet Drehung und optionale Spiegelung auf die Form an und gibt die ausgerichteten Koordinaten zurück.

- **Rückgabetyp:** `Vec<(i32, i32)>`

---

## Color (Farbe)

```rust
pub enum Color {
    Blue,    // Team One
    Yellow,  // Team Two
    Red,     // Team One
    Green,   // Team Two
}
```

### `.next()`

Nächste Farbe in der Zugreihenfolge: Blue → Yellow → Red → Green → Blue.

- **Rückgabetyp:** `Color`

```rust
let next = Color::Blue.next(); // Yellow
```

### `.team()`

Gibt das Team dieser Farbe zurück.

- **Rückgabetyp:** `Team`

```rust
let team: Team = Color::Red.team(); // Team::One
```

### `Color::ALL`

Alle 4 Farben in Zugreihenfolge als konstantes Array.

```rust
for color in Color::ALL {
    println!("{:?}", color);
}
```

---

## Team (Team)

```rust
pub enum Team {
    One,  // Blau + Rot
    Two,  // Gelb + Grün
}
```

### `.opponent()`

Gibt das gegnerische Team zurück.

- **Rückgabetyp:** `Team`

```rust
let other = Team::One.opponent(); // Team::Two
```

---

## Rotation (Drehung)

```rust
pub enum Rotation {
    None,     // 0°
    Right,    // 90° im Uhrzeigersinn
    Mirror,   // 180°
    Left,     // 90° gegen Uhrzeigersinn
}
```

---

## Coordinates (Koordinaten)

```rust
pub struct Coordinates {
    pub x: i32,
    pub y: i32,
}
```

### `Coordinates::new(x, y)`

Erzeugt eine neue Koordinate.

```rust
let pos = Coordinates::new(3, 7);
```

### `.offset(dx, dy)`

Verschiebt die Koordinate um `dx`/`dy`.

- **Rückgabetyp:** `Coordinates`

```rust
let moved = pos.offset(1, -2);
```

### `.neighbors()`

Gibt die 4 orthogonalen Nachbarn (oben, unten, links, rechts) zurück.

- **Rückgabetyp:** `[Coordinates; 4]`

```rust
for n in pos.neighbors() {
    // ...
}
```

### `.diagonal_neighbors()`

Gibt die 4 diagonalen Nachbarn zurück.

- **Rückgabetyp:** `[Coordinates; 4]`

```rust
for d in pos.diagonal_neighbors() {
    // ...
}
```

---

## FieldContent (Feldinhalt)

```rust
pub enum FieldContent {
    Empty,
    Blue,
    Yellow,
    Red,
    Green,
}
```

### `.is_empty()`

Prüft, ob das Feld leer ist.

- **Rückgabetyp:** `bool`

```rust
if field.is_empty() {
    // Frei
}
```

### `.letter()`

Gibt den Ein-Buchstaben-Code zurück (`.`, `B`, `Y`, `R`, `G`).

- **Rückgabetyp:** `char`

---

## Move-Generierung (GameRuleLogic)

### `possible_moves(state)`

Gibt alle legalen Set-Züge für die aktuelle Farbe zurück. Falls die Farbe keinen legalen Zug hat, wird eine leere Liste zurückgegeben.

- **Rückgabetyp:** `Vec<Move>`

```rust
let moves: Vec<Move> = possible_moves(&game_state);
```

### `sensible_moves(state)`

Wie `possible_moves`, gibt aber bei leerer Liste einen `Skip`-Zug zurück, sodass immer mindestens ein Zug vorhanden ist.

- **Rückgabetyp:** `Vec<Move>`

```rust
let moves: Vec<Move> = sensible_moves(&game_state);
```

### `validate_set_move(state, piece)`

Prüft, ob ein Set-Zug nach den Blokus-Regeln gültig ist (Farbe am Zug?, Form verfügbar?, im Spielfeld?, blockiert?, Eckberührung?, keine Kantenberührung?, Randbedingung für ersten Zug?).

- **Rückgabetyp:** `Result<(), BlokusMoveMistake>`

```rust
match validate_set_move(&game_state, &piece) {
    Ok(()) => { /* Zug gültig */ }
    Err(e) => { println!("Ungültig: {}", e.message()); }
}
```

### `validate_skip_move(state, color)`

Prüft, ob ein Skip-Zug erlaubt ist (nicht im ersten Zug).

- **Rückgabetyp:** `Result<(), BlokusMoveMistake>`

### `validate_move(state, mv)`

Prüft einen beliebigen Zug (Set oder Skip).

- **Rückgabetyp:** `Result<(), BlokusMoveMistake>`

### `is_on_border(pos)`

Prüft, ob eine Koordinate am Spielfeldrand liegt.

- **Rückgabetyp:** `bool`

### `points_from_undeployed(undeployed, mono_last)`

Berechnet Punkte aus nicht gesetzten Steinen. Formel:
- Alle Steine gesetzt: `89 + 15 + (5 falls Monomino zuletzt) = 109`
- Sonst: `89 - Summe_der_Größen_nicht_gesetzter_Steine`

- **Rückgabetyp:** `u32`

### `remove_valid_colors(state)`

Entfernt Farben aus `valid_colors`, die keine legalen Züge mehr haben (wird automatisch in `make_move` aufgerufen).

---

## IClientHandler (Client-Schnittstelle)

Zu implementieren, um einen eigenen Bot zu schreiben:

```rust
pub trait IClientHandler {
    fn calculate_move(&mut self) -> Move;
    fn on_gamestate_update(&mut self, state: GameState);
    fn on_welcome_message(&mut self, team: Team);
    // Optionale Methoden:
    fn on_game_joined(&mut self, room_id: &str) { ... }
    fn on_game_left(&mut self) { ... }
    fn on_game_result(&mut self, res: &GameResult) { ... }
    fn while_waiting(&mut self, cancel_handler: ComCancelHandler) { ... }
}
```

### `start_iclient(addr, reservation, handler, sleep, timeout)`

Startet die Client-Loop und verbindet zum Server.

```rust
use socha::i_client_handler::start_iclient;

fn main() -> Result<(), socha::error::ComError> {
    let mut handler = MyBot::default();
    start_iclient(
        "localhost:13050",  // Server-Adresse
        None,               // Reservierungscode
        &mut handler,       // IClientHandler-Implementierung
        std::time::Duration::from_millis(2),   // Polling-Intervall
        std::time::Duration::from_secs_f64(1.0), // Timeout
    )?;
    Ok(())
}
```

---

## Fehlerbehandlung

```rust
pub enum ReceiveErr {
    Io(io::Error),
    XmlError(strong_xml::XmlError),
    ConnectionClosed(ConnectionClosedErr),
    FailedToBuildRoomMessage(String),
    FailedToBuildAdminMessage(String),
}

pub enum ComError {
    SendErr(SendErr),
    ReceiveErr(ReceiveErr),
}
```

---

## Konstanten

```rust
pub const BOARD_LENGTH: usize = 20;     // Spielfeld 20×20
pub const ROUND_LIMIT: u32 = 25;        // Maximal 25 Runden
pub const TOTAL_PIECE_SHAPES: usize = 21; // 21 verschiedene Formen
pub const SUM_MAX_SQUARES: u32 = 89;    // Summe aller Felder (1+2+6+20+60)
pub const VALIDATE_MOVE: bool = true;   // make_move validiert vor dem Ausführen
```

---

## Beispiel: Bot-Grundgerüst

```rust
use log::LevelFilter;
use rand::Rng;
use socha::i_client_handler::handler_trait::IClientHandler;
use socha::i_client_handler::start_iclient;
use socha::internal::{GameState, sensible_moves};
use socha::neutral::{Move, Team};

#[derive(Debug, Default)]
struct MyBot {
    game_state: GameState,
}

impl IClientHandler for MyBot {
    fn calculate_move(&mut self) -> Move {
        let moves = sensible_moves(&self.game_state);
        let mut rng = rand::rng();
        moves[rng.random_range(0..moves.len())]
    }

    fn on_gamestate_update(&mut self, state: GameState) {
        self.game_state = state;
    }

    fn on_welcome_message(&mut self, team: Team) {
        println!("Spiele als Team {:?}", team);
    }
}

fn main() -> Result<(), socha::error::ComError> {
    let _ = simple_logging::log_to_file("bot.log", LevelFilter::Info);
    start_iclient("localhost:13050", None, &mut MyBot::default(),
        std::time::Duration::from_millis(2),
        std::time::Duration::from_secs_f64(1.0))?;
    Ok(())
}
```

---

## BlokusMoveMistake (Fehlerursachen)

```rust
pub enum BlokusMoveMistake {
    WrongColor,          // Falsche Farbe am Zug
    NotOnBorder,         // Erster Zug nicht am Rand
    NoSharedCorner,      // Keine Eckberührung mit eigenem Stein
    WrongShape,          // Erster Zug nicht mit Startform
    SkipFirstTurn,       // Ersten Zug übersprungen
    DuplicateShape,      // Form bereits gesetzt
    OutOfBounds,         // Stein ragt über Spielfeldrand
    Obstructed,          // Stein überdeckt anderen Stein
    TouchesSameColor,    // Berührt eigenen Stein orthogonal
}
```

Jede Variante hat eine `.message()`-Methode, die eine deutsche Fehlerbeschreibung zurückgibt.

---

## Score / Ergebnis

### `GameResult`

```rust
pub struct GameResult {
    pub definition: ScoreDefinition,
    pub scores: Vec<(Team, PlayerScore)>,
    pub winner: Option<Winner>,
}
```

### `Winner`

```rust
pub struct Winner {
    pub team: Option<Team>,  // None = Unentschieden
    pub regular: bool,
    pub reason: Option<String>,
}
```
