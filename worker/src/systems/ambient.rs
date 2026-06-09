use crate::ecs::PortfolioWorld;
use nightshade::prelude::*;

/// Advances the ambient scene animation: spinners rotate in place, bobbers
/// sine around their base, and orbiters circle their center. Everything is
/// driven from accumulated time so it stays deterministic and cheap.
pub fn update(portfolio: &mut PortfolioWorld, world: &mut World) {
    let delta = world.resources.window.timing.delta_time;
    let reduced = portfolio.resources.tour.reduced_motion;
    let ambient = &mut portfolio.resources.ambient;
    ambient.time += delta;
    let time = ambient.time;

    for spinner in &ambient.spinners {
        let speed = if reduced {
            spinner.radians_per_second * 0.2
        } else {
            spinner.radians_per_second
        };
        if let Some(transform) = world.core.get_local_transform_mut(spinner.entity) {
            let step = nalgebra_glm::quat_angle_axis(speed * delta, &spinner.axis);
            transform.rotation = step * transform.rotation;
        }
        mark_local_transform_dirty(world, spinner.entity);
    }

    if !reduced {
        for bobber in &ambient.bobbers {
            if let Some(transform) = world.core.get_local_transform_mut(bobber.entity) {
                let offset = (time * bobber.frequency * std::f32::consts::TAU + bobber.phase).sin();
                transform.translation =
                    bobber.base + Vec3::new(0.0, offset * bobber.amplitude, 0.0);
            }
            mark_local_transform_dirty(world, bobber.entity);
        }
    }

    for orbiter in &ambient.orbiters {
        let speed = if reduced {
            orbiter.radians_per_second * 0.2
        } else {
            orbiter.radians_per_second
        };
        let angle = orbiter.phase + time * speed;
        if let Some(transform) = world.core.get_local_transform_mut(orbiter.entity) {
            transform.translation = Vec3::new(
                orbiter.center.x + angle.cos() * orbiter.radius,
                orbiter.height,
                orbiter.center.z + angle.sin() * orbiter.radius,
            );
        }
        mark_local_transform_dirty(world, orbiter.entity);
    }
}
