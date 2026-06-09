use crate::ecs::PortfolioWorld;
use nightshade::prelude::*;
use protocol::GamePhase;

/// One camera stop along the scroll tour: pan-orbit targets for a section.
struct Keyframe {
    focus: [f32; 3],
    yaw: f32,
    pitch: f32,
    radius: f32,
}

/// The camera stops, in page-section order: hero, highlights, experience,
/// projects, crates, finale.
const KEYFRAMES: [Keyframe; 6] = [
    Keyframe {
        focus: [0.0, 2.4, 0.0],
        yaw: 0.55,
        pitch: 0.26,
        radius: 12.0,
    },
    Keyframe {
        focus: [0.0, 2.2, -45.0],
        yaw: -0.45,
        pitch: 0.20,
        radius: 11.0,
    },
    Keyframe {
        focus: [0.0, 3.5, -90.0],
        yaw: 0.95,
        pitch: 0.26,
        radius: 15.0,
    },
    Keyframe {
        focus: [0.0, 4.0, -135.0],
        yaw: -0.75,
        pitch: 0.34,
        radius: 17.0,
    },
    Keyframe {
        focus: [0.0, 4.0, -135.0],
        yaw: 0.85,
        pitch: 0.14,
        radius: 11.0,
    },
    Keyframe {
        focus: [0.0, 38.0, -90.0],
        yaw: 0.25,
        pitch: -0.30,
        radius: 26.0,
    },
];

const PROGRESS_RATE: f32 = 4.0;
const GLANCE_YAW: f32 = 0.07;
const GLANCE_PITCH: f32 = 0.045;

/// Chases the page's scroll progress and writes the interpolated keyframe to
/// the pan-orbit camera targets, with a pointer-glance parallax offset. The
/// controller's own smoothing turns the targets into easing. Idle while the
/// siege game owns the camera.
pub fn update(portfolio: &mut PortfolioWorld, world: &mut World) {
    if portfolio.resources.game.phase != GamePhase::Idle {
        return;
    }

    let delta = world.resources.window.timing.delta_time;
    let tour = &mut portfolio.resources.tour;
    let rate = (delta * PROGRESS_RATE).min(1.0);
    tour.progress += (tour.target_progress - tour.progress) * rate;

    let span = (KEYFRAMES.len() - 1) as f32;
    let scaled = (tour.progress.clamp(0.0, 1.0)) * span;
    let index = (scaled.floor() as usize).min(KEYFRAMES.len() - 2);
    let fraction = smoothstep(scaled - index as f32);
    let from = &KEYFRAMES[index];
    let to = &KEYFRAMES[index + 1];

    let mut yaw = lerp(from.yaw, to.yaw, fraction);
    let mut pitch = lerp(from.pitch, to.pitch, fraction);
    if !tour.reduced_motion {
        yaw += tour.glance.x * GLANCE_YAW;
        pitch += tour.glance.y * GLANCE_PITCH;
    }

    let Some(camera) = world.resources.active_camera else {
        return;
    };
    if let Some(orbit) = world.core.get_pan_orbit_camera_mut(camera) {
        let from_focus = Vec3::new(from.focus[0], from.focus[1], from.focus[2]);
        let to_focus = Vec3::new(to.focus[0], to.focus[1], to.focus[2]);
        orbit.target_focus = nalgebra_glm::lerp(&from_focus, &to_focus, fraction);
        orbit.target_radius = lerp(from.radius, to.radius, fraction);
        orbit.target_yaw = yaw;
        orbit.target_pitch = pitch;
    }
}

fn lerp(from: f32, to: f32, fraction: f32) -> f32 {
    from + (to - from) * fraction
}

fn smoothstep(fraction: f32) -> f32 {
    let clamped = fraction.clamp(0.0, 1.0);
    clamped * clamped * (3.0 - 2.0 * clamped)
}
