use crate::ecs::PortfolioWorld;
use crate::systems::world::textures::{FLOOR_TEXTURE, prototype_material};
use nightshade::ecs::camera::components::{Projection, compute_pan_orbit_transform};
use nightshade::ecs::material::components::Material;
use nightshade::prelude::*;
use protocol::{GAME_LEVELS, GameCommand, GamePhase, GameStatus, WorkerMessage, game_level_name};

const ARENA_X: f32 = 0.0;
const ARENA_Z: f32 = 150.0;
const PLATFORM_HALF: f32 = 8.0;
const SCORE_DROP_Y: f32 = -2.5;
const KNOCKDOWN_DROP: f32 = 1.1;
const CLEANUP_DROP_Y: f32 = -60.0;
const PROJECTILE_SPEED: f32 = 48.0;
const PROJECTILE_RADIUS: f32 = 0.42;
const PROJECTILE_MASS: f32 = 8.0;
const PROJECTILE_LIFETIME: f32 = 6.0;
const TARGET_RADIUS: f32 = 0.45;
const COMBO_WINDOW: f32 = 3.5;
const SETTLE_DELAY: f32 = 3.0;
const TARGET_POINTS: u32 = 100;
const SHOT_BONUS: u32 = 50;
const AIM_YAW: f32 = 0.8;
const AIM_PITCH: f32 = 0.35;
const AIM_RADIUS: f32 = 26.0;

fn aim_focus() -> Vec3 {
    arena_origin() + Vec3::new(0.0, 2.5, 0.0)
}

fn arena_origin() -> Vec3 {
    Vec3::new(ARENA_X, 0.0, ARENA_Z)
}

struct Block {
    position: Vec3,
    size: Vec3,
    color: Vec3,
}

struct Layout {
    shots: u32,
    blocks: Vec<Block>,
    targets: Vec<Vec3>,
}

/// Applies one game command from the page.
pub fn apply(portfolio: &mut PortfolioWorld, world: &mut World, command: GameCommand) {
    match command {
        GameCommand::Start { level, intro } => {
            start(portfolio, world, level.clamp(1, GAME_LEVELS), intro)
        }
        GameCommand::SkipIntro => skip_intro(portfolio, world),
        GameCommand::Fire { x, y } => fire(portfolio, world, x, y),
        GameCommand::Exit => exit(portfolio, world),
    }
}

