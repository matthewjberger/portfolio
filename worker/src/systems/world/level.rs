use crate::ecs::{Bobber, Floater, Orbiter, PortfolioWorld, Spinner};
use crate::systems::world::textures::{FLOOR_TEXTURE, prototype_material};
use nightshade::ecs::material::components::Material;
use nightshade::prelude::*;

const HELMET: &[u8] = include_bytes!("../../../assets/DamagedHelmet.glb");

const HIGHLIGHTS_Z: f32 = -45.0;
const EXPERIENCE_Z: f32 = -90.0;
const BELT_Z: f32 = -135.0;
const FINALE_CENTER: [f32; 3] = [0.0, 40.0, -90.0];

/// Builds every section vignette the camera tour flies between.
pub fn build(portfolio: &mut PortfolioWorld, world: &mut World) {
    build_hero_island(portfolio, world);
    build_highlights_shrine(portfolio, world);
    build_experience_canyon(portfolio, world);
    build_crate_belt(portfolio, world);
    build_finale_sky(portfolio, world);
}

fn matte(color: Vec3, roughness: f32) -> Material {
    Material {
        base_color: [color.x, color.y, color.z, 1.0],
        roughness,
        metallic: 0.05,
        ..Default::default()
    }
}

fn glow(color: Vec3, strength: f32) -> Material {
    Material {
        base_color: [color.x, color.y, color.z, 1.0],
        emissive_factor: [color.x, color.y, color.z],
        emissive_strength: strength,
        roughness: 0.6,
        metallic: 0.0,
        ..Default::default()
    }
}

fn violet() -> Vec3 {
    Vec3::new(0.62, 0.32, 0.92)
}

fn orange() -> Vec3 {
    Vec3::new(1.0, 0.55, 0.15)
}

fn teal() -> Vec3 {
    Vec3::new(0.25, 0.72, 0.70)
}

fn slate() -> Vec3 {
    Vec3::new(0.40, 0.44, 0.56)
}

/// The tint the tiled prototype grid texture is multiplied by on every deck.
fn deck() -> Vec3 {
    Vec3::new(0.52, 0.55, 0.66)
}

fn spawn_block(world: &mut World, position: Vec3, size: Vec3, material: Material) -> Entity {
    let entity = spawn_cube_at(world, position);
    if let Some(transform) = world.core.get_local_transform_mut(entity) {
        transform.scale = size;
    }
    mark_local_transform_dirty(world, entity);
    spawn_material(world, entity, format!("Block_{}", entity.id), material);
    entity
}

fn spawn_orb(world: &mut World, position: Vec3, radius: f32, material: Material) -> Entity {
    let entity = spawn_sphere_at(world, position);
    if let Some(transform) = world.core.get_local_transform_mut(entity) {
        transform.scale = Vec3::new(radius, radius, radius);
    }
    mark_local_transform_dirty(world, entity);
    spawn_material(world, entity, format!("Orb_{}", entity.id), material);
    entity
}

fn spawn_shape(
    world: &mut World,
    mesh: &str,
    position: Vec3,
    scale: Vec3,
    material: Material,
) -> Entity {
    let entity = match mesh {
        "Torus" => spawn_torus_at(world, position),
        "Cone" => spawn_cone_at(world, position),
        "Cylinder" => spawn_cylinder_at(world, position),
        _ => spawn_sphere_at(world, position),
    };
    if let Some(transform) = world.core.get_local_transform_mut(entity) {
        transform.scale = scale;
    }
    mark_local_transform_dirty(world, entity);
    spawn_material(world, entity, format!("Shape_{}", entity.id), material);
    entity
}

