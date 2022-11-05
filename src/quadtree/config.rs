use serde::Deserialize;
use serde::Serialize;

/// Parameters controlling the construction of a tree.
#[derive(Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct QuadTreeConfig {
    pub min_depth: usize,
    /// The maximum depth of the tree. Should be high enough to ensure
    /// that the tree can keep an approximately constant number of
    /// particles per leaf node. Should not be too high in order to
    /// prevent "infinite subdivisions" in edge cases of many
    /// particles at very similar positions.
    pub max_depth: usize,
    /// The maximum number of particles that a leaf will be filled
    /// with before it is subdivided. The maximum can be exceeded if
    /// the leaf node is at max_depth and will therefore not be
    /// subvidivided any further
    pub max_num_particles_per_leaf: usize,
}

impl Default for QuadTreeConfig {
    fn default() -> Self {
        Self {
            min_depth: 1,
            max_depth: 20,
            max_num_particles_per_leaf: 30,
        }
    }
}