/// Advances the game one frame: decays the combo, ages and culls projectiles,
/// scores targets knocked off their perches, and resolves the level outcome.
pub fn update(portfolio: &mut PortfolioWorld, world: &mut World) {
    if portfolio.resources.game.phase == GamePhase::Idle {
        return;
    }
    if portfolio.resources.game.phase == GamePhase::Intro {
        advance_cutscene_system(world);
        if cutscene_finished(world) {
            finish_intro(portfolio, world);
        }
        if std::mem::take(&mut portfolio.resources.game.dirty) {
            post_status(portfolio);
        }
        return;
    }
    let delta = world.resources.window.timing.delta_time;

    if portfolio.resources.game.combo_timer > 0.0 {
        portfolio.resources.game.combo_timer -= delta;
        if portfolio.resources.game.combo_timer <= 0.0 && portfolio.resources.game.combo > 0 {
            portfolio.resources.game.combo = 0;
            portfolio.resources.game.dirty = true;
        }
    }

    let mut expired = Vec::new();
    for (projectile, age) in &mut portfolio.resources.game.projectiles {
        *age += delta;
        let y = world
            .core
            .get_local_transform(*projectile)
            .map(|transform| transform.translation.y)
            .unwrap_or(CLEANUP_DROP_Y);
        if *age >= PROJECTILE_LIFETIME || y < CLEANUP_DROP_Y {
            expired.push(*projectile);
        }
    }
    portfolio
        .resources
        .game
        .projectiles
        .retain(|(projectile, _)| !expired.contains(projectile));
    for projectile in expired {
        if world
            .core
            .entity_has_components(projectile, LOCAL_TRANSFORM)
        {
            despawn_recursive_immediate(world, projectile);
        }
    }

    let mut sunk = Vec::new();
    portfolio.resources.game.blocks.retain(|&block| {
        let y = world
            .core
            .get_local_transform(block)
            .map(|transform| transform.translation.y)
            .unwrap_or(CLEANUP_DROP_Y);
        if y < CLEANUP_DROP_Y {
            sunk.push(block);
            false
        } else {
            true
        }
    });
    for block in sunk {
        if world.core.entity_has_components(block, LOCAL_TRANSFORM) {
            despawn_recursive_immediate(world, block);
        }
    }

    let mut scored = Vec::new();
    portfolio
        .resources
        .game
        .targets
        .retain(|&(target, spawn_y)| {
            let y = world
                .core
                .get_local_transform(target)
                .map(|transform| transform.translation.y)
                .unwrap_or(SCORE_DROP_Y - 1.0);
            if y < SCORE_DROP_Y || y < spawn_y - KNOCKDOWN_DROP {
                scored.push(target);
                false
            } else {
                true
            }
        });
    for target in scored {
        if world.core.entity_has_components(target, LOCAL_TRANSFORM) {
            despawn_recursive_immediate(world, target);
        }
        let game = &mut portfolio.resources.game;
        game.combo += 1;
        game.combo_timer = COMBO_WINDOW;
        let points = TARGET_POINTS * game.combo;
        game.score += points;
        game.dirty = true;
        crate::post(&WorkerMessage::GameHit {
            points,
            combo: game.combo,
        });
    }

    if portfolio.resources.game.phase == GamePhase::Playing {
        if portfolio.resources.game.targets.is_empty() {
            let game = &mut portfolio.resources.game;
            game.score += game.shots_left * SHOT_BONUS;
            game.phase = GamePhase::Cleared;
            game.dirty = true;
        } else if portfolio.resources.game.shots_left == 0
            && portfolio.resources.game.projectiles.is_empty()
        {
            portfolio.resources.game.settle_timer += delta;
            if portfolio.resources.game.settle_timer >= SETTLE_DELAY {
                portfolio.resources.game.phase = GamePhase::Failed;
                portfolio.resources.game.dirty = true;
            }
        } else {
            portfolio.resources.game.settle_timer = 0.0;
        }
    }

    if std::mem::take(&mut portfolio.resources.game.dirty) {
        post_status(portfolio);
    }
}

fn start(portfolio: &mut PortfolioWorld, world: &mut World, level: u32, intro: bool) {
    stop_cutscene(world);
    clear_arena(portfolio, world);
    world.resources.physics.enabled = true;

    build_arena(portfolio, world);
    let layout = layout(level);
    for block in &layout.blocks {
        let mass = (block.size.x * block.size.y * block.size.z).max(0.2) * 1.5;
        let entity = spawn_dynamic_physics_cube_with_material(
            world,
            arena_origin() + block.position,
            block.size,
            mass,
            create_textured_material(block.color, 0.85, 0.05),
        );
        portfolio.resources.game.blocks.push(entity);
    }
    for &position in &layout.targets {
        let color = Vec3::new(1.0, 0.55, 0.15);
        let world_position = arena_origin() + position;
        let entity = spawn_dynamic_physics_sphere_with_material(
            world,
            world_position,
            TARGET_RADIUS,
            1.0,
            Material {
                base_color: [color.x, color.y, color.z, 1.0],
                emissive_factor: [color.x, color.y, color.z],
                emissive_strength: 3.0,
                roughness: 0.4,
                metallic: 0.0,
                ..Default::default()
            },
        );
        world.core.set_name(entity, Name("Target".to_string()));
        portfolio
            .resources
            .game
            .targets
            .push((entity, world_position.y));
    }

    let game = &mut portfolio.resources.game;
    game.phase = if intro {
        GamePhase::Intro
    } else {
        GamePhase::Playing
    };
    game.level = level;
    game.score = 0;
    game.shots_left = layout.shots;
    game.shots_total = layout.shots;
    game.targets_total = layout.targets.len() as u32;
    game.combo = 0;
    game.combo_timer = 0.0;
    game.settle_timer = 0.0;
    game.dirty = true;

    if intro {
        begin_intro(portfolio, world, level);
    } else {
        aim_camera(world);
    }
}