/// A floating platform faced with the prototype grid and ringed by an
/// emissive trim, shared by the hero island and the highlight shrine.
fn spawn_platform(world: &mut World, center: Vec3, width: f32, depth: f32, trim: Vec3) {
    spawn_block(
        world,
        center + Vec3::new(0.0, -0.75, 0.0),
        Vec3::new(width, 1.5, depth),
        prototype_material(FLOOR_TEXTURE, deck(), 0.85, 0.05),
    );
    let half_width = width * 0.5 + 0.25;
    let half_depth = depth * 0.5 + 0.25;
    let strips = [
        (
            Vec3::new(0.0, -0.18, half_depth),
            Vec3::new(width + 0.9, 0.26, 0.4),
        ),
        (
            Vec3::new(0.0, -0.18, -half_depth),
            Vec3::new(width + 0.9, 0.26, 0.4),
        ),
        (
            Vec3::new(half_width, -0.18, 0.0),
            Vec3::new(0.4, 0.26, depth),
        ),
        (
            Vec3::new(-half_width, -0.18, 0.0),
            Vec3::new(0.4, 0.26, depth),
        ),
    ];
    for (offset, size) in strips {
        spawn_block(world, center + offset, size, glow(trim, 2.2));
    }
}

fn build_hero_island(portfolio: &mut PortfolioWorld, world: &mut World) {
    spawn_platform(world, Vec3::new(0.0, 0.0, 0.0), 11.0, 11.0, violet());

    match import_gltf_from_bytes(HELMET) {
        Ok(mut result) => {
            nightshade::ecs::loading::queue_gltf_load(world, &mut result);
            for prefab in &result.prefabs {
                let root = nightshade::ecs::prefab::spawn_prefab_with_skins(
                    world,
                    prefab,
                    &result.animations,
                    &result.skins,
                    Vec3::new(0.0, 2.6, 0.0),
                );
                if let Some(transform) = world.core.get_local_transform_mut(root) {
                    transform.scale = Vec3::new(1.7, 1.7, 1.7);
                }
                mark_local_transform_dirty(world, root);
                portfolio.resources.ambient.spinners.push(Spinner {
                    entity: root,
                    axis: Vec3::new(0.0, 1.0, 0.0),
                    radians_per_second: 0.25,
                });
            }
        }
        Err(error) => tracing::error!("failed to import the hero model: {error}"),
    }

    for index in 0..8_u32 {
        let angle = index as f32 * std::f32::consts::TAU / 8.0;
        let radius = 4.6 + (index % 3) as f32 * 0.5;
        let height = 1.6 + (index % 4) as f32 * 0.7;
        let home = Vec3::new(angle.cos() * radius, height, angle.sin() * radius);
        let color = match index % 3 {
            0 => violet(),
            1 => orange(),
            _ => teal(),
        };
        let orb = spawn_orb(
            world,
            home,
            0.22 + (index % 2) as f32 * 0.08,
            glow(color, 2.6),
        );
        portfolio.resources.floaters.items.push(Floater {
            entity: orb,
            home,
            velocity: Vec3::new(0.0, 0.0, 0.0),
        });
    }
}

fn build_highlights_shrine(portfolio: &mut PortfolioWorld, world: &mut World) {
    let center = Vec3::new(0.0, 0.0, HIGHLIGHTS_Z);
    spawn_platform(world, center, 13.0, 8.0, teal());

    let pedestals = [
        (-3.8_f32, "Torus", violet(), 2.4_f32),
        (0.0, "Sphere", orange(), 2.8),
        (3.8, "Cone", teal(), 2.4),
    ];
    for (offset_x, mesh, color, strength) in pedestals {
        let base = center + Vec3::new(offset_x, 1.0, 0.0);
        spawn_block(world, base, Vec3::new(1.5, 2.0, 1.5), matte(slate(), 0.85));
        let artifact_home = base + Vec3::new(0.0, 2.1, 0.0);
        let artifact = spawn_shape(
            world,
            mesh,
            artifact_home,
            Vec3::new(0.9, 0.9, 0.9),
            glow(color, strength),
        );
        portfolio.resources.ambient.spinners.push(Spinner {
            entity: artifact,
            axis: Vec3::new(0.3, 1.0, 0.15).normalize(),
            radians_per_second: 0.8,
        });
        portfolio.resources.ambient.bobbers.push(Bobber {
            entity: artifact,
            base: artifact_home,
            amplitude: 0.25,
            frequency: 0.7,
            phase: offset_x,
        });
    }
}

