use bevy::ecs::archetype::Archetypes;
use bevy::ecs::component::Components;
use bevy::prelude::debug;
use bevy::prelude::Bundle;
use bevy::prelude::Component;
use bevy::prelude::Query;
use bevy::prelude::StartupStage;
use bevy::prelude::With;

use crate::mass::Mass;
use crate::named::Named;
use crate::position::Position;
use crate::prelude::Simulation;
use crate::simulation::RaxiomPlugin;
use crate::velocity::Velocity;

#[derive(Component)]
pub struct LocalParticle;

/// A convenience type to query for particles.
/// ```
/// # use raxiom::prelude::Velocity;
/// # use raxiom::prelude::Mass;
/// # use raxiom::prelude::Particles;
/// fn my_system(particles: Particles<(&Velocity, &Mass)>) {
///     for (velocity, mass) in particles.iter() {
///        println!("Particle with mass  {} kg moving at {} m/s", mass.in_kilograms(), velocity.in_meters_per_second());
///     }
/// }
/// ```
pub type Particles<'world, 'state, T, F = ()> = Query<'world, 'state, T, (With<LocalParticle>, F)>;

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

#[cfg(test)]
mod tests {
    use bevy::prelude::Component;
    use bevy::prelude::With;
    use bevy::prelude::World;

    use crate::prelude::LocalParticle;
    use crate::prelude::Particles;
    use crate::test_utils::run_system_on_world;

    #[test]
    fn particles_query_respects_filters() {
        #[derive(Component)]
        struct A;
        #[derive(Component)]
        struct B;
        let mut world = World::default();
        world.spawn().insert_bundle((A, B, LocalParticle));
        world.spawn().insert_bundle((A, B, LocalParticle));
        world.spawn().insert_bundle((A, LocalParticle));
        world.spawn().insert_bundle((A,));
        fn system(particles: Particles<&A, With<B>>) {
            assert_eq!(particles.iter().count(), 2);
        }
        run_system_on_world(&mut world, system);
    }

    #[test]
    fn particles_query_respects_tuple_filters() {
        #[derive(Component)]
        struct A;
        #[derive(Component)]
        struct B;
        #[derive(Component)]
        struct C;
        let mut world = World::default();
        world.spawn().insert_bundle((A, B, C, LocalParticle));
        world.spawn().insert_bundle((A, B, LocalParticle));
        world.spawn().insert_bundle((A, LocalParticle));
        world.spawn().insert_bundle((A,));
        fn system(particles: Particles<&A, (With<B>, With<C>)>) {
            assert_eq!(particles.iter().count(), 1);
        }
        run_system_on_world(&mut world, system);
    }
}