/// Switches to the dedicated cutscene camera and plays the level's intro
/// sweep. The orbit camera is stashed so the finish can hand control back.
fn begin_intro(portfolio: &mut PortfolioWorld, world: &mut World, level: u32) {
    portfolio.resources.game.orbit_camera = world.resources.active_camera;

    let camera = match portfolio.resources.game.cutscene_camera {
        Some(camera) => camera,
        None => {
            let camera = spawn_camera(
                world,
                aim_focus() + Vec3::new(0.0, 16.0, -26.0),
                "Siege Cutscene Camera".to_string(),
            );
            if let Some(component) = world.core.get_camera_mut(camera) {
                component.smoothing = None;
            }
            portfolio.resources.game.cutscene_camera = Some(camera);
            camera
        }
    };
    set_cutscene_camera(world, camera);
    world.resources.active_camera = Some(camera);

    let field_of_view = portfolio
        .resources
        .game
        .orbit_camera
        .and_then(|orbit| world.core.get_camera(orbit))
        .and_then(|component| match &component.projection {
            Projection::Perspective(perspective) => Some(perspective.y_fov_rad.to_degrees()),
            _ => None,
        })
        .unwrap_or(45.0);

    play_cutscene(world, intro_cutscene(level, field_of_view));
    world.resources.cutscene.hold_on_finish = false;
}

/// The arena flyover: letterboxed title sweep that ends exactly on the orbit
/// camera's aim pose so the handoff to player control is seamless.
fn intro_cutscene(level: u32, field_of_view: f32) -> Cutscene {
    let origin = arena_origin();
    let focus = aim_focus();
    let (final_eye, _) = compute_pan_orbit_transform(focus, AIM_YAW, AIM_PITCH, AIM_RADIUS);
    let shot =
        |eye: Vec3, target: Vec3, fov: f32| CutsceneShot::new(eye, target).with_field_of_view(fov);

    Cutscene::new(game_level_name(level))
        .fade_in(0.0, 0.8)
        .letterbox_in(0.0, 0.6)
        .title(0.6, 2.8, "NIGHTSHADE SIEGE")
        .camera_path(
            0.0,
            7.6,
            EasingFunction::SineInOut,
            vec![
                shot(
                    origin + Vec3::new(2.0, 17.0, -27.0),
                    origin + Vec3::new(0.0, 3.0, 0.0),
                    50.0,
                ),
                shot(
                    origin + Vec3::new(-14.0, 3.2, -7.0),
                    origin + Vec3::new(0.0, 2.8, 0.0),
                    44.0,
                ),
                shot(
                    origin + Vec3::new(-9.0, 5.0, 13.0),
                    origin + Vec3::new(0.0, 2.5, 0.0),
                    46.0,
                ),
                shot(final_eye, focus, field_of_view),
            ],
        )
        .dialogue(1.2, 2.8, None, level_caption(level))
        .dialogue(4.4, 2.6, None, "Knock every glowing target from its perch.")
        .letterbox_out(6.8, 0.8)
}

fn level_caption(level: u32) -> &'static str {
    match level {
        1 => "Three watchtowers guard the first arena.",
        2 => "Two keeps share a single bridge.",
        3 => "A wall shields the five wardens behind it.",
        4 => "The citadel rises, ringed by sentries.",
        _ => "The spire holds the final stand.",
    }
}

