use serde::Deserialize;
use serde::Serialize;

use crate::named::Named;
use crate::units::Dimensionless;
use crate::units::Length;

/// Parameters for gravity. Only needed if the
/// [GravityPlugin](crate::prelude::GravityPlugin) is added
/// to the simulation.
#[derive(Clone, Serialize, Deserialize, Named)]
#[name = "gravity"]
#[serde(deny_unknown_fields)]
pub struct GravityParameters {
    /// The minimum length in the gravity calculations. Should be
    /// large enough to prevent extremely high accelerations on
    /// particles that are very close, but low enough for the results
    /// to still be accurate.
    #[serde(default)]
    pub softening_length: Length,
    /// During the tree walk in the gravity calculation, any encountered node
    /// which is seen from the particle under an angle less than the opening_angle
    /// (meaning the node is far away compared to its size), will not be opened
    /// and the force will instead be approximated by mass moments of the node.
    #[serde(default)]
    pub opening_angle: Dimensionless,
}
