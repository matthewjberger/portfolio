use nightshade::prelude::serde::{Deserialize, Serialize};

/// Marker for portfolio-side entities. State lives in resources today, but the
/// world carries this so per-entity state can grow.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(crate = "nightshade::prelude::serde")]
pub struct Marker;
