use crate::ecs::PortfolioWorld;
use nightshade::prelude::*;
use protocol::GamePhase;

const STIFFNESS: f32 = 7.0;
const DAMPING: f32 = 2.8;
const PUSH_RADIUS: f32 = 2.2;
const PUSH_STRENGTH: f32 = 26.0;
const MAX_SPEED: f32 = 9.0;

/// Springs the emissive floaters back to their homes while the pointer's
/// glance ray shoves any it passes near, so the hero orbs scatter around the
/// cursor and drift back. Plain integration, no physics engine involved.
pub fn update(portfolio: &mut PortfolioWorld, world: &mut World) {
    if portfolio.resources.game.phase != GamePhase::Idle || portfolio.resources.tour.reduced_motion
    {
        return;
    }
    let delta = world.resources.window.timing.delta_time.min(0.05);
    let ray = glance_ray(portfolio, world);

    for floater in &mut portfolio.resources.floaters.items {
        let Some(position) = world
            .core
            .get_local_transform(floater.entity)
            .map(|transform| transform.translation)
        else {
            continue;
        };
        let mut acceleration = (floater.home - position) * STIFFNESS - floater.velocity * DAMPING;

        if let Some(ray) = &ray {
            let to_position = position - ray.origin;
            let along = nalgebra_glm::dot(&to_position, &ray.direction).max(0.0);
            let closest = ray.origin + ray.direction * along;
            let away = position - closest;
            let distance = away.norm();
            if distance < PUSH_RADIUS {
                let falloff = 1.0 - distance / PUSH_RADIUS;
                let direction = if distance > 0.001 {
                    away / distance
                } else {
                    Vec3::new(0.0, 1.0, 0.0)
                };
                acceleration += direction * PUSH_STRENGTH * falloff;
            }
        }

        floater.velocity += acceleration * delta;
        let speed = floater.velocity.norm();
        if speed > MAX_SPEED {
            floater.velocity *= MAX_SPEED / speed;
        }
        let next = position + floater.velocity * delta;
        if let Some(transform) = world.core.get_local_transform_mut(floater.entity) {
            transform.translation = next;
        }
        mark_local_transform_dirty(world, floater.entity);
    }
}

/// The pointer's world ray, built from the normalized glance through the
/// camera, or `None` until a real pointer position has arrived.
fn glance_ray(portfolio: &PortfolioWorld, world: &World) -> Option<PickingRay> {
    let glance = portfolio.resources.tour.glance;
    if glance.x == 0.0 && glance.y == 0.0 {
        return None;
    }
    let (width, height) = world.resources.window.cached_viewport_size?;
    let x = (glance.x * 0.5 + 0.5) * width as f32;
    let y = (0.5 - glance.y * 0.5) * height as f32;
    PickingRay::from_screen_position(world, Vec2::new(x, y))
}
