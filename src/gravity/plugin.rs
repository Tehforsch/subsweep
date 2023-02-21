use bevy::prelude::IntoSystemDescriptor;

use super::gravity_system;
use super::parameters::GravityParameters;
use super::GravityCalculationReply;
use super::GravityCalculationRequest;
use crate::communication::CommunicationPlugin;
use crate::communication::Identified;
use crate::components::Mass;
use crate::components::Velocity;
use crate::domain::communicate_mass_moments_system;
use crate::domain::construct_quad_tree_system;
use crate::named::Named;
use crate::simulation::RaxiomPlugin;
use crate::simulation::Simulation;
use crate::simulation_plugin::integrate_motion_system;
use crate::simulation_plugin::SimulationStages;

#[derive(Named)]
pub struct GravityPlugin;

impl RaxiomPlugin for GravityPlugin {
    fn build_everywhere(&self, sim: &mut Simulation) {
        sim.add_parameter_type::<GravityParameters>()
            .add_system_to_stage(
                SimulationStages::ForceCalculation,
                construct_quad_tree_system,
            )
            .add_system_to_stage(
                SimulationStages::ForceCalculation,
                communicate_mass_moments_system.after(construct_quad_tree_system),
            )
            .add_system_to_stage(
                SimulationStages::ForceCalculation,
                gravity_system
                    .after(communicate_mass_moments_system)
                    .before(integrate_motion_system),
            )
            .add_required_component::<Velocity>()
            .add_required_component::<Mass>()
            .add_plugin(CommunicationPlugin::<Identified<GravityCalculationRequest>>::exchange())
            .add_plugin(CommunicationPlugin::<Identified<GravityCalculationReply>>::exchange());
    }
}