/// Hands control back to the orbit camera, snapped to the cutscene's final
/// pose so nothing jumps.
fn finish_intro(portfolio: &mut PortfolioWorld, world: &mut World) {
    if let Some(camera) = portfolio.resources.game.orbit_camera {
        if let Some(orbit) = world.core.get_pan_orbit_camera_mut(camera) {
            orbit.focus = aim_focus();
            orbit.yaw = AIM_YAW;
            orbit.pitch = AIM_PITCH;
            orbit.radius = AIM_RADIUS;
            orbit.target_focus = aim_focus();
            orbit.target_yaw = AIM_YAW;
            orbit.target_pitch = AIM_PITCH;
            orbit.target_radius = AIM_RADIUS;
        }
        world.resources.active_camera = Some(camera);
    }
    portfolio.resources.game.phase = GamePhase::Playing;
    portfolio.resources.game.dirty = true;
}

/// Jumps the intro to its end; the next frame's advance applies the remaining
/// events and the finish handoff runs.
fn skip_intro(portfolio: &mut PortfolioWorld, world: &mut World) {
    if portfolio.resources.game.phase != GamePhase::Intro {
        return;
    }
    if let Some(active) = &world.resources.cutscene.active {
        world.resources.cutscene.time = active.duration();
    } else {
        finish_intro(portfolio, world);
    }
}

fn fire(portfolio: &mut PortfolioWorld, world: &mut World, x: f32, y: f32) {
    if portfolio.resources.game.phase != GamePhase::Playing
        || portfolio.resources.game.shots_left == 0
    {
        return;
    }
    let Some(ray) = PickingRay::from_screen_position(world, Vec2::new(x, y)) else {
        return;
    };
    let origin = ray.origin + ray.direction * 1.2;
    let color = Vec3::new(0.62, 0.32, 0.92);
    let entity = spawn_dynamic_physics_sphere_with_material(
        world,
        origin,
        PROJECTILE_RADIUS,
        PROJECTILE_MASS,
        Material {
            base_color: [color.x, color.y, color.z, 1.0],
            emissive_factor: [color.x, color.y, color.z],
            emissive_strength: 2.0,
            roughness: 0.3,
            metallic: 0.2,
            ..Default::default()
        },
    );
    world.core.set_name(entity, Name("Cannonball".to_string()));
    if let Some(body) = world.core.get_rigid_body_mut(entity) {
        body.linvel = [
            ray.direction.x * PROJECTILE_SPEED,
            ray.direction.y * PROJECTILE_SPEED,
            ray.direction.z * PROJECTILE_SPEED,
        ];
        body.ccd_enabled = true;
    }
    let game = &mut portfolio.resources.game;
    game.shots_left -= 1;
    game.projectiles.push((entity, 0.0));
    game.dirty = true;
}

fn exit(portfolio: &mut PortfolioWorld, world: &mut World) {
    stop_cutscene(world);
    if let Some(camera) = portfolio.resources.game.orbit_camera {
        world.resources.active_camera = Some(camera);
    }
    if let Some(camera) = portfolio.resources.game.cutscene_camera
        && world.core.entity_has_components(camera, LOCAL_TRANSFORM)
    {
        despawn_recursive_immediate(world, camera);
    }
    clear_arena(portfolio, world);
    world.resources.physics.enabled = false;
    portfolio.resources.game = Default::default();
    post_status(portfolio);
}

fn clear_arena(portfolio: &mut PortfolioWorld, world: &mut World) {
    let game = &mut portfolio.resources.game;
    let mut entities = std::mem::take(&mut game.arena);
    entities.append(&mut game.blocks);
    entities.extend(
        std::mem::take(&mut game.targets)
            .into_iter()
            .map(|(target, _)| target),
    );
    entities.extend(
        std::mem::take(&mut game.projectiles)
            .into_iter()
            .map(|(projectile, _)| projectile),
    );
    for entity in entities {
        if world.core.entity_has_components(entity, LOCAL_TRANSFORM) {
            despawn_recursive_immediate(world, entity);
        }
    }
    world.resources.mesh_render_state.request_full_rebuild();
}

