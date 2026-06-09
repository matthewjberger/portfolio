use leptos::prelude::*;
use protocol::GamePhase;

/// Section ids in page order, shared by the nav, the scroll spy, and the
/// reveal logic.
pub const SECTIONS: [(&str, &str); 7] = [
    ("hero", "About"),
    ("highlights", "Highlights"),
    ("experience", "Experience"),
    ("projects", "Projects"),
    ("crates", "Crates"),
    ("skills", "Skills"),
    ("education", "Education"),
];

/// All page state, grouped as signals. `Copy`, so it threads into every
/// component and closure without cloning.
#[derive(Clone, Copy)]
pub struct PortfolioState {
    pub webgpu: bool,
    pub ready: RwSignal<bool>,
    pub fps: RwSignal<f32>,
    pub section: RwSignal<usize>,
    pub revealed: RwSignal<usize>,
    pub reduced_motion: RwSignal<bool>,
    pub game_phase: RwSignal<GamePhase>,
    pub game_level: RwSignal<u32>,
    pub game_score: RwSignal<u32>,
    pub game_shots_left: RwSignal<u32>,
    pub game_shots_total: RwSignal<u32>,
    pub game_targets_left: RwSignal<u32>,
    pub game_targets_total: RwSignal<u32>,
    pub game_combo: RwSignal<u32>,
    pub game_menu_open: RwSignal<bool>,
    pub game_hits: RwSignal<Vec<(u32, String)>>,
}

impl PortfolioState {
    pub fn new(webgpu: bool) -> Self {
        Self {
            webgpu,
            ready: RwSignal::new(false),
            fps: RwSignal::new(0.0),
            section: RwSignal::new(0),
            revealed: RwSignal::new(0),
            reduced_motion: RwSignal::new(false),
            game_phase: RwSignal::new(GamePhase::Idle),
            game_level: RwSignal::new(1),
            game_score: RwSignal::new(0),
            game_shots_left: RwSignal::new(0),
            game_shots_total: RwSignal::new(0),
            game_targets_left: RwSignal::new(0),
            game_targets_total: RwSignal::new(0),
            game_combo: RwSignal::new(0),
            game_menu_open: RwSignal::new(false),
            game_hits: RwSignal::new(Vec::new()),
        }
    }
}

fn local_storage() -> Option<web_sys::Storage> {
    web_sys::window().and_then(|window| window.local_storage().ok().flatten())
}

/// Key under which the siege game records the highest unlocked level.
const GAME_UNLOCKED_KEY: &str = "portfolio_game_unlocked";

fn game_best_key(level: u32) -> String {
    format!("portfolio_game_best_{level}")
}

fn read_storage_number(key: &str) -> u32 {
    local_storage()
        .and_then(|storage| storage.get_item(key).ok().flatten())
        .and_then(|value| value.parse().ok())
        .unwrap_or(0)
}

/// The highest siege level unlocked on this browser (at least 1).
pub fn game_unlocked() -> u32 {
    read_storage_number(GAME_UNLOCKED_KEY).max(1)
}

/// Unlocks a siege level if it is beyond the current progress.
pub fn unlock_game_level(level: u32) {
    if level > game_unlocked()
        && let Some(storage) = local_storage()
    {
        let _ = storage.set_item(GAME_UNLOCKED_KEY, &level.to_string());
    }
}

/// The best recorded score for a siege level on this browser.
pub fn game_best(level: u32) -> u32 {
    read_storage_number(&game_best_key(level))
}

/// Records a siege level score if it beats the stored best.
pub fn record_game_best(level: u32, score: u32) {
    if score > game_best(level)
        && let Some(storage) = local_storage()
    {
        let _ = storage.set_item(&game_best_key(level), &score.to_string());
    }
}
