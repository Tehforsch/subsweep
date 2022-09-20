use bevy::prelude::ParallelSystemDescriptorCoercion;

use super::gravity_system;
use super::parameters::Parameters;
use super::GravityCalculationReply;
use super::GravityCalculationRequest;
use crate::communication::CommunicationPlugin;
use crate::communication::CommunicationType;
use crate::communication::Identified;
use crate::domain::communicate_mass_moments_system;
use crate::domain::construct_quad_tree_system;
use crate::named::Named;
use crate::physics::PhysicsStages;
use crate::simulation::Simulation;
use crate::simulation::TenetPlugin;

#[derive(Named)]
pub struct GravityPlugin;

impl TenetPlugin for GravityPlugin {
    fn build_everywhere(&self, sim: &mut Simulation) {
        sim.add_parameter_type::<Parameters>("gravity")
            .add_system_to_stage(PhysicsStages::Physics, construct_quad_tree_system)
            .add_system_to_stage(
                PhysicsStages::Physics,
                communicate_mass_moments_system.after(construct_quad_tree_system),
            )
            .add_system_to_stage(
                PhysicsStages::Physics,
                gravity_system.after(communicate_mass_moments_system),
            )
            .add_plugin(
                CommunicationPlugin::<Identified<GravityCalculationRequest>>::new(
                    CommunicationType::Exchange,
                ),
            )
            .add_plugin(
                CommunicationPlugin::<Identified<GravityCalculationReply>>::new(
                    CommunicationType::Exchange,
                ),
            );
    }
}