fn build_arena(portfolio: &mut PortfolioWorld, world: &mut World) {
    let origin = arena_origin();
    let platform = spawn_static_physics_cube_with_material(
        world,
        origin + Vec3::new(0.0, -1.0, 0.0),
        Vec3::new(PLATFORM_HALF * 2.0, 2.0, PLATFORM_HALF * 2.0),
        prototype_material(FLOOR_TEXTURE, Vec3::new(0.52, 0.55, 0.66), 0.9, 0.0),
    );
    world.core.set_name(platform, Name("Arena".to_string()));
    portfolio.resources.game.arena.push(platform);

    let trim_color = Vec3::new(0.62, 0.32, 0.92);
    let offset = PLATFORM_HALF + 0.3;
    let trims = [
        (Vec3::new(0.0, -0.2, offset), Vec3::new(17.2, 0.3, 0.5)),
        (Vec3::new(0.0, -0.2, -offset), Vec3::new(17.2, 0.3, 0.5)),
        (Vec3::new(offset, -0.2, 0.0), Vec3::new(0.5, 0.3, 16.0)),
        (Vec3::new(-offset, -0.2, 0.0), Vec3::new(0.5, 0.3, 16.0)),
    ];
    for (position, size) in trims {
        let trim = spawn_cube_at(world, origin + position);
        if let Some(transform) = world.core.get_local_transform_mut(trim) {
            transform.scale = size;
        }
        mark_local_transform_dirty(world, trim);
        spawn_material(
            world,
            trim,
            format!("ArenaTrim_{}", trim.id),
            Material {
                base_color: [trim_color.x, trim_color.y, trim_color.z, 1.0],
                emissive_factor: [trim_color.x, trim_color.y, trim_color.z],
                emissive_strength: 2.5,
                roughness: 1.0,
                metallic: 0.0,
                ..Default::default()
            },
        );
        portfolio.resources.game.arena.push(trim);
    }
}

fn aim_camera(world: &mut World) {
    if let Some(camera) = world.resources.active_camera
        && let Some(orbit) = world.core.get_pan_orbit_camera_mut(camera)
    {
        orbit.target_focus = arena_origin() + Vec3::new(0.0, 2.5, 0.0);
        orbit.target_radius = 26.0;
        orbit.target_yaw = 0.8;
        orbit.target_pitch = 0.35;
    }
}

fn post_status(portfolio: &PortfolioWorld) {
    let game = &portfolio.resources.game;
    crate::post(&WorkerMessage::Game {
        status: GameStatus {
            phase: game.phase,
            level: game.level,
            score: game.score,
            shots_left: game.shots_left,
            shots_total: game.shots_total,
            targets_left: game.targets.len() as u32,
            targets_total: game.targets_total,
            combo: game.combo,
        },
    });
}

fn tower(blocks: &mut Vec<Block>, x: f32, z: f32, count: u32, size: Vec3, colors: &[Vec3]) -> f32 {
    for index in 0..count {
        blocks.push(Block {
            position: Vec3::new(x, size.y * (index as f32 + 0.5), z),
            size,
            color: colors[index as usize % colors.len()],
        });
    }
    size.y * count as f32
}

