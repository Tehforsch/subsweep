use serde::Deserialize;

use crate::named::Named;

#[derive(Deserialize, Named)]
#[name = "tree"]
pub struct QuadTreeConfig {
    pub min_depth: usize,
    pub max_depth: usize,
    pub max_num_particles_per_leaf: usize,
}

impl Default for QuadTreeConfig {
    fn default() -> Self {
        Self {
            min_depth: 1,
            max_depth: 20,
            max_num_particles_per_leaf: 1,
        }
    }
}
