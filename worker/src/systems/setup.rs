use crate::ecs::PortfolioWorld;
use crate::systems::world::{level, textures};
use nightshade::prelude::*;

/// Prepares the world once: render settings, lighting, the camera, the skybox
/// fetch, and the section vignettes.
pub fn spawn(portfolio: &mut PortfolioWorld, world: &mut World) {
    world.resources.user_interface.enabled = true;
    world.resources.retained_ui.enabled = true;
    world.resources.user_interface.gizmos.nav_gizmo_enabled = false;
    if let Some((width, height)) = world.resources.window.cached_viewport_size {
        world.resources.window.active_viewport_rect =
            Some(nightshade::ecs::window::resources::ViewportRect {
                x: 0.0,
                y: 0.0,
                width: width as f32,
                height: height as f32,
            });
    }
    world.resources.render_settings.atmosphere = Atmosphere::Sky;
    world.resources.render_settings.clear_color = [0.05, 0.06, 0.09, 1.0];
    capture_procedural_atmosphere_ibl(world, Atmosphere::Sky, 0.0);
    world.resources.render_settings.ssao_enabled = true;
    world.resources.render_settings.bloom_enabled = true;
    world.resources.render_settings.color_grading.exposure = 1.0;
    world.resources.debug_draw.show_grid = false;

    nightshade::ecs::world::commands::load_procedural_textures(world);
    textures::load_prototype_textures(world);

    let sun = spawn_sun(world);
    if let Some(light) = world.core.get_light_mut(sun) {
        light.cast_shadows = true;
        light.intensity = 3.2;
        light.shadow_bias = 0.008;
    }

    crate::systems::camera::ensure_active(portfolio, world);
    crate::systems::sky::fetch(portfolio);

    level::build(portfolio, world);
}
