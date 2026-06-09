use std::sync::{Arc, Mutex};

use nightshade::prelude::{Entity, Vec2, Vec3};
use protocol::GamePhase;

/// The downloaded Polyhaven skybox bytes, written by `ehttp` callbacks and
/// taken by the render thread once they arrive.
#[derive(Default)]
pub struct Sky {
    pub bytes: Arc<Mutex<Option<Vec<u8>>>>,
}

/// The scroll-driven camera tour: the page's smoothed scroll progress, the
/// normalized pointer glance for parallax, and the reduced-motion flag.
#[derive(Default)]
pub struct Tour {
    pub progress: f32,
    pub target_progress: f32,
    pub glance: Vec2,
    pub reduced_motion: bool,
}

/// A continuous rotation applied to an entity every frame.
pub struct Spinner {
    pub entity: Entity,
    pub axis: Vec3,
    pub radians_per_second: f32,
}

/// A vertical sine bob around a base height.
pub struct Bobber {
    pub entity: Entity,
    pub base: Vec3,
    pub amplitude: f32,
    pub frequency: f32,
    pub phase: f32,
}

/// A circular orbit around a fixed center.
pub struct Orbiter {
    pub entity: Entity,
    pub center: Vec3,
    pub radius: f32,
    pub height: f32,
    pub radians_per_second: f32,
    pub phase: f32,
}

/// Ambient scene animation state.
#[derive(Default)]
pub struct Ambient {
    pub time: f32,
    pub spinners: Vec<Spinner>,
    pub bobbers: Vec<Bobber>,
    pub orbiters: Vec<Orbiter>,
}

/// An emissive set piece that springs back to its home position after the
/// pointer ray shoves it.
pub struct Floater {
    pub entity: Entity,
    pub home: Vec3,
    pub velocity: Vec3,
}

/// The pointer-reactive floaters.
#[derive(Default)]
pub struct Floaters {
    pub items: Vec<Floater>,
}

/// The siege game: arena entities, scoreboard, and the timers that drive
/// combo decay, projectile cleanup, and the failed-level settle check.
pub struct Game {
    pub phase: GamePhase,
    pub level: u32,
    pub score: u32,
    pub shots_left: u32,
    pub shots_total: u32,
    pub targets_total: u32,
    pub arena: Vec<Entity>,
    pub blocks: Vec<Entity>,
    pub targets: Vec<(Entity, f32)>,
    pub projectiles: Vec<(Entity, f32)>,
    pub combo: u32,
    pub combo_timer: f32,
    pub settle_timer: f32,
    pub dirty: bool,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            phase: GamePhase::Idle,
            level: 1,
            score: 0,
            shots_left: 0,
            shots_total: 0,
            targets_total: 0,
            arena: Vec::new(),
            blocks: Vec::new(),
            targets: Vec::new(),
            projectiles: Vec::new(),
            combo: 0,
            combo_timer: 0.0,
            settle_timer: 0.0,
            dirty: false,
        }
    }
}