fn layout(level: u32) -> Layout {
    let slate = Vec3::new(0.42, 0.46, 0.58);
    let violet = Vec3::new(0.52, 0.36, 0.72);
    let teal = Vec3::new(0.30, 0.58, 0.60);
    let sand = Vec3::new(0.72, 0.62, 0.48);

    let mut blocks = Vec::new();
    let mut targets = Vec::new();
    let shots;

    match level {
        1 => {
            shots = 12;
            let size = Vec3::new(1.2, 1.2, 1.2);
            for (x, z) in [(-5.5, -5.5), (5.5, -5.5), (0.0, 5.5)] {
                let top = tower(&mut blocks, x, z, 3, size, &[slate, teal]);
                targets.push(Vec3::new(x, top + TARGET_RADIUS, z));
            }
        }
        2 => {
            shots = 10;
            let size = Vec3::new(1.4, 1.1, 1.4);
            let top = tower(&mut blocks, -4.5, 0.0, 5, size, &[slate, violet]);
            tower(&mut blocks, 4.5, 0.0, 5, size, &[slate, violet]);
            let plank = Vec3::new(10.4, 0.35, 1.2);
            blocks.push(Block {
                position: Vec3::new(0.0, top + plank.y * 0.5, 0.0),
                size: plank,
                color: sand,
            });
            for x in [-2.0, 2.0] {
                targets.push(Vec3::new(x, top + plank.y + TARGET_RADIUS, 0.0));
            }
            let pedestal = Vec3::new(1.1, 1.1, 1.1);
            for z in [-5.5, 5.5] {
                let pedestal_top = tower(&mut blocks, 0.0, z, 1, pedestal, &[teal]);
                targets.push(Vec3::new(0.0, pedestal_top + TARGET_RADIUS, z));
            }
        }
        3 => {
            shots = 10;
            let brick = Vec3::new(1.3, 1.0, 0.8);
            for row in 0..4 {
                for column in 0..6 {
                    blocks.push(Block {
                        position: Vec3::new(
                            -3.25 + column as f32 * 1.3,
                            brick.y * (row as f32 + 0.5),
                            -1.0,
                        ),
                        size: brick,
                        color: if (row + column) % 2 == 0 { slate } else { sand },
                    });
                }
            }
            let pedestal = Vec3::new(0.9, 1.6, 0.9);
            for index in 0..5 {
                let x = -4.0 + index as f32 * 2.0;
                let top = tower(&mut blocks, x, 3.0, 1, pedestal, &[violet]);
                targets.push(Vec3::new(x, top + TARGET_RADIUS, 3.0));
            }
        }
        4 => {
            shots = 10;
            let keep = Vec3::new(1.5, 1.0, 1.5);
            let top = tower(&mut blocks, 0.0, 0.0, 6, keep, &[slate, violet, teal]);
            targets.push(Vec3::new(0.0, top + TARGET_RADIUS, 0.0));
            let corner = Vec3::new(1.2, 1.1, 1.2);
            for (x, z) in [(-5.5, -5.5), (5.5, -5.5), (-5.5, 5.5), (5.5, 5.5)] {
                let corner_top = tower(&mut blocks, x, z, 3, corner, &[teal, sand]);
                targets.push(Vec3::new(x, corner_top + TARGET_RADIUS, z));
            }
            let pedestal = Vec3::new(0.9, 2.2, 0.9);
            let pedestal_top = tower(&mut blocks, 0.0, 6.5, 1, pedestal, &[violet]);
            targets.push(Vec3::new(0.0, pedestal_top + TARGET_RADIUS, 6.5));
        }
        _ => {
            shots = 9;
            for index in 0..8 {
                let size = if index % 2 == 0 {
                    Vec3::new(1.6, 1.0, 1.6)
                } else {
                    Vec3::new(1.2, 1.0, 1.2)
                };
                blocks.push(Block {
                    position: Vec3::new(0.0, index as f32 + 0.5, 0.0),
                    size,
                    color: [slate, violet, teal, sand][index % 4],
                });
            }
            targets.push(Vec3::new(0.0, 8.0 + TARGET_RADIUS, 0.0));
            let outpost = Vec3::new(1.1, 1.1, 1.1);
            for index in 0..6 {
                let angle = index as f32 * std::f32::consts::TAU / 6.0;
                let x = angle.cos() * 5.5;
                let z = angle.sin() * 5.5;
                let top = tower(&mut blocks, x, z, 2, outpost, &[teal, slate]);
                targets.push(Vec3::new(x, top + TARGET_RADIUS, z));
            }
        }
    }

    Layout {
        shots,
        blocks,
        targets,
    }
}
