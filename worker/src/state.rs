use crate::ecs::PortfolioWorld;
use crate::systems;
use nightshade::prelude::*;

/// The application root. Holds the portfolio-side ECS world and forwards each
/// `State` hook to system functions in `src/systems/`.
#[derive(Default)]
pub struct Portfolio {
    pub portfolio: PortfolioWorld,
}

impl State for Portfolio {
    fn initialize(&mut self, world: &mut World) {
        systems::setup::spawn(&mut self.portfolio, world);
    }

    fn run_systems(&mut self, world: &mut World) {
        systems::camera::ensure_active(&mut self.portfolio, world);
        systems::tour::update(&mut self.portfolio, world);
        pan_orbit_camera_system(world);
        systems::sky::poll(&mut self.portfolio, world);
        systems::ambient::update(&mut self.portfolio, world);
        systems::floaters::update(&mut self.portfolio, world);
        systems::game::update(&mut self.portfolio, world);
    }
}
