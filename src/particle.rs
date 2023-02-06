use bevy::ecs::archetype::Archetypes;
use bevy::ecs::component::Components;
use bevy::prelude::debug;
use bevy::prelude::Bundle;
use bevy::prelude::Component;
use bevy::prelude::Query;
use bevy::prelude::StartupStage;
use bevy::prelude::With;
use mpi::traits::Equivalence;

use crate::components::Mass;
use crate::components::Position;
use crate::components::Velocity;
use crate::named::Named;
use crate::prelude::Simulation;
use crate::simulation::RaxiomPlugin;

#[derive(Component, Clone, Debug, PartialEq, Eq, Hash, Equivalence, Copy)]
pub struct ParticleId(pub usize);

#[derive(Component)]
pub struct LocalParticle;

/// A convenience type to query for particles.
/// ```
/// # use raxiom::components::Velocity;
/// # use raxiom::components::Mass;
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
        .filter(|archetype| !archetype.is_empty())
        .collect();
    debug!("Num archetypes on main rank: {}", relevant_archetypes.len());
    for archetype in relevant_archetypes.iter() {
        debug!("----");
        for component in archetype.components() {
            let info = components.get_info(component).unwrap();
            debug!("  {}", info.name());
        }
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
        world.spawn((A, B, LocalParticle));
        world.spawn((A, B, LocalParticle));
        world.spawn((A, LocalParticle));
        world.spawn((A,));
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
        world.spawn((A, B, C, LocalParticle));
        world.spawn((A, B, LocalParticle));
        world.spawn((A, LocalParticle));
        world.spawn((A,));
        fn system(particles: Particles<&A, (With<B>, With<C>)>) {
            assert_eq!(particles.iter().count(), 1);
        }
        run_system_on_world(&mut world, system);
    }
}
