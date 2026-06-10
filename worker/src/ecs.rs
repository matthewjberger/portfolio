mod components;
mod resources;

pub use components::*;
pub use resources::*;

use nightshade::prelude::freecs;

freecs::ecs! {
    PortfolioWorld {
        marker: Marker => MARKER,
    }
    Tags {
    }
    Events {
    }
    Resources {
        tour: Tour,
        ambient: Ambient,
        floaters: Floaters,
        sky: Sky,
        cameras: Cameras,
        game: Game,
    }
}
