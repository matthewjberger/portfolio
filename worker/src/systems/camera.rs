use nightshade::prelude::*;

/// Guarantees a controllable active camera every frame. If the active camera
/// is missing, a fresh pan-orbit camera is spawned and made active so the tour
/// and the game always have something to drive.
pub fn ensure_active(world: &mut World) {
    let valid = world
        .resources
        .active_camera
        .is_some_and(|camera| world.core.entity_has_components(camera, CAMERA));
    if valid {
        return;
    }
    let camera = spawn_pan_orbit_camera(
        world,
        Vec3::new(0.0, 2.5, 0.0),
        14.0,
        0.6,
        0.3,
        "Camera".to_string(),
    );
    world.resources.active_camera = Some(camera);
    world.core.add_components(camera, VIEWPORT_SHADING);
    world.core.set_viewport_shading(
        camera,
        nightshade::ecs::camera::components::ViewportShading::default(),
    );
}
