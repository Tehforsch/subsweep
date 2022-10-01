use serde::Deserialize;

use crate::named::Named;
use crate::quadtree::QuadTreeConfig;
use crate::units::Length;

#[derive(Deserialize, Named)]
#[name = "hydrodynamics"]
#[serde(deny_unknown_fields)]
pub struct HydrodynamicsParameters {
    pub smoothing_length: Length,
    #[serde(default = "default_hydro_tree")]
    pub tree: QuadTreeConfig,
}

fn default_hydro_tree() -> QuadTreeConfig {
    QuadTreeConfig {
        min_depth: 0,
        max_depth: 20,
        max_num_particles_per_leaf: 30,
    }
}
