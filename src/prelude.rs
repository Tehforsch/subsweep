pub use crate::communication::WorldRank;
pub use crate::communication::WorldSize;
pub use crate::mass::Mass;
pub use crate::named::*;
pub use crate::particle::LocalParticle;
pub use crate::particle::Particles;
pub use crate::physics::hydrodynamics::HydrodynamicsPlugin;
pub use crate::physics::GravityPlugin;
pub use crate::physics::Timestep;
pub use crate::position::Position;
pub use crate::simulation::Simulation;
pub use crate::simulation_builder::SimulationBuilder;
pub use crate::units;
pub use crate::velocity::Velocity;
pub use crate::visualization::CameraTransform;
pub use crate::visualization::DrawCircle;
pub use crate::visualization::DrawRect;

#[cfg(feature = "2d")]
pub type MVec = glam::DVec2;
#[cfg(not(feature = "2d"))]
pub type MVec = glam::DVec3;