fn build_experience_canyon(portfolio: &mut PortfolioWorld, world: &mut World) {
    let center = Vec3::new(0.0, 0.0, EXPERIENCE_Z);
    spawn_block(
        world,
        center + Vec3::new(0.0, -0.75, 0.0),
        Vec3::new(7.0, 1.5, 26.0),
        prototype_material(FLOOR_TEXTURE, deck(), 0.85, 0.05),
    );

    let monoliths = [
        (-2.3_f32, 8.5_f32, 6.5_f32, violet()),
        (2.3, 3.0, 5.2, orange()),
        (-2.3, -2.5, 4.2, teal()),
        (2.3, -8.0, 3.4, slate()),
    ];
    for (index, (x, z, height, color)) in monoliths.into_iter().enumerate() {
        let position = center + Vec3::new(x, height * 0.5, z);
        spawn_block(
            world,
            position,
            Vec3::new(1.7, height, 1.3),
            matte(slate() * (0.8 + index as f32 * 0.08), 0.85),
        );
        let cap_home = center + Vec3::new(x, height + 0.55, z);
        let cap = spawn_orb(world, cap_home, 0.32, glow(color, 2.8));
        portfolio.resources.ambient.bobbers.push(Bobber {
            entity: cap,
            base: cap_home,
            amplitude: 0.18,
            frequency: 0.9,
            phase: index as f32 * 1.3,
        });
    }
}

fn build_crate_belt(portfolio: &mut PortfolioWorld, world: &mut World) {
    let center = Vec3::new(0.0, 4.0, BELT_Z);
    spawn_orb(world, center, 1.3, glow(orange(), 3.2));

    for index in 0..28_u32 {
        let phase = index as f32 * 2.39996;
        let radius = 4.5 + ((index * 37) % 45) as f32 * 0.1;
        let height = center.y - 1.8 + ((index * 23) % 36) as f32 * 0.1;
        let size = 0.35 + ((index * 53) % 50) as f32 * 0.011;
        let speed = 0.12 + ((index * 17) % 20) as f32 * 0.012;
        let color = match index % 5 {
            0 => violet(),
            1 => teal(),
            2 => orange() * 0.8,
            _ => slate(),
        };
        let material = if index % 5 < 3 {
            glow(color, 1.6)
        } else {
            matte(color, 0.8)
        };
        let position = Vec3::new(
            center.x + phase.cos() * radius,
            height,
            center.z + phase.sin() * radius,
        );
        let cube = spawn_block(world, position, Vec3::new(size, size, size), material);
        portfolio.resources.ambient.orbiters.push(Orbiter {
            entity: cube,
            center: Vec3::new(center.x, height, center.z),
            radius,
            height,
            radians_per_second: speed,
            phase,
        });
        portfolio.resources.ambient.spinners.push(Spinner {
            entity: cube,
            axis: Vec3::new(0.4, 1.0, 0.6).normalize(),
            radians_per_second: 0.4 + speed,
        });
    }
}

fn build_finale_sky(portfolio: &mut PortfolioWorld, world: &mut World) {
    let center = Vec3::new(FINALE_CENTER[0], FINALE_CENTER[1], FINALE_CENTER[2]);
    spawn_orb(
        world,
        center + Vec3::new(6.0, 8.0, -14.0),
        2.4,
        glow(Vec3::new(0.92, 0.9, 0.85), 1.8),
    );

    for index in 0..60_u32 {
        let phase = index as f32 * 2.39996;
        let ring = 6.0 + ((index * 29) % 200) as f32 * 0.1;
        let height = ((index * 41) % 160) as f32 * 0.1 - 4.0;
        let radius = 0.1 + ((index * 13) % 18) as f32 * 0.012;
        let color = match index % 6 {
            0 => violet(),
            1 => teal(),
            2 => orange(),
            _ => Vec3::new(0.85, 0.87, 0.95),
        };
        let position = center + Vec3::new(phase.cos() * ring, height, phase.sin() * ring * 0.6);
        let star = spawn_orb(world, position, radius, glow(color, 2.4));
        if index % 4 == 0 {
            portfolio.resources.ambient.bobbers.push(Bobber {
                entity: star,
                base: position,
                amplitude: 0.3,
                frequency: 0.5,
                phase,
            });
        }
    }
}
