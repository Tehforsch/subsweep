use bevy::ecs::archetype::Archetypes;
use bevy::ecs::component::Components;
use bevy::prelude::debug;
use bevy::prelude::Bundle;
use bevy::prelude::StartupStage;

use crate::mass::Mass;
use crate::named::Named;
use crate::physics::LocalParticle;
use crate::position::Position;
use crate::prelude::Simulation;
use crate::simulation::RaxiomPlugin;
use crate::velocity::Velocity;

#[derive(Bundle)]
pub struct LocalParticleBundle {
    pos: Position,
    vel: Velocity,
    mass: Mass,
    _local: LocalParticle,
}

#[derive(Named)]
pub struct ParticlePlugin;

impl RaxiomPlugin for ParticlePlugin {
    fn build_everywhere(&self, sim: &mut Simulation) {
        sim.add_startup_system_to_stage(StartupStage::PostStartup, count_types_system);
    }
}

fn count_types_system(archetypes: &Archetypes, components: &Components) {
    // Count archetypes
    // Filter out empty archetype and resource archetype
    let relevant_archetypes: Vec<_> = archetypes
        .iter()
        .filter(|archetype| !archetype.is_empty() && archetype.id() != archetypes.resource().id())
        .collect();
    debug!("Num archetypes on main rank: {}", relevant_archetypes.len());
    for archetype in relevant_archetypes.iter() {
        debug!("----");
        for component in archetype.components() {
            let info = components.get_info(component).unwrap();
            debug!("  {}", info.name());
        }
    }
    // Print resources
    let mut resources: Vec<_> = archetypes
        .resource()
        .components()
        .map(|id| components.get_info(id).unwrap())
        .map(|info| info.name())
        .collect();
    resources.sort();
    debug!("Resources on main rank:");
    for res in resources {
        debug!("  {}", res);
    }
}
