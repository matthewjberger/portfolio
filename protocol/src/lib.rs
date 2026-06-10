use serde::{Deserialize, Serialize};

/// Envelope field carrying the serialized message in every `postMessage`.
pub const MESSAGE_KEY: &str = "message";
/// Envelope field carrying the transferred `OffscreenCanvas` (on `Init` only).
pub const CANVAS_KEY: &str = "canvas";

/// Number of levels in the built-in siege game.
pub const GAME_LEVELS: u32 = 5;

/// Display name of a siege game level (1-based).
pub fn game_level_name(level: u32) -> &'static str {
    match level {
        1 => "First Contact",
        2 => "Twin Keeps",
        3 => "The Wall",
        4 => "Citadel",
        _ => "Nightshade Spire",
    }
}

/// Lifecycle phase of the siege game.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GamePhase {
    Idle,
    /// The intro cutscene is sweeping the arena before control hands over.
    Intro,
    Playing,
    Cleared,
    Failed,
}

/// Page to worker game actions.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum GameCommand {
    /// Build the given level (1-based) and start playing. With `intro` the
    /// level opens on its cutscene; without it control is immediate.
    Start { level: u32, intro: bool },
    /// Jump the intro cutscene to its end.
    SkipIntro,
    /// Fire a cannonball through this physical pixel position.
    Fire { x: f32, y: f32 },
    /// Tear the arena down and return to the portfolio tour.
    Exit,
}

/// Worker to page game scoreboard, sent whenever it changes.
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GameStatus {
    pub phase: GamePhase,
    pub level: u32,
    pub score: u32,
    pub shots_left: u32,
    pub shots_total: u32,
    pub targets_left: u32,
    pub targets_total: u32,
    pub combo: u32,
}

/// Lifecycle phase of a forwarded touch contact.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TouchPhase {
    Started,
    Moved,
    Ended,
    Cancelled,
}

/// Page to worker. Pixel quantities are physical surface pixels (CSS pixels
/// times the device pixel ratio), origin at the canvas top-left.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    /// Sent once with the `OffscreenCanvas` in the transfer list.
    Init {
        width: f32,
        height: f32,
    },
    Resize {
        width: f32,
        height: f32,
    },
    /// Absolute cursor position in physical pixels, forwarded while the game
    /// is active so the engine camera can orbit.
    PointerMove {
        x: f32,
        y: f32,
    },
    /// A mouse button changed. `button` is 0 left, 1 middle, 2 right.
    PointerButton {
        button: u8,
        pressed: bool,
    },
    /// Wheel delta in raw pixels (the worker converts to scroll lines).
    Wheel {
        delta: f32,
    },
    /// A touch contact in physical pixels, forwarded while the game is active.
    Touch {
        id: u64,
        phase: TouchPhase,
        x: f32,
        y: f32,
    },
    /// Overall page scroll progress in 0..1, drives the camera tour.
    Scroll {
        progress: f32,
    },
    /// Normalized pointer position in -1..1 (x right, y up), drives the
    /// parallax glance and the floater nudge ray.
    Glance {
        x: f32,
        y: f32,
    },
    /// Mirrors the page's prefers-reduced-motion media query.
    SetReducedMotion {
        enabled: bool,
    },
    /// Drive the built-in siege game.
    Game {
        command: GameCommand,
    },
}

/// Worker to page.
#[derive(Clone, Serialize, Deserialize)]
pub enum WorkerMessage {
    Ready,
    Stats {
        fps: f32,
    },
    /// The siege game scoreboard, sent whenever it changes.
    Game {
        status: GameStatus,
    },
    /// A target was knocked off its perch, with the points it scored.
    GameHit {
        points: u32,
        combo: u32,
    },
}
