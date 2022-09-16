use bevy::prelude::ParallelSystemDescriptorCoercion;
use bevy::prelude::Plugin;

use super::gravity_system;
use super::GravityCalculationReply;
use super::GravityCalculationRequest;
use crate::communication::CommunicationPlugin;
use crate::communication::CommunicationType;
use crate::communication::Identified;
use crate::domain::construct_quad_tree_system;
use crate::physics::PhysicsStages;

pub struct GravityPlugin;

impl Plugin for GravityPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system_to_stage(
            PhysicsStages::Physics,
            construct_quad_tree_system.before(gravity_system),
        )
        .add_system_to_stage(PhysicsStages::Physics, gravity_system)
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
